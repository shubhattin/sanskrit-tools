#!/usr/bin/env bash
# Download the Cologne (CDSL) English→Sanskrit dictionaries into
# dict_data/cologne/, pinned to a fixed csl-orig commit and sha256-verified so
# imports are reproducible. Idempotent: verified files are not re-downloaded.
set -euo pipefail
cd "$(dirname "$0")/.."

COMMIT=e9411e0770e7ad872e6509c7b26ce7c348b860f3 # sanskrit-lexicon/csl-orig
DIR=dict_data/cologne
declare -A SHA256=(
  [ae]=515b9876a53b0feef618290e33d37adc82ec38148ebba2263f33ae18b47acab7
  [mwe]=2a2a5b9dd1e1b1c3594d26d916d466708a81435c2e2c173cce9d165934bbbcbd
  [bor]=8e55ff77cdd13dccb1338f578ade89bc5fb2bc07578ba5875c7bebfd10eb8372
)
mkdir -p "$DIR"
for d in ae mwe bor; do
  f="$DIR/$d.txt"
  if [[ -f "$f" ]] && echo "${SHA256[$d]}  $f" | sha256sum -c --quiet - 2>/dev/null; then
    continue
  fi
  echo "downloading $d"
  curl -sfL -o "$f.tmp" "https://raw.githubusercontent.com/sanskrit-lexicon/csl-orig/$COMMIT/v02/$d/$d.txt"
  echo "${SHA256[$d]}  $f.tmp" | sha256sum -c --quiet -
  mv "$f.tmp" "$f"
done
echo "cologne sources OK ($DIR)"
