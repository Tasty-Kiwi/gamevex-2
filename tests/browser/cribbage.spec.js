import { expect, test } from '@playwright/test';

const canvas = '#gamevex';

function center(bounds) {
  return { x: bounds[0] + bounds[2] / 2, y: bounds[1] + bounds[3] / 2 };
}

async function screenPoint(page, x, y) {
  return page.locator(canvas).evaluate((el, p) => {
    const rect = el.getBoundingClientRect();
    const k = Number(el.dataset.curvature || 0);
    const qx = (p.x / rect.width) * 2 - 1;
    const qy = (p.y / rect.height) * 2 - 1;
    const radius = Math.hypot(qx, qy);
    let lo = 0;
    let hi = Math.max(1, radius);
    for (let i = 0; i < 32; i++) {
      const r = (lo + hi) / 2;
      if (r * (1 + k * r * r) < radius) lo = r;
      else hi = r;
    }
    const scale = radius ? ((lo + hi) / 2) / radius : 1;
    return {
      x: rect.left + rect.width * (1 + qx * scale) / 2,
      y: rect.top + rect.height * (1 + qy * scale) / 2,
    };
  }, { x, y });
}

async function clickBounds(page, bounds) {
  expect(bounds).toHaveLength(4);
  const p = center(bounds);
  const point = await screenPoint(page, p.x, p.y);
  await page.mouse.click(point.x, point.y);
}

async function touchBounds(page, bounds) {
  expect(bounds).toHaveLength(4);
  const p = center(bounds);
  const point = await screenPoint(page, p.x, p.y);
  await page.touchscreen.tap(point.x, point.y);
}

async function interaction(page) {
  return page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.interaction || '{}'));
}

async function save(page) {
  return page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.session || '{}'));
}

async function view(page) {
  return page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.cribbageView || '{}'));
}

async function actions(page) {
  return page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.actionBounds || '{}'));
}

function phaseName(value) {
  return String(value || '').replaceAll('_', '').toLowerCase();
}

function stable(value) {
  if (Array.isArray(value)) return value.map(stable);
  if (value && typeof value === 'object') {
    return Object.fromEntries(Object.entries(value)
      .filter(([key]) => key !== 'elapsed_ms')
      .map(([key, item]) => [key, stable(item)]));
  }
  return value;
}

async function clickAction(page, key) {
  await expect.poll(async () => {
    const b = (await actions(page))[key];
    return Array.isArray(b) && b.length === 4;
  }).toBe(true);
  await clickBounds(page, (await actions(page))[key]);
}

async function touchAction(page, key) {
  await expect.poll(async () => {
    const b = (await actions(page))[key];
    return Array.isArray(b) && b.length === 4;
  }).toBe(true);
  await touchBounds(page, (await actions(page))[key]);
}

async function clickLobbyAction(page, key) {
  await expect.poll(async () => (await interaction(page)).screen).toBe('Lobby');
  for (let i = 0; i < 20; i++) {
    const data = await actions(page);
    if (Array.isArray(data[key]) && data[key].length === 4) return clickBounds(page, data[key]);
    if ((data.lobbyScroll || 0) >= (data.lobbyMaxScroll || 0)) break;
    const old = data.lobbyScroll || 0;
    await page.mouse.move(190, 360);
    await page.mouse.wheel(0, 420);
    await expect.poll(async () => (await actions(page)).lobbyScroll || 0).not.toBe(old);
  }
  throw new Error(`Lobby action ${key} did not become visible`);
}

async function openCribbage(page) {
  await page.goto('/?qa=fresh&diagnostics');
  const c = page.locator(canvas);
  await expect.poll(async () => c.evaluate((el) => el.dataset.ready === 'true'), { timeout: 30_000 }).toBe(true);
  await expect.poll(async () => c.evaluate((el) => {
    const current = JSON.parse(el.dataset.session || '{}');
    return current.version === 11 && Boolean(current.cribbage);
  })).toBe(true);
  await clickLobbyAction(page, 'cribbage');
  await expect.poll(async () => (await interaction(page)).screen).toBe('Cribbage');
  await expect.poll(async () => (await view(page)).phase).not.toBeUndefined();
}

async function discardTwo(page, touch = false) {
  await expect.poll(async () => phaseName((await view(page)).phase)).toBe('discard');
  for (const index of [0, 1]) {
    if (touch) await touchAction(page, `cribbageCard${index}`);
    else await clickAction(page, `cribbageCard${index}`);
    await expect.poll(async () => (await view(page)).selected).toContain(index);
  }
  await expect.poll(async () => (await actions(page)).cribbageDiscard).toHaveLength(4);
  if (touch) await touchAction(page, 'cribbageDiscard');
  else await clickAction(page, 'cribbageDiscard');
  await expect.poll(async () => {
    const state = await save(page);
    return state.cribbage.crib?.length === 4 && Boolean(state.cribbage.starter)
      && state.cribbage.hands?.[0]?.length === 4;
  }, { timeout: 15_000 }).toBe(true);
}

test('Cribbage lobby starts a version 9 match and two card discards form a four card crib', async ({ page }) => {
  test.setTimeout(60_000);
  await openCribbage(page);
  expect(phaseName((await view(page)).phase)).toBe('discard');
  expect((await view(page)).legal).toEqual([]);
  for (const index of [0, 1, 2, 3, 4, 5]) {
    expect((await actions(page))[`cribbageCard${index}`]).toHaveLength(4);
  }
  await discardTwo(page);
  expect((await save(page)).cribbage.hands[0]).toHaveLength(4);
});

