import { expect } from '@playwright/test';
import { readFile } from 'node:fs/promises';
import { test } from './web-fixture.js';
import { revealLobbyGame } from './lobby.js';
import { screenPoint } from './interaction.js';

async function offlineReady(page) {
  await expect(page.locator('html')).toHaveAttribute('data-offline-ready', 'true', { timeout: 30_000 });
  await expect.poll(() => page.evaluate(() => Boolean(navigator.serviceWorker.controller))).toBe(true);
}

async function clickBounds(page, bounds) {
  const point = await screenPoint(page, bounds[0] + bounds[2] / 2, bounds[1] + bounds[3] / 2);
  await page.mouse.click(point.x, point.y);
}

async function hideWebGPU(page) {
  await page.addInitScript(() => Object.defineProperty(navigator, 'gpu', { configurable: true, get: () => undefined }));
}

async function updateWorker(page) {
  await page.evaluate(async () => {
    const registration = await navigator.serviceWorker.getRegistration();
    await registration.update();
  });
}

test('install control appears only for an available browser prompt', async ({ page, site }) => {
  await hideWebGPU(page);
  await page.addInitScript(() => window.addEventListener('beforeinstallprompt', event => {
    if (event.isTrusted) { event.preventDefault(); event.stopImmediatePropagation(); }
  }));
  await page.goto(site.url + '?qa=fresh&diagnostics');
  const canvas = page.locator('#gamevex');
  await expect(canvas).toHaveAttribute('data-ready', 'true', { timeout: 30_000 });
  await expect.poll(() => canvas.evaluate(el => JSON.parse(el.dataset.actionBounds).install)).toBeNull();
  const offerInstall = () => page.evaluate(() => {
    const event = new Event('beforeinstallprompt', { cancelable: true });
    event.prompt = () => { window.installPromptCalls = (window.installPromptCalls || 0) + 1; return Promise.resolve(); };
    window.dispatchEvent(event);
    return event.defaultPrevented;
  });
  expect(await offerInstall()).toBe(true);
  await expect.poll(() => canvas.evaluate(el => JSON.parse(el.dataset.actionBounds).install)).not.toBeNull();
  await clickBounds(page, await canvas.evaluate(el => JSON.parse(el.dataset.actionBounds).install));
  await expect.poll(() => page.evaluate(() => window.installPromptCalls)).toBe(1);
  await expect.poll(() => canvas.evaluate(el => JSON.parse(el.dataset.actionBounds).install)).toBeNull();
  await offerInstall();
  await expect.poll(() => canvas.evaluate(el => JSON.parse(el.dataset.actionBounds).install)).not.toBeNull();
  await page.evaluate(() => window.dispatchEvent(new Event('appinstalled')));
  await expect.poll(() => canvas.evaluate(el => JSON.parse(el.dataset.actionBounds).install)).toBeNull();
});

test('PWA manifest is installable and the arcade restores a save offline under a subpath', { tag: '@smoke' }, async ({ page, context, site }) => {
  const backend = /"(webgpu|webgl2)"/.exec(await readFile('web/backend.js', 'utf8'))[1];
  if (backend === 'webgl2') await hideWebGPU(page);
  const errors = [];
  page.on('pageerror', error => errors.push(error.message));
  page.on('console', message => {
    if (message.type() === 'error') errors.push(message.text());
  });
  await page.goto(site.url + '?qa=fresh&diagnostics');
  await expect(page.locator('#gamevex')).toHaveAttribute('data-ready', 'true', { timeout: 30_000 });
  await offlineReady(page);
  await expect(page.locator('#gamevex')).toHaveAttribute('data-graphics-backend', backend);
  const cdp = await context.newCDPSession(page);
  const manifest = await cdp.send('Page.getAppManifest');
  expect(manifest.errors).toEqual([]);
  const metadata = JSON.parse(manifest.data);
  expect(new URL(metadata.start_url, manifest.url).href).toBe(site.url);
  expect(new URL(metadata.scope, manifest.url).href).toBe(site.url);
  expect(metadata.display).toBe('standalone');
  expect(metadata.icons.map(icon => icon.sizes)).toEqual(expect.arrayContaining(['192x192', '512x512']));
  const installability = await cdp.send('Page.getInstallabilityErrors');
  expect(installability.installabilityErrors).toEqual([]);

  await clickBounds(page, await revealLobbyGame(page, 'sudoku', bounds => clickBounds(page, bounds)));
  await expect(page.locator('#gamevex')).toHaveAttribute('data-interaction', /"screen":"Sudoku"/);
  const bounds = await page.locator('#gamevex').evaluate(el => JSON.parse(el.dataset.actionBounds).hint);
  await clickBounds(page, bounds);
  await expect.poll(() => page.locator('#gamevex').evaluate(el => JSON.parse(el.dataset.session).sudoku.game.assisted)).toBe(true);
  const saved = await page.locator('#gamevex').evaluate(el => {
    const game = JSON.parse(el.dataset.session).sudoku.game;
    return { board: game.board, notes: game.notes, assisted: game.assisted };
  });
  await context.setOffline(true);
  const response = await page.reload();
  expect(response.fromServiceWorker()).toBe(true);
  await expect(page.locator('#gamevex')).toHaveAttribute('data-ready', 'true', { timeout: 30_000 });
  expect(await page.locator('#gamevex').evaluate(el => {
    const game = JSON.parse(el.dataset.session).sudoku.game;
    return { board: game.board, notes: game.notes, assisted: game.assisted };
  })).toEqual(saved);
  await clickBounds(page, await revealLobbyGame(page, 'sudoku', bounds => clickBounds(page, bounds)));
  await expect(page.locator('#gamevex')).toHaveAttribute('data-interaction', /"screen":"Sudoku"/);
  // Both renderers must remain available when capabilities change offline.
  if ((await readFile('web/backend.js', 'utf8')).includes('"webgl2"')) {
    await hideWebGPU(page);
    await page.reload();
    await expect(page.locator('#gamevex')).toHaveAttribute('data-ready', 'true', { timeout: 30_000 });
    await expect(page.locator('#gamevex')).toHaveAttribute('data-graphics-backend', 'webgl2');
    expect(await page.locator('#gamevex').evaluate(el => {
      const game = JSON.parse(el.dataset.session).sudoku.game;
      return { board: game.board, notes: game.notes, assisted: game.assisted };
    })).toEqual(saved);
    await clickBounds(page, await revealLobbyGame(page, 'sudoku', bounds => clickBounds(page, bounds)));
    await expect(page.locator('#gamevex')).toHaveAttribute('data-interaction', /"screen":"Sudoku"/);
  }
  await page.screenshot({ path: test.info().outputPath("offline-sudoku.png") });
  expect(errors).toEqual([]);
});

