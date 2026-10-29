#!/usr/bin/env bash
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$DIR"
if [ "${1:-}" = --native ]; then shift; fi
cargo build --workspace "$@"
cargo run --bin controlab "$@"
