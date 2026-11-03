import { screenPoint } from './interaction.js';
import { revealLobbyGame } from './lobby.js';
import { expect, test } from '@playwright/test';

const canvas = '#gamevex';

function center(bounds) {
  return { x: bounds[0] + bounds[2] / 2, y: bounds[1] + bounds[3] / 2 };
}

async function clickSurface(page, x, y) {
  const point = await screenPoint(page, x, y);
  await page.mouse.click(point.x, point.y);
}

async function touchSurface(page, x, y) {
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
    await cdp.detach();
  }
}

async function interaction(page) {
  return page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.interaction || '{}'));
}

async function save(page) {
  return page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.session));
}

async function mahjongView(page) {
  return page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.mahjongView));
}

async function actions(page) {
  return page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.actionBounds || '{}'));
}

async function tableFeedback(page) {
  return page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.tableFeedback || '{"events":[]}'));
}

async function audioStatus(page) {
  return page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.audio || '{}'));
}

async function clickAction(page, key) {
  await expect.poll(async () => {
    const bounds = (await actions(page))[key];
    return Array.isArray(bounds) && bounds.length === 4;
  }).toBe(true);
  const p = center((await actions(page))[key]);
  await clickSurface(page, p.x, p.y);
}

async function touchAction(page, key) {
  await expect.poll(async () => {
    const bounds = (await actions(page))[key];
    return Array.isArray(bounds) && bounds.length === 4;
  }).toBe(true);
  const p = center((await actions(page))[key]);
  await touchSurface(page, p.x, p.y);
}

async function openMahjong(page) {
  await page.goto('/?qa=fresh&diagnostics');
  await expect.poll(async () => page.locator(canvas).evaluate((el) => el.dataset.ready === 'true'), {
    timeout: 30_000,
  }).toBe(true);
  await expect.poll(async () => page.locator(canvas).evaluate((el) => {
    const current = JSON.parse(el.dataset.session || '{}');
    return Boolean(current.version === 14 && current.mahjong && current.mahjong.tiles?.length === 144
      && current.solitaire && current.freecell && current.spider);
  })).toBe(true);
  await clickLobbyAction(page, 'mahjong');
  await expect.poll(async () => (await interaction(page)).screen).toBe('Mahjong');
}

async function clickLobbyAction(page, key) {
  const activate = (bounds) => {
    const { x, y } = center(bounds);
    return clickSurface(page, x, y);
  };
  await activate(await revealLobbyGame(page, key, activate));
}

function gameplay(game) {
  const { elapsed_ms, ...state } = game;
  return state;
}

function facesMatch(a, b) {
  if (a >= 34 && b >= 34) return (a < 38) === (b < 38);
  return a === b;
}

// Find a legal pair from the serialized core geometry, and use the published
// hit rectangles to choose the exact visible points used by real pointer input.
async function legalPairPoints(page) {
  return page.locator(canvas).evaluate((el) => {
    const game = JSON.parse(el.dataset.session).mahjong;
    const bounds = JSON.parse(el.dataset.actionBounds);
    const tiles = game.tiles;
    const selectable = (i) => {
      const t = tiles[i];
      if (t.removed) return false;
      const above = tiles.some((o) => !o.removed && o.z > t.z
        && Math.abs(o.x - t.x) <= 1 && Math.abs(o.y - t.y) <= 1);
      const left = tiles.some((o) => !o.removed && o.z === t.z
        && o.x - t.x >= -2 && o.x - t.x <= -1 && Math.abs(o.y - t.y) <= 1);
      const right = tiles.some((o) => !o.removed && o.z === t.z
        && o.x - t.x >= 1 && o.x - t.x <= 2 && Math.abs(o.y - t.y) <= 1);
      return !above && !(left && right);
    };
    const match = (a, b) => a >= 34 && b >= 34 ? (a < 38) === (b < 38) : a === b;
    const order = tiles.map((_, i) => i).filter((i) => !tiles[i].removed)
      .sort((a, b) => tiles[a].z - tiles[b].z || tiles[a].y - tiles[b].y || tiles[a].x - tiles[b].x);
    const position = new Map(order.map((index, at) => [index, at]));
    const safePoint = (index) => {
      const r = bounds[`tile${index}`];
      if (!Array.isArray(r)) return null;
      const later = order.slice(position.get(index) + 1)
        .map((i) => bounds[`tile${i}`]).filter(Array.isArray);
      for (const fy of [0.25, 0.5, 0.75]) {
        for (const fx of [0.25, 0.5, 0.75]) {
          const point = { x: r[0] + r[2] * fx, y: r[1] + r[3] * fy };
          if (!later.some((o) => point.x >= o[0] && point.x <= o[0] + o[2]
            && point.y >= o[1] && point.y <= o[1] + o[3])) return point;
        }
      }
      return null;
    };
    const candidates = tiles.map((_, i) => i).filter((i) => selectable(i) && safePoint(i));
    for (let ai = 0; ai < candidates.length; ai++) {
      for (let bi = ai + 1; bi < candidates.length; bi++) {
        const [a, b] = [candidates[ai], candidates[bi]];
        if (match(tiles[a].face, tiles[b].face)) {
          return { pair: [a, b], points: [safePoint(a), safePoint(b)] };
        }
      }
    }
    return null;
  });
}

