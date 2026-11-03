import { test as base } from '@playwright/test';
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';

// Mount the real artifact at a Pages-style subpath. Each test gets its own
// origin and browser context, so updates and save ownership stay isolated.
export const test = base.extend({
  context: async ({ context }, use) => {
    if (process.env.GAMEVEX_TEST_GRAPHICS === 'webgl2') await context.addInitScript(() => {
      Object.defineProperty(navigator, 'gpu', { value: undefined });
    });
    await use(context);
  },
  site: async ({}, use) => {
    const state = { revision: 1, failWasm: false, failedWasmRequests: 0, requests: {}, webgpuModule: null, failWebGPUWasm: false };
    const types = { html: 'text/html', js: 'text/javascript', wasm: 'application/wasm',
      webmanifest: 'application/manifest+json', png: 'image/png' };
    const server = createServer(async (req, res) => {
      const path = new URL(req.url, 'http://localhost').pathname;
      if (!path.startsWith('/gamevex-2/')) { res.writeHead(404).end(); return; }
      const relative = path.slice('/gamevex-2/'.length) || 'index.html';
      state.requests[relative] = (state.requests[relative] || 0) + 1;
      if (relative.includes('..')) { res.writeHead(400).end(); return; }
      if ((state.failWasm || (state.failWebGPUWasm && relative.startsWith('pkg/webgpu/'))) && relative.endsWith('.wasm')) {
        state.failedWasmRequests++;
        res.writeHead(503).end(); return;
      }
      try {
        let body = await readFile(resolve('web', relative));
        if (relative === 'pkg/webgpu/gamevex_app.js' && state.webgpuModule !== null) body = Buffer.from(state.webgpuModule);
        if (relative === 'sw.js') body = Buffer.from(body.toString().replace(
          /const VERSION = "([^"]+)";/, `const VERSION = "$1-test-${state.revision}";`,
        ));
        if (relative === 'pwa.js') body = Buffer.concat([body, Buffer.from(`\n// test revision ${state.revision}\n`)]);
        res.writeHead(200, { 'Content-Type': types[relative.split('.').at(-1)] || 'application/octet-stream',
          'Cache-Control': 'no-store' }).end(body);
      } catch { res.writeHead(404).end(); }
    });
    await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
    state.url = `http://127.0.0.1:${server.address().port}/gamevex-2/`;
    await use(state);
    await new Promise(resolve => server.close(resolve));
  },
});

