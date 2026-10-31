#!/usr/bin/env bash
# Seed a Postgres database with the complete dictionary data, reproducibly.
#
#   scripts/seed.sh                      # rebuild from dict_data/ into PG_DATABASE_URL (.env.local)
#   scripts/seed.sh --url <postgres-url> # …into another database
#   scripts/seed.sh --dump db/sans_dict.dump [--url <postgres-url>]
#                                        # restore a dump from scripts/db-dump.sh (no dict_data needed)
#
# Source mode: Cologne downloads (pinned) → drizzle migrations → full import
# (`--refs --force`) → verify.
# Dump mode:   pg_restore --clean (schema + data + drizzle migration log) → verify.
# Verify compares row counts per dictionary/table with scripts/expected-counts.tsv
# and exits non-zero on any difference. Regenerate that file after an
# intentional data change with: scripts/seed.sh --write-expected
set -euo pipefail
cd "$(dirname "$0")/.."

URL=""
DUMP=""
WRITE_EXPECTED=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --url) URL="$2"; shift 2 ;;
    --dump) DUMP="$2"; shift 2 ;;
    --write-expected) WRITE_EXPECTED=1; shift ;;
    -h|--help) sed -n '2,15p' "$0"; exit 0 ;;
    *) echo "unknown option: $1" >&2; exit 1 ;;
  esac
done

if [[ -z "$URL" ]]; then
  set -a; [[ -f .env.local ]] && . ./.env.local; set +a
  URL="${PG_DATABASE_URL:-}"
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

counts() { psql "$URL" -XAt -F $'\t' -c "$COUNTS_SQL"; }

if [[ $WRITE_EXPECTED -eq 1 ]]; then
  counts > scripts/expected-counts.tsv
  echo "wrote scripts/expected-counts.tsv ($(wc -l < scripts/expected-counts.tsv) rows) from $REDACTED"
  exit 0
fi

echo "==> target: $REDACTED"
if [[ -n "$DUMP" ]]; then
  [[ -f "$DUMP" ]] || { echo "dump not found: $DUMP" >&2; exit 1; }
  echo "==> restoring $DUMP"
  pg_restore --clean --if-exists --no-owner --no-acl --exit-on-error -d "$URL" "$DUMP"
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
  psql "$URL" -XAtc "select 'database size: ' || pg_size_pretty(pg_database_size(current_database()))"
else
  echo "MISMATCH: counts differ from scripts/expected-counts.tsv (left: database, right: expected)" >&2
  exit 1
fi
