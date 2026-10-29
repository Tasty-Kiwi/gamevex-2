#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_DIR="$(cd -- "$SCRIPT_DIR/.." && pwd)"
cd "$WORKSPACE_DIR"

WASM_TARGET="wasm32-unknown-unknown"
TARGET_LIBDIR="$(rustc --print target-libdir --target "$WASM_TARGET" 2>/dev/null || true)"
HAS_WASM_STD=false
if [[ -n "$TARGET_LIBDIR" && -d "$TARGET_LIBDIR" ]] && compgen -G "$TARGET_LIBDIR/libstd-*.rlib" >/dev/null; then
  HAS_WASM_STD=true
fi

BUILD_MODE=target
if [[ "$HAS_WASM_STD" != true ]]; then
  if command -v rustup >/dev/null 2>&1; then
    echo "Installing the wasm32-unknown-unknown target with rustup…"
    rustup target add "$WASM_TARGET"
    BUILD_MODE=target
  else
    SYSROOT="$(rustc --print sysroot)"
    RUST_SRC="$SYSROOT/lib/rustlib/src/rust/library"
    if [[ -d "$RUST_SRC" ]]; then
      echo "Using the installed Rust source tree to build the wasm standard library."
      BUILD_MODE=build-std
    else
      echo "Missing wasm32-unknown-unknown Rust standard library." >&2
      echo "With rustup: rustup target add wasm32-unknown-unknown" >&2
      echo "With a source-enabled Homebrew Rust toolchain: install rust-src and retry." >&2
      exit 1
    fi
  fi
fi

WASM_BINDGEN_CMD="${WASM_BINDGEN:-wasm-bindgen}"
if ! command -v "$WASM_BINDGEN_CMD" >/dev/null 2>&1; then
  echo "Missing wasm-bindgen CLI (looked for: $WASM_BINDGEN_CMD)." >&2
  echo "Install it with: cargo install wasm-bindgen-cli --version 0.2.129 --locked" >&2
  echo "This CLI version must match the wasm-bindgen crate version in Cargo.toml." >&2
  echo "Or set WASM_BINDGEN to the CLI executable path." >&2
  exit 1
fi

LINKER_VAR="CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER"
WASM_LINKER="${CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER:-}"
if [[ -n "$WASM_LINKER" ]]; then
  if ! command -v "$WASM_LINKER" >/dev/null 2>&1; then
    echo "$LINKER_VAR points to an unavailable linker: $WASM_LINKER" >&2
    exit 1
  fi
else
  WASM_LINKER="$(command -v wasm-ld || true)"
  if [[ -z "$WASM_LINKER" ]]; then
    SYSROOT="$(rustc --print sysroot)"
    HOST="$(rustc -vV | sed -n 's/^host: //p')"
    for candidate in \
      "$SYSROOT/lib/rustlib/$HOST/bin/gcc-ld/wasm-ld" \
      "$SYSROOT/lib/rustlib/$HOST/bin/wasm-ld" \
      "$SYSROOT/lib/rustlib/$HOST/bin/rust-lld" \
      "$SYSROOT/lib/rustlib/bin/rust-lld"; do
      if [[ -x "$candidate" ]]; then
        WASM_LINKER="$candidate"
        break
      fi
    done
  fi
  if [[ -z "$WASM_LINKER" ]]; then
    echo "Missing WebAssembly linker (wasm-ld or rust-lld)." >&2
    echo "Install LLVM lld, add wasm-ld to PATH, or set $LINKER_VAR to a linker executable." >&2
    exit 1
  fi
  export CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER="$WASM_LINKER"
fi
echo "Using WebAssembly linker: $WASM_LINKER"

if [[ "$BUILD_MODE" == build-std ]]; then
  RUSTC_BOOTSTRAP=1 cargo -Z build-std=std,panic_abort build --locked -p gamevex-app --release --target "$WASM_TARGET"
else
  cargo build --locked -p gamevex-app --release --target "$WASM_TARGET"
fi
mkdir -p web/pkg web/assets
"$WASM_BINDGEN_CMD" --target web --out-dir web/pkg --out-name gamevex_app \
  target/wasm32-unknown-unknown/release/gamevex-app.wasm
if [[ -d assets ]]; then
  cp -R assets/. web/assets/
fi
echo "Web build is ready in $WORKSPACE_DIR/web"
