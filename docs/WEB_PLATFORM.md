# Web platform: automatic graphics fallback and installable PWA

## Renderer selection

`scripts/build-web.sh` builds two independent JS/WASM packages at `web/pkg/webgpu/` and `web/pkg/webgl2/`, then generates the backend descriptor and offline worker. Both run the same Rust game, renderer, input, audio, and save format. Native macOS still selects Metal.

The browser loader prefers WebGPU. It checks for the API and a compatible adapter, allowing ten seconds for adapter detection. Missing support, a rejected adapter request, a null adapter, or an adapter timeout selects WebGL2 directly. WebGL2 probes a disposable canvas so the application's canvas remains available to wgpu. Successful WebGPU startup does not import the WebGL2 package; the PWA separately downloads both for offline use.

Bevy 0.17.3 selects its browser backend at compile time: `webgpu` overrides `webgl2`. Independent packages are necessary for runtime fallback. See [the versioned Bevy examples guide](https://github.com/bevyengine/bevy/blob/v0.17.3/examples/README.md#webgl2-and-webgpu).

## Recovery after initialization starts

A partly initialized Bevy application can retain its GPU context, WASM state, event listeners, animation loop, audio context, and save lock. The loader does not initialize a second application in that document. Instead it uses `location.replace` for a single recovery navigation with a private WebGL2 marker. The next document consumes and removes that marker before starting WebGL2. It preserves the original query string, including bare flags and encoding, and the URL fragment. This does not require sessionStorage.

Recovery covers:

- JS module, WASM download, and initialization failures.
- Uncaught errors and rejected promises before readiness.
- A renderer startup that does not reach readiness within two minutes.
- Device creation failures, uncaptured GPU errors, and device loss after startup.
- Uncaught errors explicitly identifying WebGPU/wgpu after readiness. Unrelated application errors remain visible instead of being treated as graphics failures.

The loader observes the native adapter/device objects that wgpu obtains through `navigator.gpu.requestAdapter` and `adapter.requestDevice`, preserving native objects and method receivers. It attaches an `uncapturederror` listener and a handler for the actual device's `lost` promise. Devices may become unusable even after successful creation; see [MDN's GPUDevice.lost documentation](https://developer.mozilla.org/en-US/docs/Web/API/GPUDevice/lost).

Recovery unloads the original document, triggering the existing synchronous lifecycle save and releasing its Web Lock. The new application loads the persisted session and returns to the lobby. This is saved-game recovery, not an in-place renderer swap; unsaved transient Card Lab state does not survive. A later manual reload retries WebGPU. If WebGL2 is unavailable, loses its context, or fails to initialize, the loader shows a terminal error. It never automatically navigates from WebGL2 back to WebGPU.

## Renderer compatibility

`src/rendering.rs` batches triangle geometry, draws pooled text, renders into two `Rgba8UnormSrgb` targets, and composites them with a `Material2d` fragment shader. There is no custom compute pass or storage-buffer algorithm. `assets/crt.wgsl` uses texture sampling and aligned vector uniforms and compiles through Bevy/wgpu's WebGL2 path without a shader rewrite.

## PWA behavior

`web/manifest.webmanifest` supplies the stable relative app ID, standalone display mode, theme, start URL, scope, and regular/maskable icons. An Apple touch icon is linked from the shell. Installation uses the browser's own UI; see [MDN's installability guide](https://developer.mozilla.org/en-US/docs/Web/Progressive_web_apps/Guides/Making_PWAs_installable).

`scripts/generate-pwa.py` hashes the worker template and the exact runtime files, including both JS/WASM pairs. It emits an explicit cache allowlist. Fonts and the CRT shader are embedded in WASM; sounds are synthesized. Copied source assets and generated TypeScript declarations are not required at runtime.

The worker precaches both graphics backends, so a browser that loses WebGPU support can recover offline. This deliberately increases the first offline download. Offline readiness is published only after a complete cache has been installed. A failed download cannot replace the active worker. Updates wait for existing app windows to close, preventing mixed JS/WASM revisions. Activation removes only older caches for this registration's full scope URL, preserving other applications on the origin. Entry-page queries, including the recovery marker, share the cached shell while remaining visible to the loader. Saved sessions remain in localStorage.

Registration errors do not prevent online play. Offline play requires a successful first download and retained browser storage. HTTPS or localhost is required. The installed app still uses the browser's graphics implementation.

## Reproduction

```sh
# Normal WebGPU-first app with WebGL2 recovery and offline support.
scripts/build-web.sh
npm run test:graphics -- --workers=1
npm run test:pwa -- --workers=1
npm run test:browser:smoke -- --workers=1
GAMEVEX_TEST_GRAPHICS=webgl2 npm run test:browser -- tests/browser/gamevex.spec.js --workers=1

# Optional single-backend diagnostic builds; these disable automatic fallback.
GAMEVEX_WEB_BACKEND=webgpu scripts/build-web.sh
GAMEVEX_WEB_BACKEND=webgl2 scripts/build-web.sh

# Restore both packages.
scripts/build-web.sh
```

The normal CI web artifact includes both packages, the generated backend descriptor and worker, manifest, and icons. The automatic smoke suite covers missing-WebGPU fallback, recovery after a WebGPU initialization failure, and installability/offline saved-game recovery. The complete graphics and worker lifecycle suites run through their focused commands and the manual full browser workflow.

## Validation on 2026-11-03

Runtime checks use local Playwright Chromium on macOS with ANGLE/Metal. These do not establish Safari, Firefox, physical Android/iOS compatibility, or OS-level installation acceptance. No performance claim is made.

- The dual-backend release build passed.
- All 17 graphics regressions passed, including preferred WebGPU selection, missing/rejected/null/hung adapter handling, invalid modules, rejected initialization, asynchronous errors and rejected promises, stalled startup, failed WASM download, device creation rejection, and terminal handling when both APIs fail.
- The real-device cases call `GPUDevice.destroy()` and deliberately create an invalid GPU buffer after playing a Sudoku hint. Both recover to the actual WebGL2 WASM app, preserve the saved board, and reacquire save ownership. Test-only device references and failure injection live in the browser harness.
- All five PWA cases passed under a Pages-style `/gamevex-2/` subpath. Chromium reports no manifest/installability errors; a saved Sudoku game loads offline with WebGPU and then with WebGPU hidden, selecting the cached WebGL2 runtime.
- Worker lifecycle cases cover multiple open windows, scope-specific cleanup, failed update downloads, and registration failure without blocking play. An actual WebGPU device loss also recovers through the cached shell and WebGL2 package while offline.

- All 28 cases in the existing `gamevex.spec.js` suite passed with `navigator.gpu` hidden and the selected WebGL2 backend asserted. The same suite also passed with WebGPU preferred.
- All six deployment smoke cases passed, including both preflight selection and recovery after initialization fails.
- JavaScript and shell syntax checks, Rust formatting, and `git diff --check` passed. Both WASM packages are built with the locked dependencies.

Changes are local; no commit, push, or deployment has been performed.
