import { revealLobbyGame } from './lobby.js';
import { expect, test } from '@playwright/test';

const canvas = '#gamevex';

test.use({ hasTouch: true });

test.beforeEach(async ({ page }) => {
  page.__rouletteDiagnostics = { errors: [], failedRequests: [], badResponses: [] };
  page.on('pageerror', (error) => page.__rouletteDiagnostics.errors.push(error.message));
  page.on('console', (message) => {
    if (message.type() === 'error') page.__rouletteDiagnostics.errors.push(message.text());
  });
  page.on('requestfailed', (request) => page.__rouletteDiagnostics.failedRequests.push(request.url()));
  page.on('response', (response) => {
    if (response.status() >= 400) page.__rouletteDiagnostics.badResponses.push(`${response.status()} ${response.url()}`);
  });
});

test.afterEach(async ({ page }) => {
  expect(page.__rouletteDiagnostics).toEqual({ errors: [], failedRequests: [], badResponses: [] });
});

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
  const { x, y } = center(bounds);
  const point = await screenPoint(page, x, y);
  await page.mouse.click(point.x, point.y);
}

async function touchBounds(page, bounds) {
  expect(bounds).toHaveLength(4);
  const { x, y } = center(bounds);
  const point = await screenPoint(page, x, y);
  await page.touchscreen.tap(point.x, point.y);
}

async function dragTouch(page, from, to) {
  const start = await screenPoint(page, from.x, from.y);
  const end = await screenPoint(page, to.x, to.y);
  const cdp = await page.context().newCDPSession(page);
  const touch = (point) => ({ x: point.x, y: point.y, id: 1, radiusX: 2, radiusY: 2, force: 1 });
  try {
    await cdp.send('Input.dispatchTouchEvent', { type: 'touchStart', touchPoints: [touch(start)] });
    for (let step = 1; step <= 5; step++) {
      const point = {
        x: start.x + (end.x - start.x) * step / 5,
        y: start.y + (end.y - start.y) * step / 5,
      };
      await cdp.send('Input.dispatchTouchEvent', { type: 'touchMove', touchPoints: [touch(point)] });
    }
    await cdp.send('Input.dispatchTouchEvent', { type: 'touchEnd', touchPoints: [] });
  } finally {
    await cdp.detach().catch(() => {});
  }
}

async function interaction(page) {
  return page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.interaction || '{}'));
}

async function save(page) {
  return page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.session || '{}'));
}

async function view(page) {
  return page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.rouletteView || '{}'));
}

async function actions(page) {
  return page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.actionBounds || '{}'));
}

async function rouletteView(page) {
  return view(page);
}

async function swipeTable(page, direction = 'left', fraction = 0.72) {
  const current = await rouletteView(page);
  const clip = current.tableClip;
  expect(clip).toHaveLength(4);
  const y = clip[1] + clip[3] * 0.68;
  const xStart = clip[0] + clip[2] * (direction === 'left' ? 0.82 : 0.18);
  const xEnd = xStart + clip[2] * fraction * (direction === 'left' ? -1 : 1);
  await dragTouch(page, { x: xStart, y }, { x: xEnd, y });
  await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
}

async function clickAction(page, key) {
  await expect.poll(async () => {
    const bounds = (await actions(page))[key];
    return Array.isArray(bounds) && bounds.length === 4;
  }).toBe(true);
  await clickBounds(page, (await actions(page))[key]);
  await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
}

async function touchAction(page, key) {
  await expect.poll(async () => {
    const bounds = (await actions(page))[key];
    return Array.isArray(bounds) && bounds.length === 4;
  }).toBe(true);
  await touchBounds(page, (await actions(page))[key]);
  await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
}

async function clickLobbyAction(page, key) {
  const activate = (bounds) => clickBounds(page, bounds);
  await activate(await revealLobbyGame(page, key, activate));
}