test('canceling a new Cribbage match preserves the exact current game', async ({ page }) => {
  test.setTimeout(60_000);
  await openCribbage(page);
  await discardTwo(page);
  const before = stable((await save(page)).cribbage);
  await clickAction(page, 'cribbageNew');
  await expect.poll(async () => (await interaction(page)).modal).toBe('cribbage-replace');
  await clickAction(page, 'cancel');
  expect(stable((await save(page)).cribbage)).toEqual(before);
});

test('Cribbage pauses in the lobby, reloads the saved match, and leaves adjacent games unchanged', async ({ page }) => {
  test.setTimeout(60_000);
  await openCribbage(page);
  await discardTwo(page);
  const otherGames = stable(await page.locator(canvas).evaluate((el) => {
    const { cribbage, ...games } = JSON.parse(el.dataset.session || '{}');
    return games;
  }));
  const currentSession = stable(await save(page));
  await clickAction(page, 'lobby');
  await expect.poll(async () => (await interaction(page)).screen).toBe('Lobby');
  const paused = stable(await save(page));
  await page.reload();
  await expect.poll(async () => page.locator(canvas).evaluate((el) => el.dataset.ready === 'true'), { timeout: 30_000 }).toBe(true);
  await expect.poll(async () => (await interaction(page)).screen).toBe('Lobby');
  expect(stable(await save(page))).toEqual(paused);
  const pausedOtherGames = Object.fromEntries(Object.entries(paused).filter(([key]) => key !== 'cribbage'));
  expect(stable(pausedOtherGames)).toEqual(otherGames);
  expect(stable(paused.cribbage)).toEqual(currentSession.cribbage);
  await clickLobbyAction(page, 'cribbage');
  expect(stable(await save(page))).toEqual(paused);
});

test.describe('Cribbage phone controls', () => {
  test.use({ viewport: { width: 390, height: 740 }, hasTouch: true });

  test('touch can discard two cards and open rules without advancing gameplay', async ({ page }) => {
    test.setTimeout(60_000);
    await openCribbage(page);
    const before = (await save(page)).cribbage;
    const bounds = await actions(page);
    for (const key of ['cribbageCard0', 'cribbageCard1', 'cribbageRules']) {
      expect(bounds[key], `${key} has a published phone hit bound`).toHaveLength(4);
      expect(bounds[key][0]).toBeGreaterThanOrEqual(0);
      expect(bounds[key][1]).toBeGreaterThanOrEqual(0);
      expect(bounds[key][0] + bounds[key][2]).toBeLessThanOrEqual(390.5);
      expect(bounds[key][1] + bounds[key][3]).toBeLessThanOrEqual(740.5);
    }
    await touchAction(page, 'cribbageRules');
    await expect.poll(async () => (await view(page)).rulesOpen).toBe(true);
    expect(Object.keys(await actions(page)).filter((key) => key.startsWith('cribbageCard'))).toHaveLength(0);
    await touchAction(page, 'cribbageRules');
    await expect.poll(async () => (await view(page)).rulesOpen).toBe(false);
    expect(stable((await save(page)).cribbage)).toEqual(stable(before));
    await discardTwo(page, true);
  });
});

test('Cribbage legal pegging, show, and continue actions complete a round and start the next one', async ({ page }) => {
  test.setTimeout(60_000);
  await openCribbage(page);
  await discardTwo(page);
  const startingRound = Number((await view(page)).round || 1);
  const deadline = Date.now() + 45_000;
  while (Date.now() < deadline) {
    const state = await view(page);
    const phase = phaseName(state.phase);
    if (Number(state.round || 1) > startingRound) break;
    if (phase === 'matchcomplete') break;
    if (phase === 'roundcomplete') {
      await clickAction(page, 'cribbageContinue');
      await expect.poll(async () => Number((await view(page)).round || 1)).toBeGreaterThan(startingRound);
      continue;
    }
    if (phase === 'show') {
      const showStep = (await save(page)).cribbage.show_step;
      await clickAction(page, 'cribbageContinue');
      await expect.poll(async () => {
        const current = (await save(page)).cribbage;
        return phaseName(current.phase) !== 'show' || current.show_step !== showStep;
      }, { timeout: 10_000 }).toBe(true);
      continue;
    }
    if (phase !== 'pegging') throw new Error(`Unexpected Cribbage phase during round play: ${state.phase}`);
    if (state.turn !== 0) {
      await expect.poll(async () => {
        const current = await view(page);
        return phaseName(current.phase) !== 'pegging' || current.turn === 0;
      }, { timeout: 15_000 }).toBe(true);
      continue;
    }
    const before = (await save(page)).cribbage;
    if (state.legal.length > 0) {
      const index = state.legal[0];
      await expect.poll(async () => (await actions(page))[`cribbageCard${index}`]).toHaveLength(4);
      await clickAction(page, `cribbageCard${index}`);
    } else {
      await clickAction(page, 'cribbageGo');
    }
    await expect.poll(async () => {
      const current = (await save(page)).cribbage;
      return phaseName(current.phase) !== 'pegging'
        || current.played.length !== before.played.length
        || current.remaining[0].length !== before.remaining[0].length
        || current.remaining[1].length !== before.remaining[1].length
        || current.count !== before.count
        || current.turn !== before.turn
        || JSON.stringify(current.go) !== JSON.stringify(before.go);
    }, { timeout: 15_000 }).toBe(true);
  }
  expect(Number((await view(page)).round || 1)).toBeGreaterThan(startingRound);
  expect(phaseName((await view(page)).phase)).toBe('discard');
});
