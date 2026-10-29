# GameVex Rust Demo

A local-first Rust arcade prototype. The workspace lives here; the existing GameVex web application remains separate.

## Architecture

Cargo manifests, the lockfile, documentation, scripts, web files, and assets live at the repository root. `src/` contains only Rust source; the embedded CRT shader lives in `assets/crt.wgsl`. The small core crate manifest is in `crates/gamevex-core/`.

- `src/core/` contains the serializable Minesweeper game model, commands, cards, settings, records, and save validation. It does not depend on Bevy.
- The root `gamevex-app` package (`src/main.rs` and its sibling modules) owns Bevy rendering, input, audio, and platform integration. The ordinary Rust game state is authoritative; Bevy entities present that state.
- `web/` contains the browser entry point and generated WebAssembly package. `scripts/build-web.sh` copies `assets/` into `web/assets/` when that source directory exists.
- Native saves use the GameVex Rust Demo application-support namespace. Browser saves use their own namespaced local-storage key, separate from the existing TypeScript app.

The update flow is input → command → rules update → domain events → presentation. Pointer events retain their received order and per-event position, including fast presses/releases within a single frame. Presentation buffers update before Bevy lays out text and propagates transforms. Animations do not determine game results.

## Prerequisites

- Rust and Cargo.
- For the browser build, an LLD WebAssembly linker (`wasm-ld` or compatible `rust-lld`) and wasm-bindgen CLI 0.2.129, matching the `wasm-bindgen` crate in `gamevex-app`:

  ```sh
  cargo install wasm-bindgen-cli --version 0.2.129 --locked
  ```

  Install LLVM's linker with `brew install lld` on macOS or use your platform's LLVM package. `scripts/build-web.sh` finds `wasm-ld` on `PATH` or a Rust sysroot linker. If it is elsewhere, set `CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER` to its executable path.

  The script uses the installed `wasm32-unknown-unknown` standard library when present. With rustup, it installs that target when needed. A non-rustup toolchain can build the standard library from its installed Rust sources (`rust-src`) with Cargo's `build-std` fallback. Set `WASM_BINDGEN` to the CLI path if it is not on `PATH`.

- Python 3 for the local static server.

## Run

From this directory:

```sh
scripts/run-native.sh
scripts/build-web.sh
scripts/serve-web.sh
```

Then open `http://localhost:18435`. Set `GAMEVEX_PORT` to choose another port. Use an origin separate from the original GameVex app: its offline service worker can otherwise serve cached pages instead of this demo. The browser loader reports a visible setup error when the page is not in a secure context, WebGPU is unavailable, or no compatible adapter is found. `scripts/build-all.sh` builds the native release and browser release in sequence.

## Browser QA modes

Append one of these query strings to the local URL to exercise persistence behavior. All QA modes use the isolated `gamevex.rust-demo.qa.v1` storage key; regular play continues to use `gamevex.rust-demo.v1`.

- `?qa=fresh&diagnostics` starts from the isolated QA key without injecting a failure.
- `?qa=corrupt&diagnostics` seeds literal invalid JSON only when the QA key is missing, to exercise corrupt-save recovery.
- `?qa=unavailable&diagnostics` injects an explicit `QA: storage unavailable` result from the platform adapter for load and store. This is deliberate fault injection, not a test of browser permission denial.

With `diagnostics` present, the latest serialized save is mirrored into `#gamevex` as `data-session` for browser QA inspection. Without that flag, session contents are not copied to the DOM. The canvas also exposes `data-startup-ms` for time from loader evaluation to the app's canvas-ready marker (not first-pixel latency), and `data-wasm-transfer` when the browser provides the encoded WASM response size.

## Controls

On the Minesweeper board, arrow keys move focus, Space reveals, F toggles a flag, and Enter chords a revealed number. Right-click or secondary click flags a cell. Keep keyboard shortcuts scoped to the active board; Tab and Shift+Tab navigate controls outside it. The browser suppresses its context menu over the active board and Card Lab cards, where secondary click flags or flips. Other canvas areas retain the browser menu.

In Card Lab, use the visible on-screen controls and pointer interactions to select, move, flip, and reset the sample cards. Audio starts after a user gesture, as required by browser autoplay policy.

## Bevy references