async function openRoulette(page, viewport) {
  if (viewport) await page.setViewportSize(viewport);
  await page.goto('/?qa=fresh&diagnostics');
  const c = page.locator(canvas);
  await expect.poll(async () => c.evaluate((el) => el.dataset.ready === 'true'), { timeout: 30_000 }).toBe(true);
  await expect.poll(async () => JSON.parse(await c.evaluate((el) => el.dataset.session || '{}')).version).toBe(12);
  await clickLobbyAction(page, 'roulette');
  await expect.poll(async () => (await interaction(page)).screen).toBe('Roulette');
  await expect.poll(async () => (await view(page)).bankroll).toBe(1000);
}

function stable(value) {
  if (Array.isArray(value)) return value.map(stable);
  if (value && typeof value === 'object') {
    return Object.fromEntries(Object.entries(value).map(([key, item]) => [key, stable(item)]));
  }
  return value;
}

const redNumbers = new Set([1, 3, 5, 7, 9, 12, 14, 16, 18, 19, 21, 23, 25, 27, 30, 32, 34, 36]);
const wheelOrder = [0, 32, 15, 19, 4, 21, 2, 25, 17, 34, 6, 27, 13, 36, 11, 30, 8, 23, 10, 5, 24, 16, 33, 1, 20, 14, 31, 9, 22, 18, 29, 7, 28, 12, 35, 3, 26];

test('roulette selects bets and stake sizes while keeping other saved games unchanged', async ({ page }) => {
  await openRoulette(page);
  const before = stable(await save(page));
  expect((await view(page)).stake).toBe(25);
  expect((await view(page)).bet).toBe('red');

  await clickAction(page, 'rouletteBetNumber17');
  expect((await view(page)).bet).toEqual({ number: 17 });
  await clickAction(page, 'rouletteBetBlack');
  expect((await view(page)).bet).toBe('black');
  await clickAction(page, 'rouletteBetOdd');
  expect((await view(page)).bet).toBe('odd');
  await clickAction(page, 'rouletteBetLow');
  expect((await view(page)).bet).toBe('low');
  await clickAction(page, 'rouletteBetDozen2');
  expect((await view(page)).bet).toBe('dozen2');

  await clickAction(page, 'rouletteStakeUp');
  expect((await view(page)).stake).toBe(30);
  await clickAction(page, 'rouletteStakeDown');
  expect((await view(page)).stake).toBe(25);
  await clickAction(page, 'rouletteStakeMin');
  const min = (await view(page)).stake;
  await clickAction(page, 'rouletteStakeMax');
  expect((await view(page)).stake).toBeGreaterThan(min);
  await expect.poll(async () => Boolean((await actions(page)).rouletteSpin)).toBe(true);

  const after = stable(await save(page));
  const { roulette_balance: _balance, ...beforeGames } = before;
  const { roulette_balance: _newBalance, ...afterGames } = after;
  expect(afterGames).toEqual(beforeGames);
});

