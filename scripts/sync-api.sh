#!/usr/bin/env bash
set -euo pipefail

# Repo root (parent of scripts/)
cd "$(dirname "$0")/.."

CARGO_ARGS=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --release)
      CARGO_ARGS+=(--release)
      shift
      ;;
    *)
      echo "Unknown option: $1" >&2
      echo "Usage: $0 [--release]" >&2
      exit 1
      ;;
  esac
done

echo "Generating OpenAPI schema from Rust..."
cargo run --bin gen-openapi --quiet "${CARGO_ARGS[@]}"

echo "Generating TypeScript client..."
(cd frontend && bun run openapi-ts)

echo "Done. Generated client: frontend/src/lib/api/generated/"
