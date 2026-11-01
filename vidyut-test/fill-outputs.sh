#!/usr/bin/env bash
# Re-runs every example and rewrites the "// ==== SAMPLE OUTPUT ====" block at the end
# of each file with the real output (truncated to MAX lines).
#   ./fill-outputs.sh                 # all examples
#   ./fill-outputs.sh prakriya_vakya  # just one
set -euo pipefail
cd "$(dirname "$0")"
MAX=${MAX:-60}
MARK='// ==== SAMPLE OUTPUT ===='
cargo build -q --examples
for f in examples/*/*.rs; do
  name="$(basename "$(dirname "$f")")_$(basename "$f" .rs)"
  [[ $# -gt 0 && ! " $* " =~ " $name " ]] && continue
  out=$(cargo run -q --example "$name" 2>&1)
  total=$(printf '%s\n' "$out" | wc -l)
  body=$(printf '%s\n' "$out" | head -n "$MAX" | sed 's|^|// |; s| *$||')
  (( total > MAX )) && body+=$'\n'"// … ($((total - MAX)) more lines)"
  head -n "$(grep -n "^$MARK" "$f" | cut -d: -f1)" "$f" > "$f.tmp"
  printf '// $ cargo run --example %s\n%s\n' "$name" "$body" >> "$f.tmp"
  mv "$f.tmp" "$f"
  echo "filled $name ($total lines)"
done
