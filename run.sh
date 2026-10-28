#!/usr/bin/env bash
set -e

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
cd "$DIR"

# Ativa o ambiente virtual Python se existir
if [ -d ".venv" ]; then
    source .venv/bin/activate
fi

case "${1:-}" in
    --web)
        python src/python/web_server.py
        ;;
    --mobile)
        python src/python/web_server.py --host 0.0.0.0
        ;;
    --cli)
        shift
        python src/python/routh_hurwitz.py "$@"
        ;;
    *)
        if [ "${1:-}" = "--native" ]; then shift; fi
        cargo build "$@" && cargo run --bin controlab "$@"
        ;;
esac
