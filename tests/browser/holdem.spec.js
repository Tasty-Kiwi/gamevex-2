import { expect, test } from '@playwright/test';
import { PNG } from 'pngjs';

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
  return page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.holdemView || '{}'));
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

async function openHoldem(page) {
  await page.goto('/?qa=fresh&diagnostics');
  const c = page.locator(canvas);
  await expect.poll(async () => c.evaluate((el) => el.dataset.ready === 'true'), { timeout: 30_000 }).toBe(true);
  await expect.poll(async () => c.evaluate((el) => {
    const current = JSON.parse(el.dataset.session || '{}');
    return current.version === 11;
  })).toBe(true);
  await clickLobbyAction(page, 'holdem');
  await expect.poll(async () => (await interaction(page)).screen).toBe('Holdem');
  await expect.poll(async () => (await view(page)).phase).not.toBeUndefined();
}

async function beginHand(page) {
  await expect.poll(async () => phaseName((await view(page)).phase)).toBe('idle');
  await startHand(page);
}

async function startHand(page) {
  await clickAction(page, 'holdemStart');
  await expect.poll(async () => {
    const current = await view(page);
    return phaseName(current.phase) === 'preflop' && current.seats?.length >= 2 && !current.animating;
  }).toBe(true);
}

async function waitForHumanTurn(page) {
  await expect.poll(async () => {
    const current = await view(page);
    return current.turn === 0 && current.legal?.length > 0 && !current.animating;
  }, { timeout: 15_000 }).toBe(true);
}

test('Holdem starts with the exact blind, hides CPU cards, and saves only its bankroll', async ({ page }) => {
  test.setTimeout(45_000);
  await openHoldem(page);
  const before = stable(await save(page));
  const balanceBefore = (await view(page)).bankroll;
  expect(balanceBefore).toBe(1000);
  await beginHand(page);

  const current = await view(page);
  expect(phaseName(current.phase)).toBe('preflop');
  expect(current.bankroll).toBe(balanceBefore - current.buyIn + current.seats[0].stack);
  expect(current.buyIn).toBe(100);
  expect(current.seats[0].committed).toBe(5);
  expect(current.seats[0].stack).toBe(95);
  expect(current.seats.slice(1).map((seat) => seat.stack)).toEqual([90]);
  expect(current.seats[0].hole).toHaveLength(2);
  expect(current.seats.slice(1).every((seat) => seat.hole === null)).toBe(true);
  expect((await save(page)).version).toBe(11);
  expect(Object.fromEntries(Object.entries(stable(await save(page))).filter(([key]) => key !== 'holdem_balance')))
    .toEqual(Object.fromEntries(Object.entries(before).filter(([key]) => key !== 'holdem_balance')));
  expect((await actions(page)).holdemRules).toHaveLength(4);
});

test('Holdem can skip the remaining CPU play after folding', async ({ page }) => {
  await openHoldem(page);
  await clickAction(page, 'holdemOpponentsUp');
  await expect.poll(async () => (await view(page)).opponents).toBe(2);
  await clickAction(page, 'holdemOpponentsUp');
  await expect.poll(async () => (await view(page)).opponents).toBe(3);
  await beginHand(page);
  await waitForHumanTurn(page);
  expect((await actions(page)).holdemSkip).toBeUndefined();
  await clickAction(page, 'holdemFold');
  await expect.poll(async () => (await view(page)).seats[0].folded).toBe(true);
  const balance = (await view(page)).bankroll;
  await clickAction(page, 'holdemSkip');
  await expect.poll(async () => ['showdown', 'complete'].includes(phaseName((await view(page)).phase))).toBe(true);
  expect((await view(page)).bankroll).toBe(balance);
  await expect.poll(async () => Boolean((await actions(page)).holdemStart)).toBe(true);
  expect((await actions(page)).holdemSkip).toBeUndefined();
});

