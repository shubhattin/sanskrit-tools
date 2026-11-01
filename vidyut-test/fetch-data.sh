#!/usr/bin/env bash
# Downloads vidyut's pre-built linguistic data (kosha, cheda model, sandhi rules,
# chandas meters, dhatupatha …) into ./data. Safe to re-run (skips if present).
set -euo pipefail
cd "$(dirname "$0")"
VERSION=0.4.0
URL="https://github.com/ambuda-org/vidyut/releases/download/py-$VERSION/data-$VERSION.zip"
if [[ -f data/kosha/padas.fst && "${1:-}" != "--force" ]]; then
  echo "data/ already present (use --force to re-download)"; exit 0
fi
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
echo "Downloading $URL"
curl -fL --progress-bar -o "$tmp/data.zip" "$URL"
rm -rf data && mkdir data
unzip -q "$tmp/data.zip" -d data
echo "OK -> $(du -sh data | cut -f1) in ./data"
ls data
