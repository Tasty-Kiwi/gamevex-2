import { screenPoint } from './interaction.js';
import { revealLobbyGame } from './lobby.js';
import { expect, test } from '@playwright/test';

const canvas = '#gamevex';
const storageKey = 'gamevex.rust-demo.qa.v1';

function center(bounds) {
  return { x: bounds[0] + bounds[2] / 2, y: bounds[1] + bounds[3] / 2 };
}

async function clickBounds(page, bounds) {
  expect(bounds).toHaveLength(4);
  const p = center(bounds);
  const point = await screenPoint(page, p.x, p.y);
  await page.mouse.click(point.x, point.y);
  await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
}

async function touchBounds(page, bounds) {
  expect(bounds).toHaveLength(4);
  const p = center(bounds);
  const point = await screenPoint(page, p.x, p.y);
  await page.touchscreen.tap(point.x, point.y);
  await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
}

async function interaction(page) {
  return page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.interaction || '{}'));
}

async function save(page) {
  return page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.session || '{}'));
}

async function view(page) {
  return page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.backgammonView || '{}'));
}

async function actions(page) {
  return page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.actionBounds || '{}'));
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
  const activate = (bounds) => clickBounds(page, bounds);
  await activate(await revealLobbyGame(page, key, activate));
}

async function openBackgammon(page) {
  await page.goto('/?qa=fresh&diagnostics');
  const c = page.locator(canvas);
  await expect.poll(async () => c.evaluate((el) => el.dataset.ready === 'true'), { timeout: 30_000 }).toBe(true);
  await expect.poll(async () => {
    const session = await save(page);
    return session.version === 14 && Boolean(session.backgammon) && Number.isInteger(session.backgammon_wins);
  }).toBe(true);
  await clickLobbyAction(page, 'backgammon');
  await expect.poll(async () => (await interaction(page)).screen).toBe('Backgammon');
  await expect.poll(async () => (await view(page)).phase).not.toBeUndefined();
}

async function startTurn(page) {
  const current = await view(page);
  if (current.turn !== 'white' || current.phase !== 'await_roll') return current;
  await clickAction(page, 'backgammonRoll');
  await expect.poll(async () => {
    const next = await view(page);
    return next.turn !== 'white' || next.phase === 'moving';
  }).toBe(true);
  return view(page);
}

async function waitForMotion(page) {
  await expect.poll(async () => (await view(page)).animating).toBe(false);
}

async function playHumanTurn(page) {
  let state = await startTurn(page);
  for (let attempt = 0; attempt < 5 && state.phase === 'await_roll'; attempt++) state = await startTurn(page);
  const initialBoard = stable(state.board);
  while (state.turn === 'white' && state.phase === 'moving') {
    expect(state.legal.length).toBeGreaterThan(0);
    const move = state.legal[0];
    await waitForMotion(page);
    await clickAction(page, `backgammonPoint${move.from}`);
    await expect.poll(async () => (await view(page)).selected).toBe(move.from);
    await clickAction(page, `backgammonPoint${move.to}`);
    await expect.poll(async () => {
      const current = await view(page);
      return current.selected === null
        && JSON.stringify(stable(current.board)) !== JSON.stringify(stable(state.board));
    }).toBe(true);
    await waitForMotion(page);
    state = await view(page);
  }
  return { state, changed: JSON.stringify(stable(state.board)) !== JSON.stringify(initialBoard) };
}

async function seedNextLoad(page, session, marker) {
  await page.addInitScript(({ storageKey, session, marker }) => {
    if (!sessionStorage.getItem(marker)) {
      localStorage.setItem(storageKey, JSON.stringify(session));
      sessionStorage.setItem(marker, 'seeded');
    }
  }, { storageKey, session, marker });
}

