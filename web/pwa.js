let installPrompt = null;
let installed = window.matchMedia("(display-mode: standalone)").matches || navigator.standalone === true;

window.addEventListener("beforeinstallprompt", (event) => {
  event.preventDefault();
  installPrompt = event;
});
window.addEventListener("appinstalled", () => { installPrompt = null; installed = true; });

window.gamevexInstallAvailable = () => !installed && installPrompt !== null;
window.gamevexInstall = () => {
  const prompt = installPrompt;
  if (!prompt || installed) return;
  installPrompt = null;
  // The event is single-use, even when the user dismisses the dialog.
  try {
    Promise.resolve(prompt.prompt()).catch((error) => {
      console.warn("GameVex installation prompt failed:", error);
      document.querySelector("#install-help").showModal();
    });
  } catch (error) {
    console.warn("GameVex installation prompt failed:", error);
    document.querySelector("#install-help").showModal();
  }
};

// Register separately from renderer startup: cache errors must not stop play.
if (window.isSecureContext && "serviceWorker" in navigator) {
  window.addEventListener("load", async () => {
    try {
      const registration = await navigator.serviceWorker.register(new URL("./sw.js", import.meta.url), {
        scope: "./",
        updateViaCache: "none",
      });
      const updateNotice = document.querySelector("#update-notice");
      const showWaiting = () => {
        if (registration.waiting && navigator.serviceWorker.controller) updateNotice.hidden = false;
      };
      const watchInstalling = (worker) => {
        if (!worker) return;
        worker.addEventListener("statechange", showWaiting);
      };
      registration.addEventListener("updatefound", () => watchInstalling(registration.installing));
      watchInstalling(registration.installing);
      showWaiting();
      updateNotice.querySelector("button").addEventListener("click", () => { updateNotice.hidden = true; });
      const checkForUpdate = () => registration.update().catch(() => {});
      document.addEventListener("visibilitychange", () => {
        if (document.visibilityState === "visible") checkForUpdate();
      });
      setInterval(() => { if (document.visibilityState === "visible") checkForUpdate(); }, 60 * 60 * 1000);
      await navigator.serviceWorker.ready;
      document.documentElement.dataset.offlineReady = "true";
    } catch (error) {
      document.documentElement.dataset.offlineReady = "false";
      console.warn("GameVex offline caching is unavailable:", error);
    }
  }, { once: true });
}
