#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

./scripts/sync-api.sh >/dev/null

dirty=0
if ! git diff --quiet -- frontend/src/lib/api/generated; then
  dirty=1
fi
if [ -n "$(git ls-files -o --exclude-standard --directory frontend/src/lib/api/generated)" ]; then
  dirty=1
fi
if [ "$dirty" -ne 0 ]; then
  echo "Generated API client is out of date. Run: bash scripts/sync-api.sh"
  git diff --stat -- frontend/src/lib/api/generated
  git status --porcelain -- frontend/src/lib/api/generated
  exit 1
fi

echo "Generated API client is up to date."
