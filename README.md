# GameVex Rust Demo

A local-first Rust arcade prototype. The workspace lives here; the existing GameVex web application remains separate.

## Architecture

Cargo manifests, the lockfile, documentation, scripts, web files, and assets live at the repository root. `src/` contains only Rust source; the embedded CRT shader lives in `assets/crt.wgsl`. The small core crate manifest is in `crates/gamevex-core/`.

- `src/core/` contains the serializable Minesweeper, Blackjack, Baccarat, and Sudoku models, commands, a deterministic card shoe, settings, records, and save validation. It does not depend on Bevy.
- The root `gamevex-app` package (`src/main.rs` and its sibling modules) owns Bevy rendering, input, audio, and platform integration. The ordinary Rust game state is authoritative; Bevy entities present that state.
- `src/presentation/` separates the shared header, card drawing, lobby, settings, dialogs, Minesweeper, Blackjack, Baccarat, and Sudoku views.
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

The lobby opens complete Minesweeper, Blackjack, Baccarat, and Sudoku cabinets, Card Lab, and settings. Beginner is 9×9 with 10 mines, Intermediate 16×16 with 40, and Expert 30×16 with 99. Placement is seeded and deferred until the first reveal; that cell and its neighbors are protected. Reveals, floods, flags, chording, losses, wins, timers, and records live in `gamevex-core`. Restart repeats the seed; New field draws a new seed. Replacing an unfinished field requires an explicit choice.

Tab / Shift+Tab move between controls; Enter / Space activate them. Entering Minesweeper focuses the board. Arrow keys move its cell focus, Space reveals, F flags, and Enter chords. Clicking an open number also chords. Escape returns to the lobby, or closes a dialog/settings first. Expert retains 32-pixel cells: scroll to pan, Shift+scroll to pan horizontally, or move keyboard focus to pan into view.

The GameVex header logo returns to the lobby and supports pointer and keyboard activation. Settings scrolls with the wheel or Page Up / Page Down on short windows; its Back, Retry save, and Reset saved data controls stay visible.

Card Lab uses a repeatable five-card hand. Click to select; right-click to flip that card; drag to one of three empty slots. Invalid or occupied drops return to the committed position. Left/Right select a card, F flips it, 1/2/3 place it, and Backspace returns it to the hand. Deal again resets the sample. Losing focus, leaving the surface, or resizing cancels a drag. Hover lift retains the card’s resting footprint, and pointer clicks follow the painted stacking order. This is a sandbox without casino rules.

Blackjack starts with 1,000 local demo credits and supports one to five hands, adjustable bets, Hit, Stand, Double, a single split of equal ranks, Surrender, and Insurance. H/S/D/P/R trigger the corresponding actions; Y/N answer the insurance offer. Enter activates the focused control. The dealer stands on soft 17. The dealer total stays visible: it counts face-up cards while the hole card is hidden, then includes the revealed card and each subsequent draw. Naturals return 2.5× the stake, ordinary wins 2×, and pushes return the stake; combined returns round once to whole credits. Surrender refunds half the stake rounded down; insurance costs half the opening total rounded up and returns 3× its premium for a dealer natural. These rules preserve the original cabinet's behavior, including its lack of an early dealer-natural peek and its 21-versus-21 push rule. Split hands are ineligible for a natural payout. Betting automatically fits the available bankroll. Bankruptcy offers a new 1,000-credit session.

Cards deal in sequence and slide into place when drawn or split. The dealer hole card flips before dealer draws appear one at a time; results and payouts appear after the sequence, and the active hand gets a brief highlight. Controls wait for the animation to finish. Reduced motion shows every change immediately. Animations only affect presentation; the completed rules state saves immediately.

Active Blackjack rounds survive reloads, including the shoe, hidden dealer card, active hand, bets, and balance. Settings preserves the round; leaving through the lobby button, logo, or Escape asks whether to forfeit the current bets while keeping unspent credits. On narrow screens, a compact ledger keeps all hands visible beside the active hand's cards and controls. These are local simulated credits with no purchases or payouts.

Baccarat uses a separate 1,000-credit bankroll. Choose Player, Banker, or Tie with the buttons or P/B/T; adjust the wager with -10/+10 or MAX, then Deal with D. N starts the next round; R restarts a bankrupt session. All draws resolve automatically. Aces count as 1, faces and tens as 0, and totals use the last digit. Natural 8 or 9 ends the deal immediately; otherwise Player draws on 0–5 and Banker follows the standard third-card tableau. Both visible totals update as the cards arrive.

Player wins return 2× the stake, Banker wins return 1.95× rounded to whole credits, and Tie wins return 9×. A tie pushes Player/Banker bets and returns their stake. Insufficient wagers remain disabled; MAX can use a remaining bankroll below 10 credits. The next round clears the chosen side while preserving the wager and balance. Cards animate in Player/Banker order, then reveal automatic third cards and the result. The resolved round, shoe, and bankroll save immediately, so leaving or reloading during the animation preserves the completed outcome. Reduced motion bypasses the sequence.

Sudoku supports 4x4 (2x2 boxes), 6x6 (2x3), 9x9 (3x3), and experimental 16x16 (4x4) boards. Easy, Medium, Hard, and Expert set clue-removal density rather than a human solving rating. Seeded generation uses a bounded uniqueness search through 9x9; experimental 16x16 may have multiple solutions. Completion accepts any filled grid that satisfies the rules.

