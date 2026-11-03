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

// Only the board and cards use secondary-click gameplay. Other canvas areas
// retain the browser menu. Bounds share the shader's curved surface coordinates.
canvas.addEventListener("contextmenu", (event) => {
  let bounds = [];
  try {
    if (canvas.dataset.boardActive === "true" && canvas.dataset.boardBounds) bounds.push(JSON.parse(canvas.dataset.boardBounds));
    if (canvas.dataset.cardBounds) bounds.push(...JSON.parse(canvas.dataset.cardBounds));
  } catch { return; }
  const rect = canvas.getBoundingClientRect();
  if (!rect.width || !rect.height) return;
  const px = (event.clientX - rect.left) / rect.width * 2 - 1;
  const py = (event.clientY - rect.top) / rect.height * 2 - 1;
  const curvature = Number(canvas.dataset.curvature || 0);
  const warp = 1 + curvature * (px * px + py * py);
  const x = (px * warp + 1) * 0.5 * rect.width;
  const y = (py * warp + 1) * 0.5 * rect.height;
  if (bounds.some((b) => Array.isArray(b) && b.length === 4 && b.every(Number.isFinite)
    && x >= b[0] && x < b[0] + b[2] && y >= b[1] && y < b[1] + b[3])) event.preventDefault();
});

// Keep native image dragging/selection from stealing a game drag. Winit owns
// pointer capture; focus explicitly because preventing pointerdown cancels it.
canvas.addEventListener("pointerdown", (event) => {
  if (event.button === 0) event.preventDefault();
  canvas.focus({ preventScroll: true });
});
canvas.addEventListener("dragstart", (event) => event.preventDefault());

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

// Hold the lock for this document's lifetime. A second tab can play in memory
// and must reload to become the writer, so it cannot promote a stale snapshot.
function acquireSaveWriter() {
  if (!navigator.locks) return Promise.resolve(false);
  const qa = location.search.slice(1).split("&").find((part) => part.startsWith("qa="))?.slice(3);
  const key = ["fresh", "corrupt", "unavailable"].includes(qa)
    ? "gamevex.rust-demo.qa.v1"
    : "gamevex.rust-demo.v1";
  return new Promise((resolve) => {
    navigator.locks.request(key, { ifAvailable: true }, (lock) => {
      resolve(Boolean(lock));
      if (lock) return new Promise(() => {});
    }).catch(() => resolve(false));
  });
}

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
  canvas.dataset.saveWriter = String(await acquireSaveWriter());
  await initialize();
  status.textContent = "GameVex Rust Demo · WebGPU";
  canvas.focus({ preventScroll: true });
}

start().catch((error) => showError("GameVex could not initialize.", describeError(error)));

// The canvas contains the app's own focus order. Leave browser/OS shortcuts alone.
canvas.addEventListener("keydown", (event) => {
  if (event.metaKey || event.ctrlKey || event.altKey) return;
  if (["Tab", " ", "Enter", "ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown", "PageUp", "PageDown", "Backspace"].includes(event.key)) {
    event.preventDefault();
  }
});