- [Bevy 0.17.3 API documentation](https://docs.rs/bevy/0.17.3/bevy/)
- [Bevy 0.17.3 examples](https://github.com/bevyengine/bevy/tree/v0.17.3/examples)

## Implemented demo

The lobby opens a complete Minesweeper cabinet, Card Lab, and settings. Beginner is 9×9 with 10 mines, Intermediate 16×16 with 40, and Expert 30×16 with 99. Placement is seeded and deferred until the first reveal; that cell and its neighbors are protected. Reveals, floods, flags, chording, losses, wins, timers, and records live in `gamevex-core`. Restart repeats the seed; New field draws a new seed. Replacing an unfinished field requires an explicit choice.

Tab / Shift+Tab move between controls; Enter / Space activate them. Entering Minesweeper focuses the board. Arrow keys move its cell focus, Space reveals, F flags, and Enter chords. Clicking an open number also chords. Escape returns to the lobby, or closes a dialog/settings first. Expert retains 32-pixel cells: scroll to pan, Shift+scroll to pan horizontally, or move keyboard focus to pan into view.

The GameVex header logo returns to the lobby and supports pointer and keyboard activation. Settings scrolls with the wheel or Page Up / Page Down on short windows; its Back, Retry save, and Reset saved data controls stay visible.

Card Lab uses a repeatable five-card hand. Click to select; right-click to flip that card; drag to one of three empty slots. Invalid or occupied drops return to the committed position. Left/Right select a card, F flips it, 1/2/3 place it, and Backspace returns it to the hand. Deal again resets the sample. Losing focus, leaving the surface, or resizing cancels a drag. Hover lift retains the card’s resting footprint, and pointer clicks follow the painted stacking order. This is a sandbox without casino rules.

Vector CRT renders geometry at the display's pixel density. Pixel CRT renders that geometry at half logical resolution and scales with nearest-neighbor sampling. Both use the same logical layout and picking coordinates. A separate text layer stays at display resolution in both presets. Geometry and text pass through the same curved-monitor composite, and pointer picking follows the curve. One reusable mesh batches triangle strokes, filled rectangles, focus outlines, grids, and geometric card suits. An embedded fragment shader composites bounded glow, scanlines, adjustable screen curvature, an adjustable vignette, and radial chromatic aberration; disabling CRT bypasses these effects. Settings offers the original cyan scheme and a warm amber scheme. Chromatic aberration separates red/blue channels toward the screen edges, stays sharp in the center, and scales consistently with display density in both presets. Its strength is saved, defaults to 35%, and can be set to zero independently. The scene uses no custom compute pass. Reduced motion removes card movement/dealing transitions. Volume and mute control short synthesized action sounds; no audio assets are fetched.

## Saves and recovery

Native saves are atomically replaced at `~/Library/Application Support/GameVex Rust Demo/session-v1.json`. A temporary file is written and synced before rename; the parent directory is synced too. Set `GAMEVEX_SAVE_DIR` to isolate native test sessions. Browser saves use `gamevex.rust-demo.v1` in localStorage. Neither target reads or migrates the original GameVex saves.

Version 1 stores settings, the selected difficulty, the current board, status, accumulated milliseconds, and best times per difficulty. Loading checks size, version, settings ranges, cell counts, adjacency, first-reveal protection, terminal-state invariants, and timer bounds. Invalid saves are protected from automatic replacement. Settings always offers Reset saved data; its confirmation replaces settings, records, and game progress with defaults and returns to the lobby. Cancel preserves the saved session. Existing version-1 saves load the added monitor and palette settings with defaults. Storage failures keep the session playable in memory and show a persistent status message; Retry save attempts another write.

Meaningful actions save immediately, active games save at five-second intervals, and foreground/visibility/close transitions save too. Browser lifecycle listeners synchronously flush a staged domain snapshot on blur, pagehide, and visibilitychange. No wall-clock timestamp is used to add time after reopening: only active foreground play advances the timer. A process crash can still lose progress since the last successful write. Card Lab is deliberately transient.

## Native bundle and verification tools

`scripts/package-macos.sh` builds an unsigned local application at `target/GameVex Rust.app`; all fonts and shaders it needs are embedded. `scripts/run-native.sh` runs from the workspace so screenshots have a predictable destination. Native F12 captures the current screen under `docs/screenshots/`. F9 measures 300 actual frame intervals and logs mean/p95 time with adapter, display preset, effects, window size, and DPI. These developer keys do not change game results.

Run the checks from this directory:

```sh
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
scripts/build-all.sh
```

See [the verification report](docs/VERIFICATION.md) for measured results, actual runtime checks, screenshots, and remaining limits.

## Limits

This first demo targets macOS/Metal and desktop browsers/WebGPU. It has no WebGL fallback, accounts, networking, save migration, other games, or cloud storage. The browser build must be served from localhost or HTTPS and needs WebGPU. The canvas has keyboard controls and visible focus, but it does not expose individual board cells or all virtual controls as a screen-reader accessibility tree. This is not a WCAG-conformance claim. The local Mac bundle is unsigned and not notarized. Very short browser viewports scroll the minimum-height application surface; touch-only input is outside this desktop demo.
