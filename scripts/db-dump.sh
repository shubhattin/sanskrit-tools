#!/usr/bin/env bash
# Dump the local database (schema + data + drizzle migration log) to a
# compressed custom-format file that `scripts/seed.sh --dump` restores.
#
#   scripts/db-dump.sh [output]   # default: db/sans_dict.dump
#   DB_MODE=PROD scripts/db-dump.sh db/prod.dump   # dump production instead
set -euo pipefail
cd "$(dirname "$0")/.."
OUT="${1:-db/sans_dict.dump}"
set -a; [[ -f .env.local ]] && . ./.env.local; set +a
# DB_MODE=PROD dumps production (PG_DATABASE_URL1) instead of the local database.
if [[ "${DB_MODE:-}" == "PROD" ]]; then PG_DATABASE_URL="${PG_DATABASE_URL1:-}"; fi
[[ -n "${PG_DATABASE_URL:-}" ]] || { echo "database URL not set" >&2; exit 1; }
# libpq tools (psql / pg_dump / pg_restore) need a CA for sslmode=verify-full:
# use the system store unless the URL names one (PlanetScale URLs may omit it).
libpq_url() {
  local u="$1"
  if [[ "$u" == *sslmode=verify-* && "$u" != *sslrootcert=* ]]; then
    if [[ "$u" == *\?* ]]; then u="$u&sslrootcert=system"; else u="$u?sslrootcert=system"; fi
  fi
  printf '%s' "$u"
}
mkdir -p "$(dirname "$OUT")"
# Only our objects: tables in `public`, the drizzle migration log, and pg_trgm.
# (Excludes provider objects such as PlanetScale's `pscale_extensions` / hypopg.)
pg_dump -Fc -Z 9 --no-owner --no-acl \
  --schema=public --schema=drizzle --extension=pg_trgm \
  -f "$OUT" "$(libpq_url "$PG_DATABASE_URL")"
echo "wrote $OUT ($(du -h "$OUT" | cut -f1))"
