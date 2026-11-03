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

async function clickSurface(page, x, y) {
  const point = await screenPoint(page, x, y);
  await page.mouse.click(point.x, point.y);
}

async function touchSurface(page, x, y) {
  const point = await screenPoint(page, x, y);
  await page.touchscreen.tap(point.x, point.y);
}

async function interaction(page) {
  return page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.interaction || '{}'));
}

async function save(page) {
  return page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.session || '{}'));
}

async function view(page) {
  return page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.heartsView || '{}'));
}

async function actions(page) {
  return page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.actionBounds || '{}'));
}

function gameplay(game) {
  const { elapsed_ms, ...state } = game;
  return state;
}

function phaseName(value) {
  return String(value || '').replaceAll('_', '').toLowerCase();
}

function stableState(value) {
  if (Array.isArray(value)) return value.map(stableState);
  if (value && typeof value === 'object') {
    return Object.fromEntries(Object.entries(value)
      .filter(([key]) => key !== 'elapsed_ms')
      .map(([key, item]) => [key, stableState(item)]));
  }
  return value;
}

function nonHeartsGameplay(session) {
  const { hearts, ...otherGames } = session;
  return stableState(otherGames);
}

async function clickBounds(page, bounds) {
  expect(bounds).toHaveLength(4);
  const point = center(bounds);
  await clickSurface(page, point.x, point.y);
}

async function clickAction(page, key) {
  await expect.poll(async () => {
    const bounds = (await actions(page))[key];
    return Array.isArray(bounds) && bounds.length === 4;
  }).toBe(true);
  await clickBounds(page, (await actions(page))[key]);
}

async function touchAction(page, key) {
  await expect.poll(async () => {
    const bounds = (await actions(page))[key];
    return Array.isArray(bounds) && bounds.length === 4;
  }).toBe(true);
  const point = center((await actions(page))[key]);
  await touchSurface(page, point.x, point.y);
}

async function clickLobbyAction(page, key) {
  await expect.poll(async () => (await interaction(page)).screen).toBe('Lobby');
  for (let reset = 0; reset < 16; reset++) {
    const scroll = (await actions(page)).lobbyScroll || 0;
    if (scroll <= 0) break;
    await page.mouse.move(190, 360);
    await page.mouse.wheel(0, -420);
    await expect.poll(async () => (await actions(page)).lobbyScroll || 0).toBeLessThan(scroll);
  }
  for (let attempt = 0; attempt < 16; attempt++) {
    const data = await actions(page);
    const bounds = data[key];
    if (Array.isArray(bounds) && bounds.length === 4) return clickBounds(page, bounds);
    if ((data.lobbyScroll || 0) >= (data.lobbyMaxScroll || 0)) break;
    const old = data.lobbyScroll || 0;
    await page.mouse.move(190, 360);
    await page.mouse.wheel(0, 420);
    await expect.poll(async () => (await actions(page)).lobbyScroll || 0).not.toBe(old);
  }
  throw new Error(`Lobby action ${key} did not become visible`);
}

async function openHearts(page) {
  await page.goto('/?qa=fresh&diagnostics');
  const c = page.locator(canvas);
  await expect.poll(async () => c.evaluate((el) => el.dataset.ready === 'true'), { timeout: 30_000 }).toBe(true);
  await expect.poll(async () => c.evaluate((el) => {
    const current = JSON.parse(el.dataset.session || '{}');
    return current.version === 12 && Boolean(current.hearts);
  })).toBe(true);
  await clickLobbyAction(page, 'hearts');
  await expect.poll(async () => (await interaction(page)).screen).toBe('Hearts');
  await expect.poll(async () => (await view(page)).phase).not.toBeUndefined();
}

async function beginPassing(page, touch = false) {
  await expect.poll(async () => phaseName((await view(page)).phase)).toBe('passing');
  const hand = (await save(page)).hearts.hands[0];
  expect(hand.length).toBeGreaterThanOrEqual(3);
  const indices = [0, 1, 2];
  for (const index of indices) {
    if (touch) await touchAction(page, `heartsCard${index}`);
    else await clickAction(page, `heartsCard${index}`);
    await expect.poll(async () => (await view(page)).selected).toContain(index);
  }
  await expect.poll(async () => (await actions(page)).heartsPass).toHaveLength(4);
  if (touch) await touchAction(page, 'heartsPass');
  else await clickAction(page, 'heartsPass');
  await expect.poll(async () => phaseName((await view(page)).phase)).toBe('playing');
}

