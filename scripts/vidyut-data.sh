#!/usr/bin/env bash
# Rebuilds the zstd-compressed vidyut kosha + cheda data in src/vidyut/data/{kosha,cheda}
# from Ambuda's prebuilt release (the same files `include_bytes!` embeds; see src/vidyut/kosha.rs).
# Only needed when moving to a new vidyut data release. Requires curl, unzip, zstd.
set -euo pipefail
cd "$(dirname "$0")/.."
VERSION=0.4.0
URL="https://github.com/ambuda-org/vidyut/releases/download/py-$VERSION/data-$VERSION.zip"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

echo "Downloading $URL"
curl -fL --progress-bar -o "$tmp/data.zip" "$URL"
unzip -q "$tmp/data.zip" 'kosha/*' 'cheda/*' -d "$tmp/data"

out=src/vidyut/data
mkdir -p "$out/kosha" "$out/cheda"
for f in kosha/padas.fst kosha/registry.msgpack cheda/model.msgpack; do
  zstd -19 -q -f "$tmp/data/$f" -o "$out/$f.zst"
done
ls -la "$out/kosha" "$out/cheda"
