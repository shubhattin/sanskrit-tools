#!/usr/bin/env bash
# Verify two databases hold identical data (counts + content checksums).
#
#   scripts/parity.sh                       # local (PG_DATABASE_URL) vs prod (PG_DATABASE_URL1)
#   scripts/parity.sh --quick             # counts only (no content scans)
#   scripts/parity.sh --a <url> --b <url> [--quick]
#
# Counts catch missing/extra rows; checksums catch same-count content drift.
# Checksums are commutative aggregates (count + sum(hash) + sum(length)), so
# each table needs one sequential scan and no sorting — ~seconds per table,
# including the 6.2M-row paradigm_forms.
set -euo pipefail
cd "$(dirname "$0")/.."

A=""
B=""
QUICK=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --a) A="$2"; shift 2 ;;
    --b) B="$2"; shift 2 ;;
    --quick) QUICK=1; shift ;;
    -h|--help) sed -n '2,10p' "$0"; exit 0 ;;
    *) echo "unknown option: $1" >&2; exit 1 ;;
  esac
done

if [[ -z "$A" || -z "$B" ]]; then
  set -a; [[ -f .env.local ]] && . ./.env.local; set +a
  [[ -z "$A" ]] && A="${PG_DATABASE_URL:-}"
  [[ -z "$B" ]] && B="${PG_DATABASE_URL1:-}"
fi
[[ -n "$A" ]] || { echo "A side URL missing (--a or PG_DATABASE_URL)" >&2; exit 1; }
[[ -n "$B" ]] || { echo "B side URL missing (--b or PG_DATABASE_URL1)" >&2; exit 1; }

# libpq tools need a CA for sslmode=verify-full; keepalives keep long
# scans alive through NAT/proxies (same as scripts/seed.sh).
libpq_url() {
  local u="$1"
  if [[ "$u" == *sslmode=verify-* && "$u" != *sslrootcert=* ]]; then
    if [[ "$u" == *\?* ]]; then u="$u&sslrootcert=system"; else u="$u?sslrootcert=system"; fi
  fi
  if [[ "$u" == *\?* ]]; then u="$u&keepalives=1&keepalives_idle=30&keepalives_interval=10&keepalives_count=5";
  else u="$u?keepalives=1&keepalives_idle=30&keepalives_interval=10&keepalives_count=5"; fi
  printf '%s' "$u"
}
A="$(libpq_url "$A")"
B="$(libpq_url "$B")"

COUNTS_SQL="
select 'dictionary' as kind, slug as name, entry_count::bigint as n from dictionaries
union all select 'table', 'entries', count(*) from entries
union all select 'table', 'entry_keys', count(*) from entry_keys
union all select 'table', 'sutras', count(*) from sutras
union all select 'table', 'dhatus', count(*) from dhatus
union all select 'table', 'paradigms', count(*) from paradigms
union all select 'table', 'paradigm_forms', count(*) from paradigm_forms
union all select 'table', 'paradigm_layouts', count(*) from paradigm_layouts
order by 1, 2"

TABLES="entries entry_keys sutras dhatus paradigms paradigm_forms paradigm_layouts"
# dictionaries carries import metadata (imported_at wall-clock) that
# legitimately differs after a local re-import — checksum the data columns.
DICT_COLS="id, slug, name, short_name, kind, src_lang, tgt_lang, description, attribution, entry_count, sort_order, source_hash"

fail=0

echo "==> counts (A vs B)"
if diff <(psql "$A" -XAt -F $'\t' -c "$COUNTS_SQL") <(psql "$B" -XAt -F $'\t' -c "$COUNTS_SQL"); then
  echo "OK: row counts identical"
else
  echo "MISMATCH: row counts differ (left: A, right: B)" >&2
  fail=1
fi

if [[ $QUICK -eq 0 ]]; then
  echo "==> content checksums (count | sum(hash) | sum(len) per table)"
  sql="select count(*), coalesce(sum(hashtext(t::text)::bigint),0), coalesce(sum(octet_length(t::text)::bigint),0) from (select $DICT_COLS from dictionaries) t"
  a="$(psql "$A" -XAt -F '|' -c "$sql")"
  b="$(psql "$B" -XAt -F '|' -c "$sql")"
  if [[ "$a" == "$b" ]]; then
    echo "OK: dictionaries ($a)"
  else
    echo "MISMATCH: dictionaries" >&2
    echo "  A: $a" >&2
    echo "  B: $b" >&2
    fail=1
  fi
  for t in $TABLES; do
    sql="select count(*), coalesce(sum(hashtext(t::text)::bigint),0), coalesce(sum(octet_length(t::text)::bigint),0) from $t t"
    a="$(psql "$A" -XAt -F '|' -c "$sql")"
    b="$(psql "$B" -XAt -F '|' -c "$sql")"
    if [[ "$a" == "$b" ]]; then
      echo "OK: $t ($a)"
    else
      echo "MISMATCH: $t" >&2
      echo "  A: $a" >&2
      echo "  B: $b" >&2
      fail=1
    fi
  done
fi

if [[ $fail -eq 0 ]]; then
  echo "PARITY OK: databases hold identical data"
else
  echo "PARITY FAILED" >&2
  exit 1
fi
