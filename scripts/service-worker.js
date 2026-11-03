// The build replaces these tokens with a content hash and the complete runtime.
const VERSION = __VERSION__;
const ASSETS = __ASSETS__;
const CACHE_PREFIX = `gamevex:${self.registration.scope}:`;
const CACHE_NAME = CACHE_PREFIX + VERSION;
const urls = ASSETS.map((path) => new URL(path, self.registration.scope).href);
const runtimeUrls = new Set(urls);
const indexUrl = new URL("./index.html", self.registration.scope).href;
const rootUrl = new URL("./", self.registration.scope).href;

self.addEventListener("install", (event) => {
  // Failure leaves the previously installed version intact. Do not skipWaiting:
  // an update must not swap JS/WASM underneath a running game.
  event.waitUntil((async () => {
    const cache = await caches.open(CACHE_NAME);
    await cache.addAll(urls.map((url) => new Request(url, { cache: "reload" })));
  })());
});

self.addEventListener("activate", (event) => {
  event.waitUntil((async () => {
    for (const key of await caches.keys()) {
      if (key.startsWith(CACHE_PREFIX) && key !== CACHE_NAME) await caches.delete(key);
    }
    await self.clients.claim();
  })());
});

self.addEventListener("fetch", (event) => {
  if (event.request.method !== "GET") return;
  const url = new URL(event.request.url);
  const plainUrl = url.origin + url.pathname;
  // Query strings (including QA modes) share the shell, never the saved state.
  // Only the entry page and the explicit runtime list belong to this worker.
  const key = event.request.mode === "navigate" && (plainUrl === rootUrl || plainUrl === indexUrl)
    ? indexUrl
    : runtimeUrls.has(plainUrl) ? plainUrl : null;
  if (!key) return;
  event.respondWith((async () => {
    const cache = await caches.open(CACHE_NAME);
    const cached = await cache.match(key);
    return cached || fetch(event.request);
  })());
});