Click a cell, then use the value bank or keyboard (1-9; A-G for values 10-16). Arrow keys move the focused cell; Delete/Backspace erases. P toggles pencil mode, U undoes, and H applies a hint while the board has keyboard focus. Tab reaches the value bank, mode controls, error/same-value highlights, and New board. Large boards retain 32-pixel cells: wheel to pan, Shift+wheel to pan horizontally, or move keyboard focus to bring a cell into view. The smaller count beside each symbol in the value bank shows how many remain. Given clues are locked; conflicting player entries remain editable and optionally highlight red.

Hints apply a forced naked or hidden single and explain the deduction. They reject conflicts and impossible cells; experimental 16x16 has no hints. Assistance stays set after undo and reload, and assisted runs cannot set best times. Up to 100 undo snapshots restore entries and notes during the current cabinet visit; settings preserves that history, while leaving the cabinet clears it. The board, notes, assistance, elapsed time, highlight preferences, and separate best times for all 16 modes persist. Time advances only during foreground play, pauses in settings/dialogs or outside the cabinet, and stops at completion. Changing size/difficulty or requesting a new puzzle confirms replacement when entries or notes would be lost. Ticket/shop progression remains outside this Rust demo.

Both casino cabinets show persistent outcome panels with the round’s net credit change. Wins send out green sparks and rings; natural Blackjack and winning Tie bets use a larger gold celebration. Losses pulse red, pushes use a cyan diamond, and an empty bankroll gets its own result. Individual hands and Baccarat lanes show their outcomes, while action badges call out Double, Split, Insurance, Surrender, and Bust. Transient effects finish within two seconds; reduced motion keeps the static outcome colors and labels. Restoring a completed round shows its result without replaying celebrations or sounds.

Sixteen synthesized casino cues cover chips, dealing, arriving cards, the dealer reveal, player actions, busts, wins, naturals, pushes, losses, and bankruptcy. Card and result sounds follow the visible animation. Cached audio sources and a four-voice limit keep playback bounded; result sounds take priority over card sounds. Mute and volume update active playback immediately. Browser playback uses one gesture-unlocked Web Audio context; native playback uses Bevy audio. No audio assets are fetched. A short [sound preview](docs/audio/casino-preview.wav) contains Bet, Card, Double, Win, Natural, Push, Loss, and Bust in that order.

Vector CRT renders geometry at the display's pixel density. Pixel CRT renders that geometry at half logical resolution and scales with nearest-neighbor sampling. Both use the same logical layout and picking coordinates. A separate text layer stays at display resolution in both presets. Geometry and text pass through the same curved-monitor composite, and pointer picking follows the curve. One reusable mesh batches triangle strokes, filled rectangles, focus outlines, grids, and geometric card suits. An embedded fragment shader composites bounded glow, scanlines, adjustable screen curvature, an adjustable vignette, and radial chromatic aberration; disabling CRT bypasses these effects. Settings offers the original cyan scheme and a warm amber scheme. Chromatic aberration separates red/blue channels toward the screen edges, stays sharp in the center, and scales consistently with display density in both presets. Its strength is saved, defaults to 35%, and can be set to zero independently. The scene uses no custom compute pass. Reduced motion removes card movement, dealing transitions, and animated outcome effects.

## Saves and recovery

Native saves are atomically replaced at `~/Library/Application Support/GameVex Rust Demo/session-v1.json`. A temporary file is written and synced before rename; the parent directory is synced too. Set `GAMEVEX_SAVE_DIR` to isolate native test sessions. Browser saves use `gamevex.rust-demo.v1` in localStorage. Neither target reads or migrates the original GameVex saves.

Version 4 stores shared settings plus independent Minesweeper, Blackjack, Baccarat, and Sudoku sessions. Existing Rust-demo version-1, version-2, and version-3 saves migrate automatically, preserving settings, boards, records, casino rounds, and balances while initializing missing cabinets. The storage key and filename retain their original `v1` suffix so existing saves can be found. Loading checks size, version, settings ranges, cell counts, adjacency, first-reveal protection, terminal-state invariants, and timer bounds. Invalid saves are protected from automatic replacement. Settings always offers Reset saved data; its confirmation replaces settings, records, all four games, and both casino bankrolls with defaults and returns to the lobby. Cancel preserves the saved session. Older saves load added monitor and palette settings with defaults. Storage failures keep the session playable in memory and show a persistent status message; Retry save attempts another write.

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

The browser regression harness requires Node.js and the built web package:

```sh
npm ci
npx playwright install chromium
npm run test:browser
```

Thirteen Chromium specs cover ordered card clicks, secondary flips, slot placement/rejection, drag cancellation on resize, Expert horizontal pan, and Blackjack/Baccarat round persistence and settlement, browser audio startup/cache/voice release, and Sudoku editing, hints, restoration, mode changes, hex input, and responsive bounds. CI validates Rust on macOS, builds and uploads both targets, then runs the browser regressions on Ubuntu with software WebGPU. Diagnostics additionally expose logical hit bounds and current interaction state when explicitly enabled.

See [the verification report](docs/VERIFICATION.md) for measured results, actual runtime checks, screenshots, and remaining limits.

## Limits

This first demo targets macOS/Metal and desktop browsers/WebGPU. It has no WebGL fallback, accounts, networking, migration from the original TypeScript application, other games, offline service worker, or cloud storage. Blackjack/Baccarat cheatsheets, Baccarat side bets, Mahjong, and the remaining cabinets have not been ported. The browser build must be served from localhost or HTTPS and needs WebGPU. The canvas has keyboard controls and visible focus, but it does not expose individual board cells or all virtual controls as a screen-reader accessibility tree. This is not a WCAG-conformance claim. The local Mac bundle is unsigned and not notarized. Very short browser viewports scroll the minimum-height application surface; touch-only input is outside this desktop demo.