test('Backgammon opens from the lobby, exposes a legal roll and preserves other games', async ({ page }) => {
  test.setTimeout(45_000);
  await openBackgammon(page);
  const before = stable(await save(page));
  expect((await view(page)).turn).toBe('white');
  expect((await view(page)).phase).toBe('await_roll');
  expect((await actions(page)).backgammonRoll).toHaveLength(4);
  await clickAction(page, 'backgammonRoll');
  await expect.poll(async () => (await view(page)).phase).toBe('moving');
  const rolled = await view(page);
  expect([2, 4]).toContain(rolled.rolled.length);
  expect(rolled.legal.length).toBeGreaterThan(0);
  const after = stable(await save(page));
  expect(after.backgammon.board).toEqual(before.backgammon.board);
  expect(after.backgammon.turn).toBe('white');
  for (const key of Object.keys(before).filter((key) => key !== 'backgammon' && key !== 'backgammon_wins')) {
    expect(after[key], key).toEqual(before[key]);
  }
});

test('Backgammon moves, lets the CPU answer, autosaves and reloads the exact board state', async ({ page }) => {
  test.setTimeout(60_000);
  await openBackgammon(page);
  const before = await view(page);
  const humanTurn = await playHumanTurn(page);
  expect(humanTurn.changed).toBe(true);
  await expect.poll(async () => {
    const current = await view(page);
    return current.turn === 'white' && current.phase === 'await_roll' && !current.animating;
  }, { timeout: 30_000 }).toBe(true);
  const progressed = stable((await save(page)).backgammon);
  expect(progressed.board).not.toEqual(before.board);
  await clickAction(page, 'lobby');
  await expect.poll(async () => (await interaction(page)).screen).toBe('Lobby');
  await page.reload();
  await expect.poll(async () => page.locator(canvas).evaluate((el) => el.dataset.ready === 'true'), { timeout: 30_000 }).toBe(true);
  expect(stable((await save(page)).backgammon)).toEqual(progressed);
  await clickLobbyAction(page, 'backgammon');
  await expect.poll(async () => stable(await view(page).then((v) => ({
    board: v.board, turn: v.turn, phase: v.phase, dice: v.dice, rolled: v.rolled,
  })))).toEqual(stable({
    board: progressed.board, turn: progressed.turn, phase: progressed.phase,
    dice: progressed.dice, rolled: progressed.rolled,
  }));
});

test('Backgammon new-game confirmation can be canceled', async ({ page }) => {
  test.setTimeout(60_000);
  await openBackgammon(page);
  const played = await playHumanTurn(page);
  expect(played.changed).toBe(true);
  await expect.poll(async () => {
    const current = await view(page);
    return current.turn === 'white' && current.phase === 'await_roll' && !current.animating;
  }, { timeout: 15_000 }).toBe(true);
  const current = stable((await save(page)).backgammon);
  await clickAction(page, 'backgammonNew');
  await expect.poll(async () => (await interaction(page)).modal).toBe('backgammon-replace');
  await clickAction(page, 'cancel');
  expect(stable((await save(page)).backgammon)).toEqual(current);
  await clickAction(page, 'backgammonNew');
  await expect.poll(async () => (await interaction(page)).modal).toBe('backgammon-replace');
  await clickAction(page, 'confirm');
  await expect.poll(async () => (await view(page)).phase).toBe('await_roll');
  expect((await save(page)).backgammon.board).not.toEqual(current.board);
});

test('Backgammon difficulty changes only after confirmation and preserves other games', async ({ page }) => {
  test.setTimeout(45_000);
  await openBackgammon(page);
  const original = stable(await save(page));
  const played = await playHumanTurn(page);
  expect(played.changed).toBe(true);
  await expect.poll(async () => {
    const current = await view(page);
    return current.turn === 'white' && current.phase === 'await_roll' && !current.animating;
  }, { timeout: 15_000 }).toBe(true);
  const progressed = stable(await save(page));
  await clickAction(page, 'backgammonDifficulty');
  await expect.poll(async () => (await interaction(page)).modal).toBe('backgammon-replace');
  await clickAction(page, 'cancel');
  expect((await save(page)).backgammon.difficulty).toBe('easy');
  await clickAction(page, 'backgammonDifficulty');
  await clickAction(page, 'confirm');
  await expect.poll(async () => (await save(page)).backgammon.difficulty).toBe('hard');
  const changed = stable(await save(page));
  expect(changed.backgammon.board).not.toEqual(progressed.backgammon.board);
  for (const key of Object.keys(original).filter((key) => key !== 'backgammon' && key !== 'backgammon_wins')) {
    expect(changed[key], key).toEqual(original[key]);
  }
});

