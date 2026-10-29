#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_DIR="$(cd -- "$SCRIPT_DIR/.." && pwd)"
cd "$WORKSPACE_DIR"
cargo build --locked -p gamevex-app --release
"$SCRIPT_DIR/build-web.sh"
