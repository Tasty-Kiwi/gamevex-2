// Installation uses the browser's own Install / Add to Home Screen UI.
// Register separately from renderer startup: cache errors must not stop play.
if (window.isSecureContext && "serviceWorker" in navigator) {
  window.addEventListener("load", async () => {
    try {
      await navigator.serviceWorker.register(new URL("./sw.js", import.meta.url), {
        scope: "./",
        updateViaCache: "none",
      });
      await navigator.serviceWorker.ready;
      document.documentElement.dataset.offlineReady = "true";
    } catch (error) {
      document.documentElement.dataset.offlineReady = "false";
      console.warn("GameVex offline caching is unavailable:", error);
    }
  }, { once: true });
}