async function clickPair(page, pair = null) {
  const target = pair || await legalPairPoints(page);
  expect(target, 'a selectable matching pair is visible').not.toBeNull();
  await clickSurface(page, target.points[0].x, target.points[0].y);
  await expect.poll(async () => (await mahjongView(page)).selected).toBe(target.pair[0]);
  await clickSurface(page, target.points[1].x, target.points[1].y);
  await expect.poll(async () => (await mahjongView(page)).remaining).toBe(142);
  return target;
}

test('Mahjong pair match and undo feedback plays once and does not replay after reload', async ({ page }) => {
  await openMahjong(page);
  const before = await save(page);
  const initialAudio = (await audioStatus(page)).played;
  const pair = await legalPairPoints(page);
  expect(pair).not.toBeNull();
  await clickPair(page, pair);
  let after = await save(page);
  for (const index of pair.pair) expect(after.mahjong.tiles[index].removed).toBe(true);
  expect(after.mahjong.history.at(-1)).toEqual(pair.pair);
  expect(after.minesweeper).toEqual(before.minesweeper);
  await expect.poll(async () => (await tableFeedback(page)).events.some((e) => e.kind === 'Move')).toBe(true);
  await expect.poll(async () => (await audioStatus(page)).played).toBeGreaterThan(initialAudio);

  await expect.poll(async () => (await audioStatus(page)).last).toBe('TileMatch');
  await clickAction(page, 'undo');
  await expect.poll(async () => (await mahjongView(page)).remaining).toBe(144);
  after = await save(page);
  expect(after.mahjong.history).toHaveLength(0);
  expect(after.mahjong.tiles.every((tile) => !tile.removed)).toBe(true);
  await expect.poll(async () => (await tableFeedback(page)).events.some((e) => e.kind === 'Undo')).toBe(true);
  await expect.poll(async () => (await audioStatus(page)).played).toBeGreaterThan(initialAudio + 1);

  await page.reload();
  await expect.poll(async () => (await interaction(page)).screen).toBe('Lobby');
  const afterReload = (await audioStatus(page)).played;
  await clickLobbyAction(page, 'mahjong');
  await expect.poll(async () => (await interaction(page)).screen).toBe('Mahjong');
  await expect.poll(async () => (await audioStatus(page)).played).toBe(afterReload + 2); // Category and game navigation clicks only.
  expect((await tableFeedback(page)).events).toHaveLength(0);
  await page.waitForTimeout(500);
  expect((await audioStatus(page)).played).toBe(afterReload + 2);
});

test('Mahjong Undo feedback expires across slow application frames', async ({ page }) => {
  await openMahjong(page);
  await clickPair(page);
  await expect.poll(async () => (await audioStatus(page)).last).toBe('TileMatch');
  const initialAudio = (await audioStatus(page)).played;
  await page.evaluate(() => {
    const nextFrame = window.requestAnimationFrame.bind(window);
    window.requestAnimationFrame = (callback) => nextFrame((time) => setTimeout(() => callback(time), 1_200));
  });
  await clickAction(page, 'undo');
  // Undo can render and expire before a poll under sparse frames. Its saved
  // history and cue are stable acknowledgements; the next assertion tests expiry.
  await expect.poll(async () => (await save(page)).mahjong.history).toHaveLength(0);
  await expect.poll(async () => (await audioStatus(page)).last).toBe('TableUndo');
  await expect.poll(async () => (await audioStatus(page)).played).toBeGreaterThan(initialAudio);
  await expect.poll(async () => (await tableFeedback(page)).events, { timeout: 5_000 }).toHaveLength(0);
  await expect.poll(async () => (await audioStatus(page)).voices).toBe(0);
});

