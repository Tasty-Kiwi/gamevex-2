#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_DIR="$(cd -- "$SCRIPT_DIR/.." && pwd)"
cd "$WORKSPACE_DIR"
if [[ "$(uname -s)" != Darwin ]]; then
  echo "The native application targets macOS / Metal." >&2
  exit 1
fi
cargo build --locked -p gamevex-app --release
APP_DIR="$WORKSPACE_DIR/target/GameVex Rust.app"
mkdir -p "$APP_DIR/Contents/MacOS"
cp target/release/gamevex-app "$APP_DIR/Contents/MacOS/gamevex-app"
cat > "$APP_DIR/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>gamevex-app</string>
<key>CFBundleIdentifier</key><string>net.gamevex.rust-demo</string>
<key>CFBundleName</key><string>GameVex Rust</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>0.1.0</string>
<key>CFBundleVersion</key><string>1</string>
<key>NSHighResolutionCapable</key><true/>
</dict></plist>
PLIST
printf 'Native application: %s\n' "$APP_DIR"