test('roulette settles one red wager once, even after reload', async ({ page }) => {
  test.setTimeout(30_000);
  await openRoulette(page);
  const startingBalance = (await view(page)).bankroll;
  expect(startingBalance).toBe(1000);
  expect((await view(page)).bet).toBe('red');
  expect((await view(page)).stake).toBe(25);

  await clickAction(page, 'rouletteSpin');
  await expect.poll(async () => (await view(page)).spinning).toBe(true);
  const inFlight = await view(page);
  expect(Number.isFinite(inFlight.stripPosition)).toBe(true);
  expect(Number.isFinite(inFlight.stripTarget)).toBe(true);
  const balanceAtStart = (await view(page)).bankroll;
  expect([startingBalance - 25, startingBalance + 25]).toContain(balanceAtStart);
  expect((await save(page)).roulette_balance).toBe(balanceAtStart);
  expect((await view(page)).displayBalance).toBe(startingBalance - 25);
  await expect.poll(async () => (await view(page)).result, { timeout: 8_000 }).not.toBeNull();

  const settled = await view(page);
  const result = settled.result;
  expect(settled.spinning).toBe(false);
  expect(result.stake).toBe(25);
  expect(result.bet).toBe('red');
  expect(result.number).toBeGreaterThanOrEqual(0);
  expect(result.number).toBeLessThanOrEqual(36);
  expect(result.won).toBe(redNumbers.has(result.number));
  expect(result.payout).toBe(result.won ? 50 : 0);
  expect(result.net).toBe(result.payout - result.stake);
  expect(settled.bankroll).toBe(startingBalance - 25 + result.payout);
  expect(settled.bankroll).toBe(balanceAtStart);
  expect(result.balance).toBe(settled.bankroll);
  expect(Number.isFinite(settled.stripPosition)).toBe(true);
  expect(Number.isFinite(settled.stripTarget)).toBe(true);
  expect(Math.round(settled.stripTarget) % 37).toBe(wheelOrder.indexOf(result.number));
  expect((await save(page)).roulette_balance).toBe(settled.bankroll);

  await page.reload();
  await expect.poll(async () => page.locator(canvas).evaluate((el) => el.dataset.ready === 'true'), { timeout: 30_000 }).toBe(true);
  await clickLobbyAction(page, 'roulette');
  await expect.poll(async () => (await interaction(page)).screen).toBe('Roulette');
  await expect.poll(async () => (await view(page)).bankroll).toBe(settled.bankroll);
  expect((await save(page)).roulette_balance).toBe(settled.bankroll);
  await page.reload();
  await expect.poll(async () => page.locator(canvas).evaluate((el) => el.dataset.ready === 'true'), { timeout: 30_000 }).toBe(true);
  await clickLobbyAction(page, 'roulette');
  await expect.poll(async () => (await interaction(page)).screen).toBe('Roulette');
  await expect.poll(async () => (await view(page)).bankroll).toBe(settled.bankroll);
  expect((await save(page)).roulette_balance).toBe(settled.bankroll);
});

test('roulette rules pause the result timer while open', async ({ page }) => {
  test.setTimeout(20_000);
  await openRoulette(page);
  await clickAction(page, 'rouletteSpin');
  await expect.poll(async () => (await view(page)).spinning).toBe(true);
  await clickAction(page, 'rouletteRules');
  await expect.poll(async () => (await view(page)).rulesOpen).toBe(true);
  await page.waitForTimeout(2_900);
  expect((await view(page)).spinning).toBe(true);
  expect((await view(page)).result).toBeNull();
  await clickAction(page, 'rouletteRules');
  await expect.poll(async () => (await view(page)).rulesOpen).toBe(false);
  await expect.poll(async () => (await view(page)).result, { timeout: 8_000 }).not.toBeNull();
});

test('roulette desktop table keeps the reference number, column, dozen, and outside-bet geometry', async ({ page }) => {
  await openRoulette(page, { width: 1280, height: 800 });
  const bounds = await actions(page);
  const mid = (key) => center(bounds[key]);
  const top = mid('rouletteBetNumber3');
  const middle = mid('rouletteBetNumber2');
  const bottom = mid('rouletteBetNumber1');
  expect(top.y).toBeLessThan(middle.y);
  expect(middle.y).toBeLessThan(bottom.y);
  expect(mid('rouletteBetNumber6').x).toBeGreaterThan(top.x);
  expect(mid('rouletteBetNumber5').x).toBeGreaterThan(middle.x);
  expect(mid('rouletteBetNumber4').x).toBeGreaterThan(bottom.x);
  for (let column = 1; column <= 3; column++) {
    expect(bounds[`rouletteBetColumn${column}`]).toHaveLength(4);
  }
  expect(mid('rouletteBetColumn3').x).toBeCloseTo(mid('rouletteBetColumn2').x, 0);
  expect(mid('rouletteBetColumn2').x).toBeCloseTo(mid('rouletteBetColumn1').x, 0);
  for (let column=1; column<=3; column++) {
    await clickAction(page,`rouletteBetColumn${column}`);
    expect((await view(page)).bet).toBe(`column${column}`);
  }
  expect(mid('rouletteBetColumn3').y).toBeCloseTo(top.y, 0);
  expect(mid('rouletteBetColumn2').y).toBeCloseTo(middle.y, 0);
  expect(mid('rouletteBetColumn1').y).toBeCloseTo(bottom.y, 0);
  expect(mid('rouletteBetDozen1').x).toBeLessThan(mid('rouletteBetDozen2').x);
  expect(mid('rouletteBetDozen2').x).toBeLessThan(mid('rouletteBetDozen3').x);
  expect(mid('rouletteBetDozen1').y).toBeLessThan(mid('rouletteBetLow').y);
  const outside = ['rouletteBetLow', 'rouletteBetEven', 'rouletteBetRed', 'rouletteBetBlack', 'rouletteBetOdd', 'rouletteBetHigh'];
  for (let i = 1; i < outside.length; i++) expect(mid(outside[i]).x).toBeGreaterThan(mid(outside[i - 1]).x);
  for (const key of ['rouletteBetNumber36', 'rouletteBetColumn1', 'rouletteBetColumn2', 'rouletteBetColumn3',
    'rouletteBetDozen1', 'rouletteBetDozen2', 'rouletteBetDozen3', ...outside]) {
    expect(bounds[key], `${key} should be present on desktop`).toHaveLength(4);
  }
});