test('Hearts lobby entry, three-card passing, and rules action are accessible', async ({ page }) => {
  test.setTimeout(45_000);
  await openHearts(page);
  expect(phaseName((await view(page)).phase)).toBe('passing');
  expect((await actions(page)).heartsRules).toHaveLength(4);
  const otherGamesBefore = nonHeartsGameplay(await save(page));
  for (const index of [0, 1]) await clickAction(page, `heartsCard${index}`);
  await expect.poll(async () => (await view(page)).selected.length).toBe(2);
  expect((await actions(page)).heartsPass).toBeUndefined();
  await clickAction(page, 'heartsCard2');
  await expect.poll(async () => (await actions(page)).heartsPass).toHaveLength(4);
  expect((await actions(page)).heartsCard3).toBeUndefined();
  expect((await actions(page)).heartsCard0).toHaveLength(4);
  await clickAction(page, 'heartsPass');
  await expect.poll(async () => phaseName((await view(page)).phase)).toBe('playing');
  expect(nonHeartsGameplay(await save(page))).toEqual(otherGamesBefore);
});

test('Hearts autosaves gameplay and reloads the exact saved state after returning to the lobby', async ({ page }) => {
  test.setTimeout(45_000);
  await openHearts(page);
  await beginPassing(page);
  await clickAction(page, 'lobby');
  await expect.poll(async () => (await interaction(page)).screen).toBe('Lobby');
  const persisted = gameplay((await save(page)).hearts);
  expect(phaseName(persisted.phase)).toBe('playing');
  await page.reload();
  await expect.poll(async () => page.locator(canvas).evaluate((el) => el.dataset.ready === 'true'), { timeout: 30_000 }).toBe(true);
  await expect.poll(async () => (await interaction(page)).screen).toBe('Lobby');
  expect(gameplay((await save(page)).hearts)).toEqual(persisted);
  await clickLobbyAction(page, 'hearts');
  await expect.poll(async () => phaseName((await view(page)).phase)).toBe('playing');
  expect(gameplay((await save(page)).hearts)).toEqual(persisted);
});

test('Hearts publishes legal plays and enforces following suit', async ({ page }) => {
  test.setTimeout(45_000);
  await openHearts(page);
  await beginPassing(page);
  await expect.poll(async () => {
    const state = await view(page);
    return phaseName(state.phase) === 'playing' && state.turn === 0 && state.legal.length > 0;
  }, { timeout: 30_000 }).toBe(true);
  const state = await view(page);
  expect(state.legal.every((index) => Number.isInteger(index))).toBe(true);
  const hand = (await save(page)).hearts.hands[0];
  const game = (await save(page)).hearts;
  const trick = game.trick;
  if (trick.length === 0 && game.completed_tricks === 0) {
    expect(state.legal).toEqual([hand.findIndex((card) => card.rank === 2 && card.suit === 'Clubs')]);
  }
  if (trick.length > 0) {
    const leadSuit = trick[0].card.suit;
    const matching = hand.flatMap((card, index) => card.suit === leadSuit ? [index] : []);
    const firstTrickPoints = hand.flatMap((card, index) => card.suit === 'Hearts'
      || (card.suit === 'Spades' && card.rank === 12) ? [] : [index]);
    const expected = matching.length > 0 ? matching
      : game.completed_tricks === 0 && firstTrickPoints.length > 0
        ? firstTrickPoints : hand.map((_, index) => index);
    expect([...state.legal].sort((a, b) => a - b)).toEqual(expected);
  }
  const candidate = state.legal[0];
  const before = (await save(page)).hearts;
  await clickAction(page, `heartsCard${candidate}`);
  await expect.poll(async () => {
    const current = (await save(page)).hearts;
    return current.turn !== before.turn || phaseName(current.phase) !== 'playing'
      || current.hands[0].length < before.hands[0].length;
  }, { timeout: 10_000 }).toBe(true);
  expect((await save(page)).hearts.hands[0].length).toBe(before.hands[0].length - 1);
});

test.describe('Hearts phone controls', () => {
  test.use({ viewport: { width: 390, height: 740 }, hasTouch: true });

  test('fit a phone viewport and touch can pass three cards', async ({ page }) => {
    test.setTimeout(45_000);
    await openHearts(page);
    const { viewport, bounds } = await page.locator(canvas).evaluate((el) => ({
      viewport: JSON.parse(el.dataset.interaction || '{}').viewport,
      bounds: JSON.parse(el.dataset.actionBounds || '{}'),
    }));
    expect(viewport).toEqual([390, 740]);
    const controls = [...Array.from({ length: 13 }, (_, index) => `heartsCard${index}`), 'heartsRules', 'heartsNew'];
    for (const key of controls) {
      expect(bounds[key], `${key} has a published hit bound`).toHaveLength(4);
      expect(bounds[key][0]).toBeGreaterThanOrEqual(0);
      expect(bounds[key][1]).toBeGreaterThanOrEqual(0);
      expect(bounds[key][0] + bounds[key][2]).toBeLessThanOrEqual(390.5);
      expect(bounds[key][1] + bounds[key][3]).toBeLessThanOrEqual(740.5);
    }
    expect(bounds.heartsPass).toBeUndefined();
    await clickAction(page, 'heartsRules');
    await expect.poll(async () => (await view(page)).rulesOpen).toBe(true);
    expect(Object.keys(await actions(page)).filter((key) => key.startsWith('heartsCard'))).toHaveLength(0);
    await clickAction(page, 'heartsRules');
    await expect.poll(async () => (await view(page)).rulesOpen).toBe(false);
    await expect.poll(async () => (await actions(page)).heartsCard12).toHaveLength(4);
    await expect.poll(async () => (await actions(page)).heartsPass).toBeUndefined();
    await beginPassing(page, true);
    await expect.poll(async () => (await actions(page)).heartsPass).toBeUndefined();
  });
});