test('Mahjong hint marks assistance, a legal pair can be undone, and the board is restored', async ({ page }) => {
  await openMahjong(page);
  await clickAction(page, 'hint');
  await expect.poll(async () => (await save(page)).mahjong.assisted).toBe(true);
  const hinted = (await mahjongView(page)).hint;
  expect(hinted).toHaveLength(2);
  await expect.poll(async () => {
    const bounds = await actions(page);
    return hinted.every((i) => Array.isArray(bounds[`tile${i}`]));
  }).toBe(true);
  const pair = await page.locator(canvas).evaluate((el, indices) => {
    const bounds = JSON.parse(el.dataset.actionBounds);
    return {
      pair: indices,
      points: indices.map((i) => {
        const r = bounds[`tile${i}`];
        return { x: r[0] + r[2] / 2, y: r[1] + r[3] / 2 };
      }),
    };
  }, hinted);
  await clickPair(page, pair);
  await clickAction(page, 'undo');
  await expect.poll(async () => (await mahjongView(page)).remaining).toBe(144);
  const restored = (await save(page)).mahjong;
  expect(restored.assisted).toBe(true);
  expect(restored.history).toHaveLength(0);
  expect(restored.tiles.every((tile) => !tile.removed)).toBe(true);
});

test('choosing Zen on a pristine board freezes and restores elapsed time', async ({ page }) => {
  await openMahjong(page);
  await clickAction(page, 'zen');
  await expect.poll(async () => (await save(page)).mahjong.zen).toBe(true);
  const zenGame = (await save(page)).mahjong;
  expect(zenGame.tiles.every((tile) => !tile.removed)).toBe(true);

  await page.evaluate(() => new Promise((resolve) => requestAnimationFrame(
    () => requestAnimationFrame(() => requestAnimationFrame(resolve)),
  )));
  await page.waitForTimeout(600);
  expect((await save(page)).mahjong.elapsed_ms).toBe(zenGame.elapsed_ms);

  await page.reload();
  await expect.poll(async () => (await interaction(page)).screen).toBe('Lobby');
  const restored = (await save(page)).mahjong;
  expect(restored.zen).toBe(true);
  expect(restored.elapsed_ms).toBe(zenGame.elapsed_ms);
});

test.describe('Mahjong Zen mode touch selection', () => {
  test.use({ viewport: { width: 390, height: 844 }, hasTouch: true });

  test('a real touch tap chooses Zen on an untouched board', async ({ page }) => {
    await openMahjong(page);
    await touchAction(page, 'zen');
    await expect.poll(async () => (await save(page)).mahjong.zen).toBe(true);
    expect((await save(page)).mahjong.tiles.every((tile) => !tile.removed)).toBe(true);
  });
});

test('switching a progressed Zen run to Classic can be canceled or confirmed', async ({ page }) => {
  await openMahjong(page);
  await clickAction(page, 'zen');
  await expect.poll(async () => (await save(page)).mahjong.zen).toBe(true);
  await clickPair(page);

  const before = (await save(page)).mahjong;
  await clickAction(page, 'classic');
  await expect.poll(async () => (await interaction(page)).modal).toBe('mahjong-replace');
  await clickAction(page, 'cancel');
  await expect.poll(async () => (await interaction(page)).modal).not.toBe('mahjong-replace');
  expect(gameplay((await save(page)).mahjong)).toEqual(gameplay(before));

  await clickAction(page, 'classic');
  await expect.poll(async () => (await interaction(page)).modal).toBe('mahjong-replace');
  await clickAction(page, 'confirm');
  await expect.poll(async () => {
    const game = (await save(page)).mahjong;
    return !game.zen && game.tiles.every((tile) => !tile.removed);
  }).toBe(true);
  const classic = (await save(page)).mahjong;
  expect(classic.zen).toBe(false);
  expect(classic.tiles.every((tile) => !tile.removed)).toBe(true);
  expect(classic.history).toHaveLength(0);
  expect(classic.elapsed_ms).toBe(0);
  expect(classic.seed).not.toBe(before.seed);
});

test('Mahjong progress reloads exactly apart from its running elapsed clock', async ({ page }) => {
  await openMahjong(page);
  await clickPair(page);
  const before = await save(page);
  await page.reload();
  await expect.poll(async () => (await interaction(page)).screen).toBe('Lobby');
  const after = await save(page);
  expect(gameplay(after.mahjong)).toEqual(gameplay(before.mahjong));
  expect(after.mahjong_records).toEqual(before.mahjong_records);
  expect(after.minesweeper).toEqual(before.minesweeper);
  expect(after.blackjack).toEqual(before.blackjack);
  expect(after.baccarat).toEqual(before.baccarat);
  expect(after.sudoku).toEqual(before.sudoku);

  await clickLobbyAction(page, 'mahjong');
  await expect.poll(async () => (await interaction(page)).screen).toBe('Mahjong');
  expect(gameplay((await save(page)).mahjong)).toEqual(gameplay(before.mahjong));
});

