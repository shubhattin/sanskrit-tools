#!/usr/bin/env bash
# Seed a Postgres database with the complete dictionary data, reproducibly.
#
#   scripts/seed.sh                      # rebuild from dict_data/ into PG_DATABASE_URL (.env.local)
#   scripts/seed.sh --url <postgres-url> # …into another database
#   DB_MODE=PROD scripts/seed.sh [--dump …] # …into production (PG_DATABASE_URL1)
#   scripts/seed.sh --dump db/sans_dict.dump [--url <postgres-url>]
#                                        # restore a dump from scripts/db-dump.sh (no dict_data needed)
#   scripts/seed.sh --dump db/sans_dict.dump --data-only
#                                        # schema already exists (e.g. created by migrate/push):
#                                        # empty our tables, load data only, restore the drizzle log
#
# Source mode: Cologne downloads (pinned) → drizzle migrations → full import
# (`--refs --force`) → verify.
# Dump mode:   pg_restore --clean (schema + data + drizzle migration log) → verify.
# Data-only:   TRUNCATE our tables → drizzle log (schema `drizzle`) → data of
#              schema `public` (sequential, dependency order) → VACUUM ANALYZE
#              → verify. Leaves extensions and table definitions untouched.
# Verify compares row counts per dictionary/table with scripts/expected-counts.tsv
# and exits non-zero on any difference. Regenerate that file after an
# intentional data change with: scripts/seed.sh --write-expected
set -euo pipefail
cd "$(dirname "$0")/.."

URL=""
DUMP=""
WRITE_EXPECTED=0
DATA_ONLY=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --url) URL="$2"; shift 2 ;;
    --dump) DUMP="$2"; shift 2 ;;
    --write-expected) WRITE_EXPECTED=1; shift ;;
    --data-only) DATA_ONLY=1; shift ;;
    -h|--help) sed -n '2,22p' "$0"; exit 0 ;;
    *) echo "unknown option: $1" >&2; exit 1 ;;
  esac
done

if [[ -z "$URL" ]]; then
  set -a; [[ -f .env.local ]] && . ./.env.local; set +a
  # DB_MODE=PROD → production (PG_DATABASE_URL1), same rule as schema/db-url.ts
  if [[ "${DB_MODE:-}" == "PROD" ]]; then URL="${PG_DATABASE_URL1:-}"; else URL="${PG_DATABASE_URL:-}"; fi
fi
[[ -n "$URL" ]] || { echo "No database URL (pass --url or set PG_DATABASE_URL in .env.local)" >&2; exit 1; }
export PG_DATABASE_URL="$URL"
REDACTED=$(sed -E 's#//[^@]*@#//***@#' <<<"$URL")

COUNTS_SQL="
select 'dictionary' as kind, slug as name, entry_count::bigint as n from dictionaries
union all select 'table', 'entries', count(*) from entries
union all select 'table', 'entry_keys', count(*) from entry_keys
union all select 'table', 'sutras', count(*) from sutras
union all select 'table', 'dhatus', count(*) from dhatus
union all select 'table', 'paradigms', count(*) from paradigms
union all select 'table', 'paradigm_forms', count(*) from paradigm_forms
order by 1, 2"

# libpq tools (psql / pg_dump / pg_restore) need a CA for sslmode=verify-full:
# use the system store unless the URL names one (PlanetScale URLs may omit it).
# TCP keepalives keep long COPYs to distant servers (ap-south-1) alive
# through NAT/proxies that drop idle connections.
libpq_url() {
  local u="$1"
  if [[ "$u" == *sslmode=verify-* && "$u" != *sslrootcert=* ]]; then
    if [[ "$u" == *\?* ]]; then u="$u&sslrootcert=system"; else u="$u?sslrootcert=system"; fi
  fi
  if [[ "$u" == *\?* ]]; then u="$u&keepalives=1&keepalives_idle=30&keepalives_interval=10&keepalives_count=5";
  else u="$u?keepalives=1&keepalives_idle=30&keepalives_interval=10&keepalives_count=5"; fi
  printf '%s' "$u"
}
counts() { psql "$(libpq_url "$URL")" -XAt -F $'\t' -c "$COUNTS_SQL"; }