test('Holdem call and check advance the hand while the buy-in stays accounted for', async ({ page }) => {
  test.setTimeout(45_000);
  await openHoldem(page);
  const initialBankroll = (await view(page)).bankroll;
  await beginHand(page);
  const buyIn = (await view(page)).buyIn;
  const deadline = Date.now() + 20_000;
  while (Date.now() < deadline) {
    const state = await view(page);
    const phase = phaseName(state.phase);
    if (['flop', 'turn', 'river', 'showdown', 'complete'].includes(phase)) break;
    if (phase !== 'preflop') throw new Error(`Unexpected Holdem phase: ${state.phase}`);
    if (state.turn !== 0 || state.animating) {
      await expect.poll(async () => {
        const current = await view(page);
        return phaseName(current.phase) !== 'preflop' || current.turn === 0 && !current.animating;
      }, { timeout: 10_000 }).toBe(true);
      continue;
    }
    const legal = state.legal;
    const choice = legal.includes('check') ? 'holdemCheck' : 'holdemCall';
    expect(legal).toContain(choice === 'holdemCheck' ? 'check' : 'call');
    const committed = (v) => JSON.stringify({ phase: v.phase, turn: v.turn, seats: v.seats, board: v.board, pot: v.pot });
    const before = committed(state);
    await clickAction(page, choice);
    // Wait for the command to commit; the previous legal list can still be
    // present when the mouse event returns, before the next game frame.
    await expect.poll(async () => committed(await view(page)), { timeout: 10_000 }).not.toBe(before);
    const current = await view(page);
    expect(current.bankroll).toBe(initialBankroll - buyIn + current.seats[0].stack);
  }
  expect(['flop', 'turn', 'river', 'showdown', 'complete']).toContain(phaseName((await view(page)).phase));
  expect((await view(page)).board.length).toBeGreaterThanOrEqual(3);
});

test('leaving an active hand can be canceled or confirmed without losing unspent chips', async ({ page }) => {
  test.setTimeout(45_000);
  await openHoldem(page);
  await beginHand(page);
  const before = stable((await save(page)).holdem_balance);
  await expect.poll(async () => (await view(page)).animating).toBe(false);
  const beforeView = stable(await view(page));
  await clickAction(page, 'lobby');
  await expect.poll(async () => (await interaction(page)).modal).toBe('holdem-leave');
  await clickAction(page, 'cancel');
  expect(stable((await save(page)).holdem_balance)).toEqual(before);
  await expect.poll(async () => (await view(page)).animating).toBe(false);
  const afterCancelView = stable(await view(page));
  delete afterCancelView.animating;
  const stableBeforeView = { ...beforeView };
  delete stableBeforeView.animating;
  expect(afterCancelView).toEqual(stableBeforeView);

  await clickAction(page, 'lobby');
  await expect.poll(async () => (await interaction(page)).modal).toBe('holdem-leave');
  await clickAction(page, 'confirm');
  await expect.poll(async () => (await interaction(page)).screen).toBe('Lobby');
  expect((await save(page)).holdem_balance).toBeGreaterThanOrEqual(before);
  await clickLobbyAction(page, 'holdem');
  await expect.poll(async () => phaseName((await view(page)).phase)).toBe('idle');
  expect((await view(page)).bankroll).toBe((await save(page)).holdem_balance);
});

test('reload returns an active Holdem table to the lobby with its bankroll preserved', async ({ page }) => {
  test.setTimeout(45_000);
  await openHoldem(page);
  await beginHand(page);
  const persistedBalance = (await save(page)).holdem_balance;
  await page.reload();
  await expect.poll(async () => page.locator(canvas).evaluate((el) => el.dataset.ready === 'true'), { timeout: 30_000 }).toBe(true);
  await expect.poll(async () => (await interaction(page)).screen).toBe('Lobby');
  expect((await save(page)).holdem_balance).toBe(persistedBalance);
  await clickLobbyAction(page, 'holdem');
  await expect.poll(async () => phaseName((await view(page)).phase).toLowerCase()).toBe('idle');
  expect((await view(page)).bankroll).toBe(persistedBalance);
});

