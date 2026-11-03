import { expect } from '@playwright/test';
import { test } from './web-fixture.js';
import { revealLobbyGame } from './lobby.js';
import { screenPoint } from './interaction.js';

// Keep package requests attributable to the loader in these cases. The PWA
// suite separately verifies offline recovery with both real packages cached.
test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    navigator.serviceWorker.register = () => Promise.reject(new Error('QA: no precache'));
  });
});

async function ready(page, backend) {
  await expect(page.locator('#gamevex')).toHaveAttribute('data-graphics-backend', backend);
  await expect(page.locator('#gamevex')).toHaveAttribute('data-ready', 'true', { timeout: 30_000 });
  await expect(page.locator('#status')).toBeHidden();
}

async function clickBounds(page, bounds) {
  const point = await screenPoint(page, bounds[0] + bounds[2] / 2, bounds[1] + bounds[3] / 2);
  await page.mouse.click(point.x, point.y);
}

test('WebGPU is preferred and WebGL2 is not imported on success', async ({ page, site }) => {
  await page.goto(site.url + '?qa=fresh&diagnostics');
  await ready(page, 'webgpu');
  expect(site.requests['pkg/webgpu/gamevex_app_bg.wasm']).toBeGreaterThan(0);
  expect(site.requests['pkg/webgl2/gamevex_app.js']).toBeUndefined();
  expect(site.requests['index.html']).toBe(1);
});

for (const mode of ['missing', 'no-adapter', 'adapter-rejected', 'adapter-hung']) {
  test(`WebGPU ${mode} selects a playable WebGL2 game`, mode === 'missing' ? { tag: '@smoke' } : {}, async ({ page, site }) => {
    await page.addInitScript(mode => {
      const requestAdapter = mode === 'no-adapter' ? async () => null
        : mode === 'adapter-rejected' ? async () => { throw new Error('QA: adapter refused'); }
        : () => new Promise(() => {});
      Object.defineProperty(navigator, 'gpu', { value: mode === 'missing' ? undefined : { requestAdapter } });
      if (mode === 'adapter-hung') {
        const timeout = window.setTimeout;
        window.setTimeout = (fn, delay, ...args) => timeout(fn, delay === 10_000 ? 50 : delay, ...args);
      }
    }, mode);
    await page.goto(site.url + '?qa=fresh&diagnostics#arcade');
    await ready(page, 'webgl2');
    expect(site.requests['pkg/webgpu/gamevex_app.js']).toBeUndefined();
    expect(site.requests['index.html']).toBe(1);
    await clickBounds(page, await revealLobbyGame(page, 'cards', bounds => clickBounds(page, bounds)));
    await expect(page.locator('#gamevex')).toHaveAttribute('data-interaction', /"screen":"Cards"/);
  });
}

for (const [failure, module] of [
  ['initialization-rejected', 'export default async () => { throw new Error("QA: WebGPU initialization failed"); };'],
  ['asynchronous-error', 'export default () => { setTimeout(() => { throw new Error("QA: WebGPU initialization failed"); }, 0); };'],
  ['asynchronous-rejection', 'export default () => { setTimeout(() => Promise.reject(new Error("QA: WebGPU initialization failed")), 0); };'],
  ['startup-hung', 'export default async () => {};'],
  ['module-invalid', 'this is not a JavaScript module'],
]) {
  test(`WebGPU ${failure} restarts once with a clean WebGL2 runtime`, failure === 'initialization-rejected' ? { tag: '@smoke' } : {}, async ({ page, site }) => {
    site.webgpuModule = module;
    if (failure === 'startup-hung') await page.addInitScript(() => {
      if (location.search.includes('__gamevex_fallback')) return;
      const timeout = window.setTimeout;
      window.setTimeout = (fn, delay, ...args) => timeout(fn, delay === 120_000 ? 50 : delay, ...args);
    });
    const originalUrl = site.url + '?qa=fresh&diagnostics&encoded=%2f%20&flag#arcade';
    await page.goto(originalUrl);
    await ready(page, 'webgl2');
    expect(page.url()).toBe(originalUrl);
    expect(site.requests['index.html']).toBe(2);
    expect(site.requests['pkg/webgpu/gamevex_app.js']).toBe(1);
    expect(site.requests['pkg/webgl2/gamevex_app.js']).toBe(1);
    await expect(page.locator('#gamevex')).toHaveAttribute('data-save-writer', 'true');
  });
}