test('canceling a new Hearts match preserves the current hand and confirmation starts a fresh match', async ({ page }) => {
  test.setTimeout(45_000);
  await openHearts(page);
  await beginPassing(page);
  await clickAction(page, 'heartsNew');
  await expect.poll(async () => (await interaction(page)).modal).toBe('hearts-replace');
  const current = gameplay((await save(page)).hearts);
  await clickAction(page, 'cancel');
  expect(gameplay((await save(page)).hearts)).toEqual(current);
  await clickAction(page, 'heartsNew');
  await expect.poll(async () => (await interaction(page)).modal).toBe('hearts-replace');
  await clickAction(page, 'confirm');
  await expect.poll(async () => (await save(page)).hearts.hand_number).toBe(1);
  await expect.poll(async () => phaseName((await view(page)).phase)).toBe('passing');
  expect((await save(page)).hearts.totals).toEqual([0, 0, 0, 0]);
});

test('Hearts hover lifts and highlights a card without losing its resting hit area', async ({ page }) => {
  await openHearts(page);
  await page.mouse.move(8, 8);
  await expect.poll(async () => (await view(page)).hovered).toBeNull();
  const before = (await view(page)).handPositions[0];
  const bounds = (await actions(page)).heartsCard0;
  // The exposed left edge avoids the next card. Stay near the bottom edge:
  // lifting the card must not repeatedly drop and reacquire this hover.
  const point = await screenPoint(page, bounds[0] + 8, bounds[1] + bounds[3] - 2);
  await page.mouse.move(point.x, point.y);
  await expect.poll(async () => (await view(page)).hovered).toBe(0);
  await expect.poll(async () => (await view(page)).handPositions[0][1]).toBeLessThan(before[1] - 16);
  const steady = await page.locator(canvas).evaluate(async (el) => {
    const samples = [];
    for (let frame = 0; frame < 8; frame++) {
      await new Promise((resolve) => requestAnimationFrame(resolve));
      const current = JSON.parse(el.dataset.heartsView);
      samples.push({ hovered: current.hovered, y: current.handPositions[0][1] });
    }
    return samples;
  });
  expect(steady.every((frame) => frame.hovered === 0 && frame.y < before[1] - 16)).toBe(true);
  await page.mouse.click(point.x, point.y);
  await expect.poll(async () => (await view(page)).selected).toEqual([0]);
  await page.mouse.move(8, 8);
  await expect.poll(async () => (await view(page)).hovered).toBeNull();
  expect((await view(page)).selected).toEqual([0]);
  expect((await view(page)).handPositions[0][1]).toBeLessThan(before[1] - 16);
});

test('Hearts pass and trick collection commit safely and do not replay after reload', async ({ page }) => {
  await openHearts(page);
  await beginPassing(page);
  await expect.poll(async () => {
    const current = await view(page);
    return current.turn === 0 && current.legal.length > 0 && !current.animating;
  }, { timeout: 15_000 }).toBe(true);
  const index = (await view(page)).legal[0];
  await clickAction(page, `heartsCard${index}`);
  await expect.poll(async () => (await save(page)).hearts.completed_tricks, { timeout: 15_000 }).toBe(1);
  const collected = (await save(page)).hearts;
  const winningSeat = collected.captured.findIndex((cards) => cards.length === 4);
  expect(winningSeat).toBeGreaterThanOrEqual(0);
  expect(collected.captured.flat()).toHaveLength(4);
  await clickAction(page, 'lobby');
  const paused = (await save(page)).hearts;
  await page.reload();
  await expect.poll(async () => page.locator(canvas).evaluate((el) => el.dataset.ready === 'true'), { timeout: 30_000 }).toBe(true);
  expect((await save(page)).hearts).toEqual(paused);
  await clickLobbyAction(page, 'hearts');
  const restored = await view(page);
  expect(['Pass', 'Collect', 'Deal']).not.toContain(restored.motion);
  expect((await save(page)).hearts.captured[winningSeat]).toEqual(collected.captured[winningSeat]);
});