test('an update waits for all app windows to close and preserves unrelated caches', async ({ page, context, site }) => {
  await hideWebGPU(page);
  await page.goto(site.url);
  await offlineReady(page);
  await page.evaluate(async () => {
    await caches.open('unrelated-app');
    await caches.open('gamevex:another-scope:old');
  });
  const second = await context.newPage();
  await hideWebGPU(second);
  await second.goto(site.url);
  await offlineReady(second);
  site.revision = 2;
  await updateWorker(page);
  await expect.poll(() => page.evaluate(async () => Boolean((await navigator.serviceWorker.getRegistration()).waiting))).toBe(true);
  await expect(page.locator('#update-notice')).toBeVisible();
  await expect(second.locator('#update-notice')).toBeVisible();
  expect(await page.evaluate(() => fetch('./pwa.js').then(r => r.text()))).toContain('// test revision 1');
  const fresh = await context.newPage();
  await page.close();
  expect(await second.evaluate(async () => Boolean((await navigator.serviceWorker.getRegistration()).waiting))).toBe(true);
  await second.close();
  await fresh.goto(site.url);
  await offlineReady(fresh);
  await expect(fresh.locator('#update-notice')).toBeHidden();
  await expect.poll(() => fresh.evaluate(() => fetch('./pwa.js').then(r => r.text()))).toContain('// test revision 2');
  const keys = await fresh.evaluate(() => caches.keys());
  expect(keys).toContain('unrelated-app');
  expect(keys).toContain('gamevex:another-scope:old');
  expect(keys.filter(key => key.startsWith(`gamevex:${site.url}:`))).toHaveLength(1);
});

test('an incomplete update leaves the installed offline version usable', async ({ page, context, site }) => {
  await hideWebGPU(page);
  await page.goto(site.url);
  await offlineReady(page);
  site.revision = 2;
  site.failWasm = true;
  await page.evaluate(async () => {
    const registration = await navigator.serviceWorker.getRegistration();
    await registration.update();
    const worker = registration.installing;
    if (worker && worker.state !== 'redundant') await new Promise(resolve => worker.addEventListener('statechange', () => {
      if (worker.state === 'redundant') resolve();
    }));
  });
  expect(site.failedWasmRequests).toBeGreaterThan(0);
  expect(await page.evaluate(async () => Boolean((await navigator.serviceWorker.getRegistration()).waiting))).toBe(false);
  await context.setOffline(true);
  await page.reload();
  expect(await page.evaluate(() => fetch('./pwa.js').then(r => r.text()))).toContain('// test revision 1');
  expect(await page.evaluate(() => fetch('./pkg/webgl2/gamevex_app_bg.wasm').then(r => r.ok))).toBe(true);
});

test('service-worker registration failure does not stop the arcade', async ({ page, site }) => {
  await page.addInitScript(() => {
    navigator.serviceWorker.register = () => Promise.reject(new Error('QA: registration blocked'));
  });
  await page.goto(site.url + '?qa=fresh&diagnostics');
  await expect(page.locator('html')).toHaveAttribute('data-offline-ready', 'false');
  await expect(page.locator('#gamevex')).toHaveAttribute('data-ready', 'true', { timeout: 30_000 });
  await expect(page.locator('#status')).toBeHidden();
});

test('a WebGPU device loss recovers through the cached shell while offline', async ({ page, context, site }) => {
  await page.addInitScript(() => {
    const requestDevice = GPUAdapter.prototype.requestDevice;
    GPUAdapter.prototype.requestDevice = async function (...args) {
      const device = await requestDevice.apply(this, args);
      window.offlineTestDevice = device;
      return device;
    };
  });
  await page.goto(site.url + '?qa=fresh&diagnostics#offline');
  await expect(page.locator('#gamevex')).toHaveAttribute('data-ready', 'true', { timeout: 30_000 });
  await expect(page.locator('#gamevex')).toHaveAttribute('data-graphics-backend', 'webgpu');
  await offlineReady(page);
  await context.setOffline(true);
  await page.evaluate(() => window.offlineTestDevice.destroy());
  await expect(page.locator('#gamevex')).toHaveAttribute('data-graphics-backend', 'webgl2');
  await expect(page.locator('#gamevex')).toHaveAttribute('data-ready', 'true', { timeout: 30_000 });
  await expect(page.locator('#status')).toBeHidden();
  expect(page.url()).toBe(site.url + '?qa=fresh&diagnostics#offline');
  await expect(page.locator('#gamevex')).toHaveAttribute('data-save-writer', 'true');
});
