#!/usr/bin/env bash
# Dump the local database (schema + data + drizzle migration log) to a
# compressed custom-format file that `scripts/seed.sh --dump` restores.
#
#   scripts/db-dump.sh [output]   # default: db/sans_dict.dump
set -euo pipefail
cd "$(dirname "$0")/.."
OUT="${1:-db/sans_dict.dump}"
set -a; [[ -f .env.local ]] && . ./.env.local; set +a
[[ -n "${PG_DATABASE_URL:-}" ]] || { echo "PG_DATABASE_URL not set" >&2; exit 1; }
mkdir -p "$(dirname "$OUT")"
pg_dump -Fc -Z 9 --no-owner --no-acl -f "$OUT" "$PG_DATABASE_URL"
echo "wrote $OUT ($(du -h "$OUT" | cut -f1))"