test('canceling a Mahjong replacement preserves progress in every cabinet', async ({ page }) => {
  await openMahjong(page);
  await clickPair(page);
  const before = await save(page);
  const mahjongBefore = gameplay(before.mahjong);
  await clickAction(page, 'newarena');
  await expect.poll(async () => (await interaction(page)).modal).toBe('mahjong-replace');
  await clickAction(page, 'cancel');
  await expect.poll(async () => (await interaction(page)).modal).not.toBe('mahjong-replace');
  const after = await save(page);
  expect(gameplay(after.mahjong)).toEqual(mahjongBefore);
  expect(after.mahjong_records).toEqual(before.mahjong_records);
  expect(after.minesweeper).toEqual(before.minesweeper);
  expect(after.blackjack).toEqual(before.blackjack);
  expect(after.baccarat).toEqual(before.baccarat);
  expect(after.sudoku).toEqual(before.sudoku);
});

test.describe('Mahjong phone touch controls', () => {
  test.use({ viewport: { width: 390, height: 844 }, hasTouch: true });

  test('all layouts expose in-bounds controls and a board swipe pans the tiles', async ({ page }) => {
    await openMahjong(page);
    for (const layout of ['turtle', 'arena', 'monument', 'turtle']) {
      await expect.poll(async () => (await actions(page))[`new${layout}`]).toBeTruthy();
      const data = await page.locator(canvas).evaluate((el) => ({
        bounds: JSON.parse(el.dataset.actionBounds),
        viewport: JSON.parse(el.dataset.interaction).viewport,
      }));
      for (const key of ['hint', 'restart', 'zen', 'classic', 'newturtle', 'newarena', 'newmonument']) {
        const rect = data.bounds[key];
        expect(rect, `${key} is visible on phone`).toHaveLength(4);
        expect(rect[0]).toBeGreaterThanOrEqual(0);
        expect(rect[1]).toBeGreaterThanOrEqual(0);
        expect(rect[0] + rect[2]).toBeLessThanOrEqual(data.viewport[0] + 1);
        expect(rect[1] + rect[3]).toBeLessThanOrEqual(data.viewport[1] + 1);
      }
      expect(data.bounds.undo, 'undo is absent while the pristine board has no history').toBeUndefined();
      expect(data.bounds.shuffle, 'shuffle is absent while the board has available moves').toBeUndefined();
      const target = center(data.bounds[`new${layout}`]);
      await touchSurface(page, target.x, target.y);
      await expect.poll(async () => (await save(page)).mahjong.layout).toBe(layout);
    }

    const tile = await page.locator(canvas).evaluate((el) => {
      const game = JSON.parse(el.dataset.session).mahjong;
      const bounds = JSON.parse(el.dataset.actionBounds);
      const selectable = (i) => {
        const t = game.tiles[i];
        if (t.removed) return false;
        const above = game.tiles.some((o) => !o.removed && o.z > t.z
          && Math.abs(o.x - t.x) <= 1 && Math.abs(o.y - t.y) <= 1);
        const left = game.tiles.some((o) => !o.removed && o.z === t.z
          && o.x - t.x >= -2 && o.x - t.x <= -1 && Math.abs(o.y - t.y) <= 1);
        const right = game.tiles.some((o) => !o.removed && o.z === t.z
          && o.x - t.x >= 1 && o.x - t.x <= 2 && Math.abs(o.y - t.y) <= 1);
        return !above && !(left && right);
      };
      const index = game.tiles.findIndex((_, i) => selectable(i) && Array.isArray(bounds[`tile${i}`]));
      if (index < 0) return null;
      const r = bounds[`tile${index}`];
      return { index, point: { x: r[0] + r[2] / 2, y: r[1] + r[3] / 2 } };
    });
    expect(tile, 'at least one visible selectable tile is published').not.toBeNull();
    await touchSurface(page, tile.point.x, tile.point.y);
    await expect.poll(async () => (await mahjongView(page)).selected).toBe(tile.index);

    const before = await mahjongView(page);
    const clip = before.clip;
    const maxPan = before.maxPan;
    expect(clip).toHaveLength(4);
    expect(maxPan[0] > before.pan[0] || maxPan[1] > before.pan[1]).toBe(true);
    const boardCenter = { x: clip[0] + clip[2] / 2, y: clip[1] + clip[3] / 2 };
    const axis = maxPan[0] > before.pan[0] ? 'x' : 'y';
    const target = { ...boardCenter, [axis]: boardCenter[axis] - 70 };
    await dragTouch(page, boardCenter, target);
    await expect.poll(async () => (await mahjongView(page)).pan).not.toEqual(before.pan);
  });
});