test.describe('Holdem phone controls', () => {
  test.use({ viewport: { width: 390, height: 740 }, hasTouch: true });

  test('touch setup changes opponents, starts a hand, and rules pause play', async ({ page }) => {
    test.setTimeout(45_000);
    await openHoldem(page);
    const { viewport } = await page.locator(canvas).evaluate((el) => ({
      viewport: JSON.parse(el.dataset.interaction || '{}').viewport,
    }));
    expect(viewport).toEqual([390, 740]);
    await expect.poll(async () => (await actions(page)).holdemOpponentsUp).toHaveLength(4);
    await touchAction(page, 'holdemOpponentsUp');
    await expect.poll(async () => (await view(page)).opponents).toBe(2);
    await expect.poll(async () => (await actions(page)).holdemRules).toHaveLength(4);
    await touchAction(page, 'holdemRules');
    await expect.poll(async () => (await view(page)).rulesOpen).toBe(true);
    expect(Object.keys(await actions(page)).filter((key) => key.startsWith('holdem') && key !== 'holdemRules'))
      .toHaveLength(0);
    await touchAction(page, 'holdemRules');
    await expect.poll(async () => (await view(page)).rulesOpen).toBe(false);
    await touchAction(page, 'holdemStart');
    await expect.poll(async () => phaseName((await view(page)).phase)).toBe('preflop');
    await expect.poll(async () => (await actions(page)).holdemFold).toHaveLength(4);
    for (const key of ['holdemFold', 'holdemRules', 'lobby']) {
      const bounds = (await actions(page))[key];
      expect(bounds, `${key} has a visible phone hit bound`).toHaveLength(4);
      expect(bounds[0]).toBeGreaterThanOrEqual(0);
      expect(bounds[1]).toBeGreaterThanOrEqual(0);
      expect(bounds[0] + bounds[2]).toBeLessThanOrEqual(390.5);
      expect(bounds[1] + bounds[3]).toBeLessThanOrEqual(740.5);
    }
  });
});

test('an all-in hand settles the bankroll and the next three-seat deal rotates the dealer', async ({ page }) => {
  test.setTimeout(60_000);
  await openHoldem(page);
  await clickAction(page, 'holdemOpponentsUp');
  await expect.poll(async () => (await view(page)).opponents).toBe(2);
  const startingBankroll = (await view(page)).bankroll;
  await beginHand(page);
  expect((await view(page)).turn).toBe(0);
  expect((await view(page)).seats.map((seat) => seat.committed)).toEqual([0, 5, 10]);

  await waitForHumanTurn(page);
  expect((await view(page)).legal).toContain('all_in');
  await clickAction(page, 'holdemAllIn');
  await expect.poll(async () => {
    const current = await view(page);
    return ['showdown', 'complete'].includes(phaseName(current.phase)) && !current.animating;
  }, { timeout: 30_000 }).toBe(true);
  const settled = await view(page);
  expect(settled.bankroll).toBe(startingBankroll - settled.buyIn + settled.seats[0].stack);

  await clickAction(page, 'holdemStart');
  await expect.poll(async () => (await view(page)).handNumber).toBe(2);
  const next = await view(page);
  expect(next.handNumber).toBe(2);
  expect(next.dealer).toBe(1);
});

for (const density of [1, 2]) {
  test.describe(`Holdem text stability at ${density}x density`, () => {
    test.use({ viewport: { width: 1280, height: 800 }, deviceScaleFactor: density });
    test('showdown keeps the title visible', async ({ page }) => {
      test.setTimeout(60_000);
      await openHoldem(page);
      const clip = { x: 82, y: 101, width: 300, height: 30 };
      await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(() => requestAnimationFrame(resolve)))));
      const ink = (buffer) => {
        const { data } = PNG.sync.read(buffer);
        let count = 0;
        for (let i = 0; i < data.length; i += 4) {
          if (Math.max(data[i], data[i + 1], data[i + 2]) > 100) count++;
        }
        return count;
      };
      let before;
      await expect.poll(async () => {
        before = await page.screenshot({ clip });
        return ink(before);
      }).toBeGreaterThan(1000 * density * density);
      for (let i = 0; i < 3; i++) {
        await clickAction(page, 'holdemOpponentsUp');
        await expect.poll(async () => (await view(page)).opponents).toBe(i + 2);
      }
      await beginHand(page);
      await waitForHumanTurn(page);
      await clickAction(page, 'holdemAllIn');
      await expect.poll(async () => {
        const v = await view(page);
        return ['showdown', 'complete'].includes(v.phase) && !v.animating;
      }, { timeout: 25_000 }).toBe(true);
      // The title lies above every animated/table primitive. Allow small
      // antialiasing differences while detecting lost text in the actual pixels.
      await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(() => requestAnimationFrame(resolve)))));
      const after = await page.screenshot({ clip });
      expect(ink(after)).toBeGreaterThan(ink(before) * 0.9);
      expect(ink(after)).toBeLessThan(ink(before) * 1.1);
    });
  });
}
