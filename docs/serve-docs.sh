#!/usr/bin/env bash
# Serve the IronRoot docsify site locally.
#
# Tries `docsify-cli` first (best DX: live reload, sidebar autoreload).
# Falls back to `python3 -m http.server`, then `python -m http.server`.

set -euo pipefail

DOCS_DIR="$(cd "$(dirname "$0")" && pwd)"
PORT="${PORT:-3000}"

cd "$DOCS_DIR"

echo "Serving $DOCS_DIR on http://localhost:$PORT"
echo

if command -v docsify >/dev/null 2>&1; then
    echo "Using docsify-cli."
    exec docsify serve "$DOCS_DIR" --port "$PORT"
fi

if command -v npx >/dev/null 2>&1; then
    # Try docsify-cli via npx without forcing an install if it's not cached.
    if npx --no-install docsify-cli --version >/dev/null 2>&1; then
        echo "Using docsify-cli via npx."
        exec npx --no-install docsify-cli serve "$DOCS_DIR" --port "$PORT"
    fi
fi

if command -v python3 >/dev/null 2>&1; then
    echo "docsify-cli not found; falling back to python3 -m http.server."
    echo "Tip: install live-reload with: npm i -g docsify-cli"
    exec python3 -m http.server "$PORT" --bind 127.0.0.1
fi

if command -v python >/dev/null 2>&1; then
    echo "docsify-cli not found; falling back to python -m http.server."
    exec python -m http.server "$PORT" --bind 127.0.0.1
fi

echo "ERROR: no server available." >&2
echo "Install either docsify-cli (npm i -g docsify-cli) or Python 3." >&2
exit 1
