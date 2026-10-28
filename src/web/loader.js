const startupStartedAt = performance.now();
const canvas = document.querySelector("#gamevex");
const status = document.querySelector("#status");

function showError(title, detail) {
  status.hidden = false;
  status.classList.add("error");
  status.textContent = `${title}\n${detail}`;
}

function describeError(error) {
  return error instanceof Error ? `${error.message}${error.stack ? `\n\n${error.stack}` : ""}` : String(error);
}

canvas.dataset.lifecycleEpoch = "0";
function advanceLifecycleEpoch() {
  const current = Number.parseInt(canvas.dataset.lifecycleEpoch || "0", 10);
  canvas.dataset.lifecycleEpoch = String(Number.isFinite(current) ? current + 1 : 1);
}

// WASM animation frames can pause while hidden. This counter lets Rust notice the
// transition on the next frame, including a background/foreground cycle between ticks.
window.addEventListener("blur", advanceLifecycleEpoch);
window.addEventListener("focus", advanceLifecycleEpoch);
window.addEventListener("pagehide", advanceLifecycleEpoch);
window.addEventListener("pageshow", advanceLifecycleEpoch);
document.addEventListener("visibilitychange", advanceLifecycleEpoch);

// The Rust board publishes a visible rectangle in logical CSS-pixel coordinates
// as JSON in canvas.dataset.boardBounds, and enables it only during Minesweeper.
canvas.addEventListener("contextmenu", (event) => {
  if (canvas.dataset.boardActive !== "true" || !canvas.dataset.boardBounds) return;
  let bounds;
  try { bounds = JSON.parse(canvas.dataset.boardBounds); } catch { return; }
  if (!Array.isArray(bounds) || bounds.length !== 4 || !bounds.every(Number.isFinite)) return;
  const rect = canvas.getBoundingClientRect();
  if (!rect.width || !rect.height) return;
  const px = (event.clientX - rect.left) / rect.width * 2 - 1;
  const py = (event.clientY - rect.top) / rect.height * 2 - 1;
  const curvature = Number(canvas.dataset.curvature || 0);
  const warp = 1 + curvature * (px * px + py * py);
  const x = (px * warp + 1) * 0.5 * rect.width;
  const y = (py * warp + 1) * 0.5 * rect.height;
  const [left, top, width, height] = bounds;
  if (x >= left && x < left + width && y >= top && y < top + height) event.preventDefault();
});

// Report initialization-to-ready time, not first-pixel latency. Resource Timing's
// encodedBodySize is the transferred WASM body size when the browser exposes it.
function updateReadyMarkers() {
  if (canvas.dataset.ready !== "true") return;
  if (!canvas.dataset.startupMs) {
    canvas.dataset.startupMs = (performance.now() - startupStartedAt).toFixed(1);
    const wasmUrl = new URL("./pkg/gamevex_app_bg.wasm", import.meta.url).href;
    const resource = performance.getEntriesByName(wasmUrl, "resource").at(-1);
    if (resource?.encodedBodySize > 0) canvas.dataset.wasmTransfer = String(resource.encodedBodySize);
  }
  if (!status.classList.contains("error")) status.hidden = true;
}

// Keep the setup line out of the way once Bevy reports that the canvas is ready.
new MutationObserver(updateReadyMarkers).observe(canvas, { attributes: true, attributeFilter: ["data-ready"] });
updateReadyMarkers();

window.addEventListener("error", (event) => {
  if (status.classList.contains("error")) return;
  showError("The arcade encountered a browser error.", event.message || "An unknown JavaScript error occurred.");
});
window.addEventListener("unhandledrejection", (event) => {
  showError("The arcade could not start or continue.", describeError(event.reason));
});

async function start() {
  if (!window.isSecureContext) {
    throw new Error("WebGPU requires a secure context. Open this page from http://localhost or an HTTPS site.");
  }
  if (!navigator.gpu) {
    throw new Error("This browser does not expose WebGPU. Try a current browser with WebGPU enabled.");
  }
  const adapter = await navigator.gpu.requestAdapter();
  if (!adapter) throw new Error("WebGPU is available, but the browser could not find a compatible graphics adapter.");

  status.textContent = "Starting GameVex…";
  const { default: initialize } = await import("./pkg/gamevex_app.js");
  await initialize();
  status.textContent = "GameVex Rust Demo · WebGPU";
  canvas.focus({ preventScroll: true });
}

start().catch((error) => showError("GameVex could not initialize.", describeError(error)));

// The canvas contains the app's own focus order. Leave browser/OS shortcuts alone.
canvas.addEventListener("keydown", (event) => {
  if (event.metaKey || event.ctrlKey || event.altKey) return;
  if (["Tab", " ", "Enter", "ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"].includes(event.key)) {
    event.preventDefault();
  }
});
