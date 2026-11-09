import { graphicsBackends } from "./backend.js";
import "./pwa.js";

const startupStartedAt = performance.now();
const canvas = document.querySelector("#gamevex");
const status = document.querySelector("#status");
const statusMessage = document.querySelector("#status-message");
const progress = status.querySelector('[role="progressbar"]');
const recoveryParameter = "__gamevex_fallback";
const recoveryReasons = new Set(["adapter-unavailable", "device-unavailable", "device-lost", "gpu-error", "startup-failed", "startup-timeout"]);
const entryUrl = new URL(location.href);
const recoveryReason = entryUrl.searchParams.get(recoveryParameter);
const recovering = recoveryReasons.has(recoveryReason) && graphicsBackends.includes("webgl2");
if (entryUrl.searchParams.has(recoveryParameter)) {
  // Preserve bare flags and their encoding: Rust recognizes `diagnostics`,
  // while URLSearchParams serialization would change it to `diagnostics=`.
  entryUrl.search = entryUrl.search.slice(1).split("&")
    .filter((part) => !new URLSearchParams(part).has(recoveryParameter)).join("&");
  history.replaceState(history.state, "", entryUrl);
}
let graphicsBackend = recovering ? "webgl2" : graphicsBackends[0];
let webgpuStarted = false;
let transitioning = false;
let startupTimer;
if (recovering) canvas.dataset.fallbackReason = recoveryReason;

function showError(title, detail) {
  status.hidden = false;
  status.classList.add("error");
  progress.hidden = true;
  status.querySelector(".loading-heading").textContent = "Error";
  statusMessage.textContent = `${title}\n${detail}`;
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
  if (canvas.dataset.ready !== "true" || transitioning) return;
  clearTimeout(startupTimer);
  if (!canvas.dataset.startupMs) {
    canvas.dataset.startupMs = (performance.now() - startupStartedAt).toFixed(1);
    const wasmUrl = new URL(`./pkg/${graphicsBackend}/gamevex_app_bg.wasm`, import.meta.url).href;
    const resource = performance.getEntriesByName(wasmUrl, "resource").at(-1);
    if (resource?.encodedBodySize > 0) canvas.dataset.wasmTransfer = String(resource.encodedBodySize);
  }
  if (!status.classList.contains("error")) status.hidden = true;
}

// Keep the setup line out of the way once Bevy reports that the canvas is ready.
new MutationObserver(updateReadyMarkers).observe(canvas, { attributes: true, attributeFilter: ["data-ready"] });
updateReadyMarkers();

// A failed Bevy startup can leave event loops, GPU contexts, and save locks alive.
// After WASM has begun, recover in a fresh document instead of mixing two apps.
function failGraphics(error, reason = "startup-failed") {
  if (transitioning) return;
  clearTimeout(startupTimer);
  if (graphicsBackend === "webgpu" && graphicsBackends.includes("webgl2")) {
    console.warn("WebGPU failed; starting GameVex with WebGL2.", error);
    status.hidden = false;
    statusMessage.textContent = "Switching to compatible graphics…";
    if (webgpuStarted) {
      transitioning = true;
      const retryUrl = new URL(location.href);
      retryUrl.search += `${retryUrl.search ? "&" : "?"}${recoveryParameter}=${reason}`;
      location.replace(retryUrl);
    } else {
      graphicsBackend = "webgl2";
      canvas.dataset.fallbackReason = reason;
      startBackend().catch((failure) => failGraphics(failure));
    }
    return;
  }
  transitioning = true;
  showError("GameVex could not initialize its graphics.",
    `${canvas.dataset.fallbackReason ? "WebGPU failed, and the WebGL2 fallback could not continue.\n" : ""}${describeError(error)}`);
}

function isGraphicsError(error) {
  return /webgpu|wgpu|gpudevice|gpuvalidation|gpuinternal|device.?lost/i.test(describeError(error));
}

