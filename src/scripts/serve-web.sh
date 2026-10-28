#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_DIR="$(cd -- "$SCRIPT_DIR/.." && pwd)"
WEB_DIR="$WORKSPACE_DIR/web"
if [[ ! -f "$WEB_DIR/pkg/gamevex_app.js" ]]; then
  echo "Web artifacts are missing. Run scripts/build-web.sh first." >&2
  exit 1
fi
cd "$WEB_DIR"
echo "Serving GameVex at http://localhost:8000 (Ctrl-C to stop)"
exec python3 -m http.server 8000 --bind 127.0.0.1