test('a failed WebGPU WASM download restarts with WebGL2', async ({ page, site }) => {
  site.failWebGPUWasm = true;
  await page.goto(site.url + '?qa=fresh&diagnostics');
  await ready(page, 'webgl2');
  expect(site.failedWasmRequests).toBeGreaterThan(0);
  expect(site.requests['index.html']).toBe(2);
});

test('WebGPU device creation failure restarts with WebGL2', async ({ page, site }) => {
  await page.addInitScript(() => {
    GPUAdapter.prototype.requestDevice = async () => { throw new Error('QA: WebGPU device refused'); };
  });
  await page.goto(site.url + '?qa=fresh&diagnostics');
  await ready(page, 'webgl2');
  expect(site.requests['index.html']).toBe(2);
});

for (const failure of ['lost-device', 'validation-error']) {
  test(`a real WebGPU ${failure} falls back and preserves the saved game`, async ({ page, site }) => {
    await page.addInitScript(() => {
      const requestDevice = GPUAdapter.prototype.requestDevice;
      GPUAdapter.prototype.requestDevice = async function (...args) {
        const device = await requestDevice.apply(this, args);
        window.testGPUDevice = device;
        return device;
      };
    });
    await page.goto(site.url + '?qa=fresh&diagnostics');
    await ready(page, 'webgpu');
    await clickBounds(page, await revealLobbyGame(page, 'sudoku', bounds => clickBounds(page, bounds)));
    await expect(page.locator('#gamevex')).toHaveAttribute('data-interaction', /"screen":"Sudoku"/);
    const hint = await page.locator('#gamevex').evaluate(el => JSON.parse(el.dataset.actionBounds).hint);
    await clickBounds(page, hint);
    await expect.poll(() => page.locator('#gamevex').evaluate(el => JSON.parse(el.dataset.session).sudoku.game.assisted)).toBe(true);
    const board = await page.locator('#gamevex').evaluate(el => JSON.parse(el.dataset.session).sudoku.game.board);
    await page.evaluate(failure => {
      if (failure === 'lost-device') window.testGPUDevice.destroy();
      else window.testGPUDevice.createBuffer({ size: 4, usage: 0 });
    }, failure);
    await ready(page, 'webgl2');
    expect(site.requests['index.html']).toBe(2);
    await expect(page.locator('#gamevex')).toHaveAttribute('data-save-writer', 'true');
    expect(await page.locator('#gamevex').evaluate(el => JSON.parse(el.dataset.session).sudoku.game.board)).toEqual(board);
    expect(await page.locator('#gamevex').evaluate(el => JSON.parse(el.dataset.session).sudoku.game.assisted)).toBe(true);
    await page.reload();
    await ready(page, 'webgpu');
  });
}

test('failure of both APIs gives a visible error without reloading repeatedly', async ({ page, site }) => {
  await page.addInitScript(() => {
    Object.defineProperty(navigator, 'gpu', { value: undefined });
    const getContext = HTMLCanvasElement.prototype.getContext;
    HTMLCanvasElement.prototype.getContext = function (type, ...args) {
      return type === 'webgl2' ? null : getContext.call(this, type, ...args);
    };
  });
  await page.goto(site.url);
  await expect(page.locator('#status')).toHaveClass('error');
  await expect(page.locator('#status')).toContainText('WebGL2 fallback could not continue');
  expect(site.requests['index.html']).toBe(1);
});

test('a failed WebGL2 startup after a WebGPU failure stops after one recovery', async ({ page, site }) => {
  site.webgpuModule = 'export default async () => { throw new Error("QA: WebGPU failed"); };';
  await page.addInitScript(() => {
    const getContext = HTMLCanvasElement.prototype.getContext;
    HTMLCanvasElement.prototype.getContext = function (type, ...args) {
      return type === 'webgl2' ? null : getContext.call(this, type, ...args);
    };
  });
  await page.goto(site.url + '?qa=fresh&diagnostics');
  await expect(page.locator('#status')).toHaveClass('error');
  await expect(page.locator('#status')).toContainText('WebGL2 fallback could not continue');
  expect(site.requests['index.html']).toBe(2);
});

test('an unrelated application error after readiness remains visible without changing backends', async ({ page, site }) => {
  await page.goto(site.url + '?qa=fresh&diagnostics');
  await ready(page, 'webgpu');
  await page.evaluate(() => setTimeout(() => { throw new Error('QA: unrelated application failure'); }, 0));
  await expect(page.locator('#status')).toHaveClass('error');
  await expect(page.locator('#status')).toContainText('QA: unrelated application failure');
  expect(site.requests['index.html']).toBe(1);
  await expect(page.locator('#gamevex')).toHaveAttribute('data-graphics-backend', 'webgpu');
});