test('roulette phone table pans horizontally for far and middle bets without selecting during a swipe', async ({ page }) => {
  test.setTimeout(30_000);
  for (const viewport of [{ width: 390, height: 740 }, { width: 360, height: 560 }]) {
    await openRoulette(page, viewport);
    const initial = await view(page);
    expect(initial.tablePan).toBe(0);
    expect(initial.maxPan).toBeGreaterThan(0);
    expect(initial.tableClip).toHaveLength(4);
    const startingBalance = initial.bankroll;
    const initialBet = initial.bet;

    await swipeTable(page, 'left');
    await expect.poll(async () => (await view(page)).tablePan).toBeGreaterThan(0);
    await expect.poll(async () => Array.isArray((await actions(page)).rouletteBetNumber36)).toBe(true);
    expect((await view(page)).bet).toEqual(initialBet);
    expect((await view(page)).bankroll).toBe(startingBalance);
    expect((await actions(page)).rouletteBetColumn3).toHaveLength(4);
    await touchAction(page, 'rouletteBetColumn3');
    expect((await view(page)).bet).toBe('column3');
    await touchAction(page, 'rouletteBetNumber36');
    expect((await view(page)).bet).toEqual({ number: 36 });
    expect((await view(page)).bankroll).toBe(startingBalance);

    for (let swipe = 0; swipe < 8 && (await view(page)).tablePan > 0.5; swipe++) {
      await swipeTable(page, 'right', 0.72);
    }
    await expect.poll(async () => (await view(page)).tablePan).toBeLessThanOrEqual(0.5);
    await expect.poll(async () => Array.isArray((await actions(page)).rouletteBetNumber0)).toBe(true);
    await touchAction(page, 'rouletteBetNumber0');
    expect((await view(page)).bet).toEqual({ number: 0 });
    expect((await view(page)).bankroll).toBe(startingBalance);

    for (let swipe = 0; swipe < 6 && !Array.isArray((await actions(page)).rouletteBetRed); swipe++) {
      await swipeTable(page, 'left', 0.32);
    }
    await expect.poll(async () => Array.isArray((await actions(page)).rouletteBetRed)).toBe(true);
    expect((await view(page)).bet).toEqual({ number: 0 });
    expect((await view(page)).bankroll).toBe(startingBalance);
    await touchAction(page, 'rouletteBetRed');
    expect((await view(page)).bet).toBe('red');
    expect((await view(page)).bankroll).toBe(startingBalance);
    await touchAction(page, 'rouletteRules');
    await expect.poll(async () => (await view(page)).rulesOpen).toBe(true);
    await touchAction(page, 'rouletteRules');
    await expect.poll(async () => (await view(page)).rulesOpen).toBe(false);
  }
});