test.describe('Backgammon phone touch controls', () => {
  test.use({ viewport: { width: 390, height: 740 }, hasTouch: true });

  test('fits the phone viewport and touch selects a source and destination once', async ({ page }) => {
    test.setTimeout(45_000);
    await openBackgammon(page);
    const data = await page.locator(canvas).evaluate((el) => ({
      viewport: JSON.parse(el.dataset.interaction || '{}').viewport,
      bounds: JSON.parse(el.dataset.actionBounds || '{}'),
    }));
    expect(data.viewport).toEqual([390, 740]);
    for (const key of ['backgammonRoll', 'backgammonNew', 'backgammonDifficulty']) {
      expect(data.bounds[key], `${key} hit bound`).toHaveLength(4);
    }
    await touchAction(page, 'backgammonRoll');
    await expect.poll(async () => (await view(page)).phase).toBe('moving');
    await waitForMotion(page);
    const state = await view(page);
    expect(state.legal.length).toBeGreaterThan(0);
    const move = state.legal[0];
    const before = (await save(page)).backgammon.board;
    await touchAction(page, `backgammonPoint${move.from}`);
    await expect.poll(async () => (await view(page)).selected).toBe(move.from);
    await touchAction(page, `backgammonPoint${move.to}`);
    await expect.poll(async () => stable((await save(page)).backgammon.board)).not.toEqual(stable(before));
    await expect.poll(async () => (await view(page)).selected).toBeNull();
  });
});

test('Backgammon near-bear-off fixture reaches a win once and restores it after reload', async ({ page }) => {
  test.setTimeout(60_000);
  await page.goto('/?qa=fresh&diagnostics');
  await expect.poll(async () => page.locator(canvas).evaluate((el) => el.dataset.ready === 'true'), { timeout: 30_000 }).toBe(true);
  const base = await save(page);
  const nearWin = structuredClone(base);
  nearWin.version = 12;
  nearWin.backgammon = {
    board: {
      points: Array.from({ length: 24 }, (_, index) => ({
        color: index === 0 ? 'white' : index === 23 ? 'black' : null,
        count: index === 0 ? 1 : index === 23 ? 15 : 0,
      })),
      bar: [0, 0],
      off: [14, 0],
    },
    turn: 'white', phase: 'moving', dice: [1], rolled: [1, 2], difficulty: 'easy',
    winner: null, elapsed_ms: 100, rng_state: 12345, turn_number: 8,
  };
  nearWin.backgammon_wins = 0;
  await seedNextLoad(page, nearWin, 'backgammon-near-win');
  await page.reload();
  await expect.poll(async () => page.locator(canvas).evaluate((el) => el.dataset.ready === 'true'), { timeout: 30_000 }).toBe(true);
  expect((await save(page)).backgammon_wins).toBe(0);
  await clickLobbyAction(page, 'backgammon');
  await expect.poll(async () => (await view(page)).phase).toBe('moving');
  expect((await view(page)).legal).toContainEqual(expect.objectContaining({ from: 0, to: 25, die: 1 }));
  await clickAction(page, 'backgammonPoint0');
  await clickAction(page, 'backgammonPoint25');
  await expect.poll(async () => (await view(page)).phase).toBe('won');
  expect((await save(page)).backgammon_wins).toBe(1);
  const won = stable((await save(page)).backgammon);
  await page.reload();
  await expect.poll(async () => page.locator(canvas).evaluate((el) => el.dataset.ready === 'true'), { timeout: 30_000 }).toBe(true);
  expect((await save(page)).backgammon_wins).toBe(1);
  await clickLobbyAction(page, 'backgammon');
  expect(stable((await save(page)).backgammon)).toEqual(won);
  expect((await save(page)).backgammon_wins).toBe(1);
});

test('Backgammon saves its running clock without another move', async ({ page }) => {
  await openBackgammon(page);
  await clickAction(page, 'backgammonRoll');
  await expect.poll(async () => (await view(page)).phase).toBe('moving');
  await expect.poll(() => page.evaluate(() => (
    JSON.parse(localStorage.getItem('gamevex.rust-demo.qa.v1')).backgammon.elapsed_ms
  )), { timeout: 20_000 }).toBeGreaterThan(0);
});