if [[ $WRITE_EXPECTED -eq 1 ]]; then
  counts > scripts/expected-counts.tsv
  echo "wrote scripts/expected-counts.tsv ($(wc -l < scripts/expected-counts.tsv) rows) from $REDACTED"
  exit 0
fi

echo "==> target: $REDACTED"
if [[ $DATA_ONLY -eq 1 && -z "$DUMP" ]]; then
  echo "--data-only needs --dump <file>" >&2; exit 1
fi
if [[ -n "$DUMP" && $DATA_ONLY -eq 1 ]]; then
  [[ -f "$DUMP" ]] || { echo "dump not found: $DUMP" >&2; exit 1; }
  L="$(libpq_url "$URL")"
  echo "==> emptying tables (schema kept)"
  psql "$L" -v ON_ERROR_STOP=1 -XAq -c "truncate dictionaries, entries, entry_keys, sutras, dhatus,
    paradigms, paradigm_forms, paradigm_layouts restart identity cascade"
  echo "==> drizzle migration log"
  # `pg_restore --schema=X` restores objects in X but not X itself.
  psql "$L" -v ON_ERROR_STOP=1 -XAq -c "drop schema if exists drizzle cascade; create schema drizzle"
  pg_restore --no-owner --no-acl --exit-on-error --schema=drizzle -d "$L" "$DUMP"
  # Foreign keys already exist, so load in dependency order, strictly
  # sequentially: one COPY at a time. Parallel chains plus any concurrent
  # writer (a second seed run, a migrate) deadlocked on PlanetScale before.
  restore_tables() {
    for t in "$@"; do
      echo "    $t"
      pg_restore --data-only --no-owner --no-acl --exit-on-error --schema=public --table="$t" -d "$L" "$DUMP"
    done
  }
  echo "==> data"
  restore_tables dictionaries sutras dhatus paradigm_layouts
  restore_tables entries entry_keys paradigms paradigm_forms
  echo "==> identity sequences"
  psql "$L" -v ON_ERROR_STOP=1 -XAq -c "
    select setval(pg_get_serial_sequence(t, 'id'), coalesce(m, 0) + 1, false)
    from (values
      ('dictionaries', (select max(id) from dictionaries)),
      ('dhatus', (select max(id) from dhatus)),
      ('entries', (select max(id) from entries)),
      ('paradigms', (select max(id) from paradigms)),
      ('paradigm_layouts', (select max(id) from paradigm_layouts))) v(t, m)" >/dev/null
  echo "==> vacuum analyze"
  psql "$L" -v ON_ERROR_STOP=1 -XAq -c "vacuum analyze"
elif [[ -n "$DUMP" ]]; then
  [[ -f "$DUMP" ]] || { echo "dump not found: $DUMP" >&2; exit 1; }
  echo "==> restoring $DUMP"
  pg_restore --clean --if-exists --no-owner --no-acl --exit-on-error -d "$(libpq_url "$URL")" "$DUMP"
else
  [[ -d dict_data ]] || { echo "dict_data/ is missing (or use --dump)" >&2; exit 1; }
  echo "==> Cologne English→Sanskrit sources (pinned, checksummed)"
  bash scripts/fetch-cologne.sh
  echo "==> migrations"
  [[ -d node_modules/drizzle-kit || -d schema/node_modules/drizzle-kit ]] || bun install --frozen-lockfile
  (cd schema && bunx drizzle-kit migrate)
  echo "==> import (all works, reference tables rebuilt)"
  cargo run --release --quiet --bin import -- --refs --force
fi

echo "==> verify"
if diff <(counts) scripts/expected-counts.tsv; then
  echo "OK: $(wc -l < scripts/expected-counts.tsv) counts match scripts/expected-counts.tsv"
  psql "$(libpq_url "$URL")" -XAtc "select 'database size: ' || pg_size_pretty(pg_database_size(current_database()))"
else
  echo "MISMATCH: counts differ from scripts/expected-counts.tsv (left: database, right: expected)" >&2
  exit 1
fi