window.addEventListener("error", (event) => {
  if (transitioning) { event.preventDefault(); return; }
  if (canvas.dataset.ready !== "true" || isGraphicsError(event.error || event.message)) {
    event.preventDefault();
    failGraphics(event.error || new Error(event.message), "gpu-error");
  } else {
    showError("The arcade encountered a browser error.", event.message || "An unknown JavaScript error occurred.");
  }
});
window.addEventListener("unhandledrejection", (event) => {
  if (transitioning) { event.preventDefault(); return; }
  if (canvas.dataset.ready !== "true" || isGraphicsError(event.reason)) {
    event.preventDefault();
    failGraphics(event.reason, "gpu-error");
  } else {
    showError("The arcade could not start or continue.", describeError(event.reason));
  }
});

// Observe the real devices requested by wgpu, including loss after startup.
// Keep the native adapter/device objects and preserve their method receivers.
function observeWebGPUDevices(gpu) {
  const requestAdapter = gpu.requestAdapter;
  gpu.requestAdapter = async function (...args) {
    const adapter = await requestAdapter.apply(this, args);
    if (!adapter) return adapter;
    const requestDevice = adapter.requestDevice;
    adapter.requestDevice = async function (...deviceArgs) {
      try {
        const device = await requestDevice.apply(this, deviceArgs);
        device.addEventListener("uncapturederror", (event) => {
          event.preventDefault();
          failGraphics(event.error, "gpu-error");
        });
        device.lost.then((info) => failGraphics(new Error(`WebGPU device lost: ${info.message}`), "device-lost"))
          .catch((error) => failGraphics(error, "device-lost"));
        return device;
      } catch (error) {
        failGraphics(error, "device-unavailable");
        throw error;
      }
    };
    return adapter;
  };
}

async function withTimeout(promise, milliseconds, message) {
  let timer;
  try {
    return await Promise.race([promise, new Promise((_, reject) => {
      timer = setTimeout(() => reject(new Error(message)), milliseconds);
    })]);
  } finally {
    clearTimeout(timer);
  }
}

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

async function startBackend() {
  canvas.dataset.graphicsBackend = graphicsBackend;
  if (graphicsBackend === "webgl2") {
    // Probe a separate canvas so the game's canvas remains available to wgpu.
    const probe = document.createElement("canvas");
    const gl = probe.getContext("webgl2");
    if (!gl) throw new Error("This browser could not create a WebGL2 graphics context.");
    gl.getExtension("WEBGL_lose_context")?.loseContext();
    canvas.addEventListener("webglcontextlost", (event) => {
      event.preventDefault();
      failGraphics(new Error("The WebGL2 graphics context was lost. Reload to try again."));
    });
  } else {
    try {
      if (!navigator.gpu) throw new Error("This browser does not expose WebGPU.");
      const adapter = await withTimeout(navigator.gpu.requestAdapter(), 10_000, "The WebGPU adapter did not respond.");
      if (!adapter) throw new Error("WebGPU could not find a compatible graphics adapter.");
    } catch (error) {
      failGraphics(error, "adapter-unavailable");
      return;
    }
    observeWebGPUDevices(navigator.gpu);
    webgpuStarted = true;
  }
  statusMessage.textContent = `Starting GameVex · ${graphicsBackend === "webgl2" ? "WebGL2" : "WebGPU"}…`;
  startupTimer = setTimeout(() => failGraphics(new Error("The graphics renderer did not finish starting."), "startup-timeout"), 120_000);
  const { default: initialize } = await import(`./pkg/${graphicsBackend}/gamevex_app.js`);
  canvas.dataset.saveWriter = String(await acquireSaveWriter());
  await initialize();
  canvas.focus({ preventScroll: true });
}

if (window.isSecureContext) {
  startBackend().catch((error) => failGraphics(error));
} else {
  showError("GameVex could not initialize.", "Open this page from http://localhost or an HTTPS site.");
}

// The canvas contains the app's own focus order. Leave browser/OS shortcuts alone.
canvas.addEventListener("keydown", (event) => {
  if (event.metaKey || event.ctrlKey || event.altKey) return;
  if (["Tab", " ", "Enter", "ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown", "PageUp", "PageDown", "Backspace"].includes(event.key)) {
    event.preventDefault();
  }
});
