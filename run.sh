#!/usr/bin/env bash
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$DIR"
if [ -d .venv ]; then
    source .venv/bin/activate
fi
case "${1:-}" in
    --cli)
        shift
        python src/python/routh_hurwitz.py "$@"
        ;;
    *)
        if [ "${1:-}" = --native ]; then shift; fi
        cargo build --workspace "$@"
        cargo run --bin controlab "$@"
        ;;
esac
