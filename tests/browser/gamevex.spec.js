import { screenPoint } from './interaction.js';
import { revealLobbyGame } from './lobby.js';
import { expect, test } from '@playwright/test';

const canvas = '#gamevex';

async function openReady(page) {
  await page.goto('/?qa=fresh&diagnostics');
  const status = page.locator('#status');
  await expect.poll(async () => status.evaluate((el) => {
    if (el.classList.contains('error')) return `ERROR:${el.textContent}`;
    return document.querySelector('#gamevex').dataset.ready === 'true' ? 'READY' : null;
  }), { timeout: 30_000 }).not.toBeNull();
  const state = await status.evaluate((el) => ({
    ready: document.querySelector('#gamevex').dataset.ready === 'true',
    error: el.classList.contains('error') ? el.textContent : null,
  }));
  if (state.error) {
    throw new Error(`WebGPU browser harness unavailable: ${state.error}`);
  }
  expect(state.ready).toBe(true);
  await expect.poll(async () => page.locator(canvas).evaluate((el) => {
    if (!el.dataset.session) return null;
    const save = JSON.parse(el.dataset.session);
    return Boolean(save.version === 12 && save.mahjong && save.minesweeper && save.blackjack && save.baccarat && save.sudoku
      && save.solitaire && save.freecell && save.spider);
  })).toBe(true);
}

async function clickLobbyAction(page, key, touch = false) {
  const activate = (bounds) => {
    const { x, y } = center(bounds);
    return touch ? touchSurface(page, x, y) : clickSurface(page, x, y);
  };
  await activate(await revealLobbyGame(page, key, activate));
}

async function enterCards(page) {
  await openReady(page);
  await clickLobbyAction(page, 'cards');
  await expect(page.locator(canvas)).toHaveAttribute('data-interaction', /"screen":"Cards"/);
}

async function interaction(page) {
  return page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.interaction || '{}'));
}

async function blackjackSave(page) {
  return page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.session).blackjack);
}

async function clickBlackjackAction(page, action) {
  await expect.poll(async () => page.locator(canvas).evaluate((el, action) => {
    const bounds = JSON.parse(el.dataset.actionBounds);
    return Array.isArray(bounds[action]) && bounds[action].length === 4;
  }, action)).toBe(true);
  const bounds = await page.locator(canvas).evaluate((el, action) => JSON.parse(el.dataset.actionBounds)[action], action);
  const p = center(bounds);
  await clickSurface(page, p.x, p.y);
}

async function settledBounds(page) {
  let previous;
  await expect.poll(async () => {
    const current = await page.locator(canvas).evaluate(async (el) => {
      await new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)));
      return JSON.parse(el.dataset.cardBounds);
    });
    const stable = previous && current.every((r, i) => r.every((v, j) => Math.abs(v - previous[i][j]) < 0.25));
    previous = current;
    return Boolean(stable);
  }, { timeout: 12_000 }).toBe(true);
  return previous;
}

async function clickSurface(page, x, y, button = 'left') {
  const point = await screenPoint(page, x, y);
  await page.mouse.click(point.x, point.y, { button });
}

async function touchSurface(page, x, y) {
  const point = await screenPoint(page, x, y);
  await page.touchscreen.tap(point.x, point.y);
}

async function touchSudokuAction(page, action) {
  await expect.poll(async () => page.locator(canvas).evaluate((el, action) => {
    const bounds = JSON.parse(el.dataset.actionBounds || '{}');
    return Array.isArray(bounds[action]) && bounds[action].length === 4;
  }, action)).toBe(true);
  const bounds = await page.locator(canvas).evaluate((el, action) => JSON.parse(el.dataset.actionBounds)[action], action);
  const p = center(bounds);
  await touchSurface(page, p.x, p.y);
}

async function dragTouch(page, from, to) {
  const start = await screenPoint(page, from.x, from.y);
  const end = await screenPoint(page, to.x, to.y);
  const cdp = await page.context().newCDPSession(page);
  const touch = (point) => ({
    x: point.x,
    y: point.y,
    id: 1,
    radiusX: 2,
    radiusY: 2,
    force: 1,
  });
  try {
    await cdp.send('Input.dispatchTouchEvent', {
      type: 'touchStart',
      touchPoints: [touch(start)],
    });
    for (let step = 1; step <= 5; step++) {
      const point = {
        x: start.x + (end.x - start.x) * step / 5,
        y: start.y + (end.y - start.y) * step / 5,
      };
      await cdp.send('Input.dispatchTouchEvent', {
        type: 'touchMove',
        touchPoints: [touch(point)],
      });
    }
    await cdp.send('Input.dispatchTouchEvent', { type: 'touchEnd', touchPoints: [] });
  } finally {
    await cdp.detach();
  }
}

async function cancelTouch(page, at) {
  const point = await screenPoint(page, at.x, at.y);
  const cdp = await page.context().newCDPSession(page);
  try {
    await cdp.send('Input.dispatchTouchEvent', {
      type: 'touchStart',
      touchPoints: [{ x: point.x, y: point.y, id: 1, radiusX: 2, radiusY: 2, force: 1 }],
    });
    await cdp.send('Input.dispatchTouchEvent', { type: 'touchCancel', touchPoints: [] });
  } finally {
    await cdp.detach();
  }
}

async function moveSurface(page, x, y, options) {
  const point = await screenPoint(page, x, y);
  await page.mouse.move(point.x, point.y, options);
}

function center(bounds) {
  return { x: bounds[0] + bounds[2] / 2, y: bounds[1] + bounds[3] / 2 };
}

function exposedCardPoint(bounds, index) {
  return { x: bounds[index][0] + bounds[index][2] * 0.2, y: bounds[index][1] + bounds[index][3] * 0.35 };
}

test('card clicks keep rapid selections ordered and CRT-aware', async ({ page }) => {
  await enterCards(page);
  const bounds = await settledBounds(page);
  const first = exposedCardPoint(bounds, 0);
  // Selecting the first card raises it above its neighbor. The far end stays
  // exposed whether or not a paint happens between these rapid clicks.
  const second = exposedCardPoint(bounds, 4);
  await clickSurface(page, first.x, first.y);
  await clickSurface(page, second.x, second.y);
  await expect.poll(async () => (await interaction(page)).selectedCard).toBe(4);
});

test('right click flips a card without opening the browser context menu', async ({ page }) => {
  await enterCards(page);
  await page.locator(canvas).evaluate((el) => el.addEventListener('contextmenu', (event) => {
    window.__unpreventedContextMenus = (window.__unpreventedContextMenus || 0) + Number(!event.defaultPrevented);
  }));
  const bounds = await settledBounds(page);
  const p = exposedCardPoint(bounds, 4);
  await clickSurface(page, p.x, p.y, 'right');
  await expect.poll(async () => (await interaction(page)).flipped[4]).toBe(true);
  const menus = await page.evaluate(() => window.__unpreventedContextMenus || 0);
  expect(menus).toBe(0);
});

test('a card can be dragged into an empty slot and an occupied slot rejects another card', async ({ page }) => {
  await enterCards(page);
  const c = page.locator(canvas);
  const cards = await settledBounds(page);
  const slots = await c.evaluate((el) => JSON.parse(el.dataset.slotBounds));
  const from = exposedCardPoint(cards, 0);
  const slot = center(slots[0]);
  await moveSurface(page, from.x, from.y);
  await page.mouse.down();
  await moveSurface(page, slot.x, slot.y, { steps: 5 });
  await page.mouse.up();
  await expect.poll(async () => (await interaction(page)).slots[0]).toBe(0);
  const other = exposedCardPoint(cards, 1);
  await moveSurface(page, other.x, other.y);
  await page.mouse.down();
  await moveSurface(page, slot.x, slot.y, { steps: 5 });
  await page.mouse.up();
  await expect.poll(async () => (await interaction(page)).slots[0]).toBe(0);
  expect((await interaction(page)).slots).toEqual([0, null, null]);
});

test('resizing during a card drag cancels the drag without placing it', async ({ page }) => {
  await enterCards(page);
  const c = page.locator(canvas);
  const card = exposedCardPoint(await settledBounds(page), 0);
  await moveSurface(page, card.x, card.y);
  await page.mouse.down();
  await expect.poll(async () => (await interaction(page)).dragging).toBe(0);
  await page.setViewportSize({ width: 1180, height: 820 });
  await expect.poll(async () => (await interaction(page)).dragging).toBe(null);
  expect((await interaction(page)).slots).toEqual([null, null, null]);
});

test('Expert mines board supports horizontal pan on a narrow viewport', async ({ page }) => {
  await page.setViewportSize({ width: 640, height: 760 });
  await openReady(page);
  await clickLobbyAction(page, 'mines');
  await expect(page.locator(canvas)).toHaveAttribute('data-interaction', /"screen":"Mines"/);
  // Expert is the third difficulty control in the mines header.
  const bounds = await page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.difficultyBounds));
  const expert = center(bounds[2]);
  await clickSurface(page, expert.x, expert.y);
  await expect.poll(async () => (await interaction(page)).difficulty).toBe('Expert');
  const before = (await interaction(page)).pan[0];
  await page.mouse.move(320, 420);
  await page.mouse.wheel(160, 0);
  await expect.poll(async () => (await interaction(page)).pan[0]).not.toBe(before);
});

test('Blackjack deals one hand, resolves it, and restores the round after reload', async ({ page }) => {
  await openReady(page);
  await clickLobbyAction(page, 'blackjack');
  await expect.poll(async () => (await interaction(page)).screen).toBe('Blackjack');
  await expect.poll(async () => (await interaction(page)).phase).toBe('Betting');

  const initial = await blackjackSave(page);
  expect(initial.hand_count).toBe(1);
  await clickBlackjackAction(page, 'deal');
  await expect.poll(async () => (await blackjackSave(page)).phase).not.toBe('betting');

  const dealt = await blackjackSave(page);
  await page.reload();
  await expect.poll(async () => (await interaction(page)).screen).toBe('Lobby');
  expect(await blackjackSave(page)).toEqual(dealt);
  await clickLobbyAction(page, 'blackjack');
  await expect.poll(async () => (await interaction(page)).screen).toBe('Blackjack');

  // Diagnostics use PascalCase phases; the persisted save uses snake_case.
  if ((await interaction(page)).phase === 'Insurance') {
    await clickBlackjackAction(page, 'decline');
    await expect.poll(async () => (await interaction(page)).phase).not.toBe('Insurance');
  }
  if ((await interaction(page)).phase === 'PlayerTurn') {
    await clickBlackjackAction(page, 'stand');
    await expect.poll(async () => (await interaction(page)).phase).toBe('Settled');
  }

  const beforeReload = await blackjackSave(page);
  expect(beforeReload.hands).toHaveLength(1);
  expect(beforeReload.dealer.length).toBeGreaterThan(0);
  expect(beforeReload.phase).toBe('settled');

  await page.reload();
  await expect.poll(async () => (await interaction(page)).screen).toBe('Lobby');
  expect(await blackjackSave(page)).toEqual(beforeReload);

  await clickLobbyAction(page, 'blackjack');
  await expect.poll(async () => (await interaction(page)).screen).toBe('Blackjack');
  await expect.poll(async () => blackjackSave(page)).toEqual(beforeReload);
});

test('Baccarat settles, persists, and starts the next round with the same balance', async ({ page }) => {
  // The software WebGPU runner can take seconds per rendered frame. The
  // staged deal still has to finish and match the saved round before reload.
  if (process.env.CI) test.setTimeout(60_000);
  await openReady(page);
  await clickLobbyAction(page, 'baccarat');
  await expect.poll(async () => (await interaction(page)).screen).toBe('Baccarat');
  await expect.poll(async () => (await interaction(page)).phase).toBe('Betting');

  const clickAction = async (action) => {
    await expect.poll(async () => page.locator(canvas).evaluate((el, action) => {
      const bounds = JSON.parse(el.dataset.actionBounds);
      return Array.isArray(bounds[action]) && bounds[action].length === 4;
    }, action)).toBe(true);
    const bounds = await page.locator(canvas).evaluate((el, action) => JSON.parse(el.dataset.actionBounds)[action], action);
    const p = center(bounds);
    await clickSurface(page, p.x, p.y);
  };
  const baccaratSave = async () => page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.session).baccarat);

  await clickAction('banker');
  await expect.poll(async () => (await baccaratSave()).selected).toBe('banker');
  const beforeDeal = await baccaratSave();
  expect(beforeDeal.balance).toBe(1000);
  expect(beforeDeal.bet).toBe(10);

  await clickAction('deal');
  await expect.poll(async () => (await baccaratSave()).phase).not.toBe('betting');
  await expect.poll(async () => page.locator(canvas).evaluate((el) => {
    const view = JSON.parse(el.dataset.baccaratView);
    return !view.animating && view.playerCards === JSON.parse(el.dataset.session).baccarat.player.length
      && view.bankerCards === JSON.parse(el.dataset.session).baccarat.banker.length;
  }), { timeout: process.env.CI ? 30_000 : 8_000 }).toBe(true);

  const settled = await baccaratSave();
  expect(settled.result).toMatch(/^(player|banker|tie)$/);
  expect(settled.outcome).toMatch(/^(win|push|loss)$/);
  expect(settled.player.length).toBeGreaterThanOrEqual(2);
  expect(settled.banker.length).toBeGreaterThanOrEqual(2);
  const expectedPayout = settled.outcome === 'loss' ? 0
    : settled.outcome === 'push' ? settled.bet
      : settled.result === 'banker' ? Math.round(settled.bet * 1.95)
        : settled.result === 'player' ? settled.bet * 2 : settled.bet * 9;
  expect(settled.payout).toBe(expectedPayout);
  expect(settled.balance).toBe(beforeDeal.balance - beforeDeal.bet + expectedPayout);

  await page.reload();
  await expect.poll(async () => page.locator(canvas).evaluate((el) => {
    if (!el.dataset.session) return false;
    return JSON.parse(el.dataset.session).baccarat.phase === 'settled';
  })).toBe(true);
  expect(await baccaratSave()).toEqual(settled);
  await expect.poll(async () => (await interaction(page)).screen).toBe('Lobby');
  await clickLobbyAction(page, 'baccarat');
  await expect.poll(async () => (await interaction(page)).screen).toBe('Baccarat');
  await expect.poll(async () => (await baccaratSave()).phase).toBe('settled');
  expect(await baccaratSave()).toEqual(settled);

  await clickAction('next');
  await expect.poll(async () => (await baccaratSave()).phase).toBe('betting');
  const nextRound = await baccaratSave();
  expect(nextRound.balance).toBe(settled.balance);
  expect(nextRound.selected).toBeNull();
  expect(nextRound.player).toEqual([]);
  expect(nextRound.banker).toEqual([]);
});

for (const rate of [44_100, 48_000]) {
  test(`button beep matches native PCM at ${rate} Hz`, async ({ page }) => {
    const { readFile } = await import('node:fs/promises');
    const reference = JSON.parse(await readFile(new URL('../fixtures/native-button-beep.json', import.meta.url), 'utf8')).rates[rate];
    await page.addInitScript((rate) => {
      const Context = window.AudioContext;
      window.AudioContext = class extends Context {
        constructor() { super({ sampleRate: rate }); }
      };
      window.__qaAudioBuffers = [];
      const copy = AudioBuffer.prototype.copyToChannel;
      AudioBuffer.prototype.copyToChannel = function (samples, channel, offset = 0) {
        copy.call(this, samples, channel, offset);
        let hash = 2166136261;
        for (const byte of new Uint8Array(samples.buffer, samples.byteOffset, samples.byteLength)) {
          hash = Math.imul(hash ^ byte, 16777619) >>> 0;
        }
        // Bevy also keeps a silent stereo output stream alive. Capture the
        // app's mono cue buffers, not that backend's 2048-frame stream blocks.
        if (this.numberOfChannels === 1) window.__qaAudioBuffers.push({ rate: this.sampleRate, length: this.length, hash });
      };
    }, rate);
    await enterCards(page);
    await page.keyboard.press('Escape');
    await expect.poll(async () => (await interaction(page)).screen).toBe('Lobby');
    await expect.poll(async () => page.evaluate(() => window.__qaAudioBuffers.length)).toBeGreaterThan(0);
    const buffers = await page.evaluate(() => window.__qaAudioBuffers);
    for (const buffer of buffers) expect(buffer).toEqual({ rate, ...reference });
  });
}

test('casino audio starts on a gesture, caches cues, and releases finished voices', async ({ page }) => {
  await openReady(page);
  const c = page.locator(canvas);
  const audio = () => c.evaluate((el) => el.dataset.audio ? JSON.parse(el.dataset.audio) : null);
  await expect.poll(async () => (await audio())?.played).toBe(0);
  expect((await audio()).backend).toBe('WebAudio');
  expect((await audio()).running).toBe(false);
  await clickLobbyAction(page, 'baccarat');
  await expect.poll(async () => (await interaction(page)).screen).toBe('Baccarat');
  for (const side of ['player', 'banker', 'tie', 'player']) {
    const bounds = await c.evaluate((el, side) => JSON.parse(el.dataset.actionBounds)[side], side);
    const p = center(bounds);
    await clickSurface(page, p.x, p.y);
    await expect.poll(async () => c.evaluate((el) => JSON.parse(el.dataset.session).baccarat.selected))
      .toBe(side);
  }
  await expect.poll(async () => (await audio()).running).toBe(true);
  await expect.poll(async () => (await audio()).played).toBeGreaterThan(0);
  await expect.poll(async () => (await audio()).voices, { timeout: 5_000 }).toBe(0);
  expect((await audio()).cached).toBeLessThanOrEqual(21);
  await page.reload();
  await expect.poll(async () => (await audio())?.played).toBe(0);
  expect((await audio()).running).toBe(false);
});

async function enterSudoku(page) {
  await openReady(page);
  await clickLobbyAction(page, 'sudoku');
  await expect.poll(async () => (await interaction(page)).screen).toBe('Sudoku');
}

async function enterSudokuTouch(page) {
  await openReady(page);
  await clickLobbyAction(page, 'sudoku', true);
  await expect.poll(async () => (await interaction(page)).screen).toBe('Sudoku');
}

async function sudokuView(page) {
  return page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.sudokuView));
}

async function sudokuSave(page) {
  return page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.session).sudoku);
}

function sudokuGameplay(game) {
  const { elapsed_ms, ...state } = game;
  return state;
}

async function clickSudokuAction(page, action) {
  await expect.poll(async () => page.locator(canvas).evaluate((el, action) => {
    const bounds = JSON.parse(el.dataset.actionBounds);
    return Array.isArray(bounds[action]) && bounds[action].length === 4;
  }, action)).toBe(true);
  const bounds = await page.locator(canvas).evaluate((el, action) => JSON.parse(el.dataset.actionBounds)[action], action);
  const p = center(bounds);
  await clickSurface(page, p.x, p.y);
}

async function expectSudokuActionAbsent(page, action) {
  await expect.poll(async () => page.locator(canvas).evaluate((el, action) => {
    const bounds = JSON.parse(el.dataset.actionBounds || '{}');
    return !Array.isArray(bounds[action]) || bounds[action].length !== 4;
  }, action)).toBe(true);
}

// Start a chosen Sudoku mode through the setup modal. Replacement is only
// confirmed by callers that explicitly ask to replace an edited live board.
async function chooseSudokuMode(page, size, difficulty, { confirmReplacement = false } = {}) {
  await clickSudokuAction(page, 'setup');
  await expect.poll(async () => (await interaction(page)).modal).toBe('sudoku-setup');
  await expect.poll(async () => page.locator(canvas).evaluate((el, mode) => {
    const view = JSON.parse(el.dataset.sudokuView);
    const bounds = JSON.parse(el.dataset.actionBounds);
    return Boolean(view.setup && bounds[`size${mode.size}`] && bounds[mode.difficulty] && bounds.start && bounds.cancel);
  }, { size, difficulty })).toBe(true);
  await clickSudokuAction(page, `size${size}`);
  await clickSudokuAction(page, difficulty);
  await expect.poll(async () => (await sudokuView(page)).setup).toEqual({ size, difficulty });
  const before = (await sudokuSave(page)).game;
  await clickSudokuAction(page, 'start');

  if (confirmReplacement) {
    await expect.poll(async () => (await interaction(page)).modal).toBe('sudoku-replace');
    await expect.poll(async () => page.locator(canvas).evaluate((el) => {
      const bounds = JSON.parse(el.dataset.actionBounds);
      return Boolean(bounds.cancel && bounds.confirm);
    })).toBe(true);
    await clickSudokuAction(page, 'confirm');
  } else {
    await expect.poll(async () => (await interaction(page)).modal).not.toBe('sudoku-replace');
  }
  await expect.poll(async () => page.locator(canvas).evaluate((el, mode) => {
    const view = JSON.parse(el.dataset.sudokuView);
    const game = JSON.parse(el.dataset.session).sudoku.game;
    return !view.setup && game.size === mode.size && game.difficulty.toLowerCase() === mode.difficulty;
  }, { size, difficulty })).toBe(true);
  if (confirmReplacement) {
    const after = (await sudokuSave(page)).game;
    expect(sudokuGameplay(after)).not.toEqual(sudokuGameplay(before));
  }
}

async function firstVisibleEditableCell(page) {
  return page.locator(canvas).evaluate((el) => {
    const game = JSON.parse(el.dataset.session).sudoku.game;
    const view = JSON.parse(el.dataset.sudokuView);
    const [ox, oy] = view.origin;
    const [cx, cy, cw, ch] = view.clip;
    const n = game.size;
    for (let i = 0; i < n * n; i++) {
      if (game.givens[i] !== 0 || game.board[i] !== 0) continue;
      const col = i % n;
      const row = Math.floor(i / n);
      const x = ox + (col + 0.5) * view.cell;
      const y = oy + (row + 0.5) * view.cell;
      if (x >= cx && x <= cx + cw && y >= cy && y <= cy + ch) return { index: i, x, y };
    }
    return null;
  });
}

async function expectSudokuBoundsInsideViewport(page) {
  const data = await page.locator(canvas).evaluate(async (el) => {
    await new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)));
    return {
      bounds: JSON.parse(el.dataset.actionBounds),
      viewport: JSON.parse(el.dataset.interaction).viewport,
    };
  });
  for (const [name, rect] of Object.entries(data.bounds)) {
    expect(rect, `${name} has a positive rectangle`).toHaveLength(4);
    expect(rect[2], `${name} width`).toBeGreaterThan(0);
    expect(rect[3], `${name} height`).toBeGreaterThan(0);
    expect(rect[0], `${name} left`).toBeGreaterThanOrEqual(0);
    expect(rect[1], `${name} top`).toBeGreaterThanOrEqual(0);
    expect(rect[0] + rect[2], `${name} right`).toBeLessThanOrEqual(data.viewport[0] + 1);
    expect(rect[1] + rect[3], `${name} bottom`).toBeLessThanOrEqual(data.viewport[1] + 1);
  }
}

test('Sudoku pencil notes, erase, undo, and reload keep the editable cell state', async ({ page }) => {
  await enterSudoku(page);
  const target = await firstVisibleEditableCell(page);
  expect(target).not.toBeNull();
  await clickSurface(page, target.x, target.y);
  await expect.poll(async () => JSON.parse(await page.locator(canvas).getAttribute('data-sudoku-view')).selected)
    .toBe(target.index);

  await clickSudokuAction(page, 'pencil');
  await expect.poll(async () => (await sudokuView(page)).pencil).toBe(true);
  await clickSudokuAction(page, 'value1');
  await expect.poll(async () => (await sudokuSave(page)).game.notes[target.index]).toBe(1);
  await clickSudokuAction(page, 'erase');
  await expect.poll(async () => (await sudokuSave(page)).game.notes[target.index]).toBe(0);
  await clickSudokuAction(page, 'undo');
  await expect.poll(async () => (await sudokuSave(page)).game.notes[target.index]).toBe(1);

  const noteGame = (await sudokuSave(page)).game;
  await page.reload();
  await expect.poll(async () => (await interaction(page)).screen).toBe('Lobby');
  expect(sudokuGameplay((await sudokuSave(page)).game)).toEqual(sudokuGameplay(noteGame));
  await clickLobbyAction(page, 'sudoku');
  await expect.poll(async () => (await interaction(page)).screen).toBe('Sudoku');
  expect((await sudokuSave(page)).game.notes[target.index]).toBe(1);
});

test('a Sudoku hint remains assisted after reload and other games keep their saves', async ({ page }) => {
  await enterSudoku(page);
  const game = (await sudokuSave(page)).game;
  test.skip(game.size === 16, 'The experimental 16x16 mode deliberately has no hint action.');
  const minesBefore = await page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.session).minesweeper);
  const blackjackBefore = await page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.session).blackjack);
  const baccaratBefore = await page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.session).baccarat);
  await clickSudokuAction(page, 'hint');
  await expect.poll(async () => (await sudokuSave(page)).game.assisted).toBe(true);

  // Visit each neighboring game and return to the lobby without making moves.
  // The Sudoku clock may advance, so capture its persisted state afterwards.
  await page.keyboard.press('Escape');
  await expect.poll(async () => (await interaction(page)).screen).toBe('Lobby');
  for (const mode of ['mines', 'blackjack', 'baccarat']) {
    await clickLobbyAction(page, mode);
    const expectedScreen = mode[0].toUpperCase() + mode.slice(1);
    await expect.poll(async () => (await interaction(page)).screen).toBe(expectedScreen);
    await page.keyboard.press('Escape');
    await expect.poll(async () => (await interaction(page)).screen).toBe('Lobby');
  }
  const hinted = (await sudokuSave(page)).game;
  await page.reload();
  await expect.poll(async () => (await interaction(page)).screen).toBe('Lobby');
  expect(sudokuGameplay((await sudokuSave(page)).game)).toEqual(sudokuGameplay(hinted));
  const after = await page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.session));
  expect(after.minesweeper).toEqual(minesBefore);
  expect(after.blackjack).toEqual(blackjackBefore);
  expect(after.baccarat).toEqual(baccaratBefore);
});

test('canceling a Sudoku replacement protects entered values and notes', async ({ page }) => {
  await enterSudoku(page);
  const target = await firstVisibleEditableCell(page);
  expect(target).not.toBeNull();
  await clickSurface(page, target.x, target.y);
  await clickSudokuAction(page, 'value1');
  await expect.poll(async () => (await sudokuSave(page)).game.board[target.index]).toBe(1);
  const before = (await sudokuSave(page)).game;

  await clickSudokuAction(page, 'setup');
  await expect.poll(async () => (await interaction(page)).modal).toBe('sudoku-setup');
  await clickSudokuAction(page, 'size4');
  await clickSudokuAction(page, 'hard');
  await expect.poll(async () => (await sudokuView(page)).setup)
    .toEqual({ size: 4, difficulty: 'hard' });
  await clickSudokuAction(page, 'start');
  await expect.poll(async () => (await interaction(page)).modal).toBe('sudoku-replace');
  await clickSudokuAction(page, 'cancel');
  await expect.poll(async () => (await interaction(page)).screen).toBe('Sudoku');
  await expect.poll(async () => (await interaction(page)).modal).not.toBe('sudoku-replace');
  expect(sudokuGameplay((await sudokuSave(page)).game)).toEqual(sudokuGameplay(before));
});

test('16x16 Sudoku accepts A through G values and pans the clipped board', async ({ page }) => {
  await enterSudoku(page);
  await chooseSudokuMode(page, 16, 'medium');
  await expect.poll(async () => (await sudokuSave(page)).game.size).toBe(16);
  await page.setViewportSize({ width: 640, height: 760 });
  await expect.poll(async () => page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.interaction).viewport))
    .toEqual([640, 760]);
  const target = await firstVisibleEditableCell(page);
  expect(target).not.toBeNull();
  await clickSurface(page, target.x, target.y);
  await expect.poll(async () => page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.sudokuView).selected))
    .toBe(target.index);

  // Board focus is observable because arrow keys move the selected cell.
  const selected = target.index;
  const moveKey = selected % 16 < 15 ? 'ArrowRight' : 'ArrowLeft';
  const undoMoveKey = moveKey === 'ArrowRight' ? 'ArrowLeft' : 'ArrowRight';
  await page.keyboard.press(moveKey);
  await expect.poll(async () => page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.sudokuView).selected))
    .not.toBe(selected);
  await page.keyboard.press(undoMoveKey);
  await expect.poll(async () => page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.sudokuView).selected))
    .toBe(selected);
  const viewBeforePan = await page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.sudokuView));
  const wheelPoint = await screenPoint(page, viewBeforePan.clip[0] + viewBeforePan.clip[2] / 2,
    viewBeforePan.clip[1] + viewBeforePan.clip[3] / 2);
  await page.mouse.move(wheelPoint.x, wheelPoint.y);
  await page.mouse.wheel(0, 180);
  await expect.poll(async () => page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.sudokuView).origin[1]))
    .not.toBe(viewBeforePan.origin[1]);

  // Click a visible editable cell again after panning and type the extended
  // symbols on that cell; the last key proves the 16th value is reachable.
  const afterPan = await firstVisibleEditableCell(page);
  expect(afterPan).not.toBeNull();
  await clickSurface(page, afterPan.x, afterPan.y);
  await expect.poll(async () => page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.sudokuView).selected))
    .toBe(afterPan.index);
  for (const [offset, key] of ['A', 'B', 'C', 'D', 'E', 'F', 'G'].entries()) {
    const expected = offset + 10;
    await page.keyboard.press(key);
    await expect.poll(async () => (await sudokuSave(page)).game.board[afterPan.index])
      .toBe(expected);
  }
  expect((await sudokuSave(page)).game.board[afterPan.index]).toBe(16);
});

test('Sudoku controls stay inside a narrow viewport in every size mode', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 740 });
  await enterSudoku(page);
  await expect.poll(async () => page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.interaction).viewport))
    .toEqual([390, 740]);
  const sizeCycle = [4, 6, 9, 16];
  for (const size of sizeCycle) {
    const game = (await sudokuSave(page)).game;
    expect([4, 6, 9, 16]).toContain(game.size);
    await expectSudokuBoundsInsideViewport(page);
    await chooseSudokuMode(page, size, 'medium');
  }
  await expectSudokuBoundsInsideViewport(page);
});

test('Sudoku setup stages size and difficulty, then cancel preserves the entire board', async ({ page }) => {
  await enterSudoku(page);
  const target = await firstVisibleEditableCell(page);
  expect(target).not.toBeNull();
  await clickSurface(page, target.x, target.y);
  await clickSudokuAction(page, 'pencil');
  await expect.poll(async () => (await sudokuView(page)).pencil).toBe(true);
  await clickSudokuAction(page, 'value1');
  await expect.poll(async () => (await sudokuSave(page)).game.notes[target.index]).toBe(1);
  const before = (await sudokuSave(page)).game;
  await clickSudokuAction(page, 'setup');
  await expect.poll(async () => (await interaction(page)).modal).toBe('sudoku-setup');
  await clickSudokuAction(page, 'size6');
  await clickSudokuAction(page, 'expert');
  await expect.poll(async () => sudokuView(page)).toMatchObject({
    setup: { size: 6, difficulty: 'expert' },
  });
  expect(sudokuGameplay((await sudokuSave(page)).game)).toEqual(sudokuGameplay(before));
  await clickSudokuAction(page, 'cancel');
  await expect.poll(async () => (await interaction(page)).modal).not.toBe('sudoku-setup');
  expect(sudokuGameplay((await sudokuSave(page)).game)).toEqual(sudokuGameplay(before));
  expect((await sudokuView(page)).setup).toBeNull();
});

test('Sudoku setup starts the explicitly chosen mode after confirming replacement', async ({ page }) => {
  await enterSudoku(page);
  const target = await firstVisibleEditableCell(page);
  expect(target).not.toBeNull();
  await clickSurface(page, target.x, target.y);
  await clickSudokuAction(page, 'value1');
  await expect.poll(async () => (await sudokuSave(page)).game.board[target.index]).toBe(1);
  await chooseSudokuMode(page, 6, 'hard', { confirmReplacement: true });
  const game = (await sudokuSave(page)).game;
  expect(game.size).toBe(6);
  expect(game.difficulty.toLowerCase()).toBe('hard');
  expect(game.notes.every((notes) => notes === 0)).toBe(true);
});

test('Sudoku omits unavailable actions and toggles saved highlight preferences', async ({ page }) => {
  await enterSudoku(page);
  await expectSudokuActionAbsent(page, 'undo');
  const game = (await sudokuSave(page)).game;
  const given = game.givens.findIndex((value) => value !== 0);
  expect(given).toBeGreaterThanOrEqual(0);
  const n = game.size;
  const view = await sudokuView(page);
  const x = view.origin[0] + (given % n + 0.5) * view.cell;
  const y = view.origin[1] + (Math.floor(given / n) + 0.5) * view.cell;
  await clickSurface(page, x, y);
  await expect.poll(async () => (await sudokuView(page)).selected).toBe(given);
  await expectSudokuActionAbsent(page, 'erase');
  await expectSudokuActionAbsent(page, `value${game.givens[given]}`);

  const before = await page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.session).sudoku);
  await clickSudokuAction(page, 'highlights');
  await expect.poll(async () => (await sudokuView(page)).highlights).toBe(true);
  await clickSudokuAction(page, 'errors');
  await expect.poll(async () => (await sudokuSave(page)).highlight_errors).toBe(!before.highlight_errors);
  await clickSudokuAction(page, 'same');
  await expect.poll(async () => (await sudokuSave(page)).highlight_same).toBe(!before.highlight_same);
  const after = await page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.session).sudoku);
  expect(after.highlight_errors).toBe(!before.highlight_errors);
  expect(after.highlight_same).toBe(!before.highlight_same);

  await chooseSudokuMode(page, 16, 'expert');
  await expectSudokuActionAbsent(page, 'hint');
});

test('Sudoku setup modal starts on Cancel and a full Tab cycle returns before Enter cancels', async ({ page }) => {
  await enterSudoku(page);
  const before = sudokuGameplay((await sudokuSave(page)).game);
  await clickSudokuAction(page, 'setup');
  await expect.poll(async () => (await interaction(page)).modal).toBe('sudoku-setup');
  await expect.poll(async () => page.locator(canvas).evaluate((el) => {
    const bounds = JSON.parse(el.dataset.actionBounds);
    return Boolean(bounds.cancel && bounds.start);
  })).toBe(true);
  await expect.poll(async () => (await interaction(page)).focusIndex).toBe(0);
  await expect.poll(async () => (await interaction(page)).focusCount).toBe(10);
  const focusCount = (await interaction(page)).focusCount;
  for (let i = 0; i < focusCount; i++) {
    await page.keyboard.press('Tab');
    await expect.poll(async () => (await interaction(page)).focusIndex).toBe((i + 1) % focusCount);
  }
  await expect.poll(async () => (await interaction(page)).focusIndex).toBe(0);
  await page.keyboard.press('Enter');
  await expect.poll(async () => (await interaction(page)).modal).not.toBe('sudoku-setup');
  expect(sudokuGameplay((await sudokuSave(page)).game)).toEqual(before);
});

test.describe('touch input regressions', () => {
  test.use({ hasTouch: true });
  test.beforeEach(async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
  });

  test('the lobby manual opens, scrolls, and returns through its contents', async ({ page }) => {
    await page.setViewportSize({ width: 360, height: 560 });
    await openReady(page);
    const bounds = () => page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.actionBounds || '{}'));
    const tap = (rect) => {
      const { x, y } = center(rect);
      return touchSurface(page, x, y);
    };
    await tap((await bounds()).manual);
    await expect.poll(async () => (await interaction(page)).screen).toBe('Manual');
    const topics = (await bounds()).manualTopics;
    expect(Object.keys(topics)).toHaveLength(14);
    for (const [x, y, w, h] of Object.values(topics)) {
      expect(x + w).toBeLessThanOrEqual(360);
      expect(y + h).toBeLessThanOrEqual(527);
    }
    await tap(topics.Blackjack);
    await expect.poll(async () => (await bounds()).manualTopic).toBe('Blackjack');
    await page.keyboard.press('PageDown');
    await expect.poll(async () => (await bounds()).manualScroll).toBeGreaterThan(0);
    await tap((await bounds()).manualContents);
    await expect.poll(async () => (await bounds()).manualTopic).toBeNull();
    await page.keyboard.press('Escape');
    await expect.poll(async () => (await interaction(page)).screen).toBe('Lobby');
    await clickLobbyAction(page, 'blackjack', true);
    await expect.poll(async () => (await interaction(page)).screen).toBe('Blackjack');
    expect((await bounds()).manual).toBeUndefined();
  });

  test('a touch opens Sudoku from the lobby', async ({ page }) => {
    await openReady(page);
    await clickLobbyAction(page, 'sudoku', true);
    await expect.poll(async () => (await interaction(page)).screen).toBe('Sudoku');
  });

  test('touch selects a Sudoku cell and enters a note and value through the keypad', async ({ page }) => {
    await enterSudokuTouch(page);
    const target = await firstVisibleEditableCell(page);
    expect(target).not.toBeNull();
    await touchSurface(page, target.x, target.y);
    await expect.poll(async () => (await sudokuView(page)).selected).toBe(target.index);

    await touchSudokuAction(page, 'pencil');
    await expect.poll(async () => (await sudokuView(page)).pencil).toBe(true);
    await touchSudokuAction(page, 'value1');
    await expect.poll(async () => (await sudokuSave(page)).game.notes[target.index]).toBe(1);
    await touchSudokuAction(page, 'pencil');
    await expect.poll(async () => (await sudokuView(page)).pencil).toBe(false);
    await touchSudokuAction(page, 'value2');
    await expect.poll(async () => (await sudokuSave(page)).game.board[target.index]).toBe(2);
  });

  test('touch can cancel Sudoku setup without changing the saved board', async ({ page }) => {
    await enterSudokuTouch(page);
    const before = sudokuGameplay((await sudokuSave(page)).game);
    await touchSudokuAction(page, 'setup');
    await expect.poll(async () => (await interaction(page)).modal).toBe('sudoku-setup');
    await touchSudokuAction(page, 'cancel');
    await expect.poll(async () => (await interaction(page)).modal).not.toBe('sudoku-setup');
    await expect.poll(async () => (await sudokuView(page)).setup).toBeNull();
    expect(sudokuGameplay((await sudokuSave(page)).game)).toEqual(before);
  });

  test('a canceled touch on Sudoku entry does not open the game', async ({ page }) => {
    await openReady(page);
    const entry = center(await revealLobbyGame(page, 'sudoku', (bounds) => {
      const { x, y } = center(bounds);
      return touchSurface(page, x, y);
    }));
    await cancelTouch(page, entry);
    await expect.poll(async () => page.locator(canvas).evaluate(async (el) => {
      await new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)));
      return JSON.parse(el.dataset.interaction);
    })).toMatchObject({ screen: 'Lobby', modal: null });
  });

  test('a touch swipe pans the 16x16 Sudoku board without entering a value', async ({ page }) => {
    await enterSudokuTouch(page);
    await touchSudokuAction(page, 'setup');
    await expect.poll(async () => (await interaction(page)).modal).toBe('sudoku-setup');
    await touchSudokuAction(page, 'size16');
    await expect.poll(async () => (await sudokuView(page)).setup.size).toBe(16);
    await touchSudokuAction(page, 'medium');
    await expect.poll(async () => (await sudokuView(page)).setup)
      .toEqual({ size: 16, difficulty: 'medium' });
    await touchSudokuAction(page, 'start');
    await expect.poll(async () => (await sudokuSave(page)).game.size).toBe(16);
    await expect.poll(async () => (await sudokuView(page)).setup).toBeNull();

    const before = sudokuGameplay((await sudokuSave(page)).game);
    const view = await sudokuView(page);
    const start = {
      x: view.clip[0] + view.clip[2] / 2,
      y: view.clip[1] + view.clip[3] / 2,
    };
    expect(16 * view.cell).toBeGreaterThan(view.clip[3]);
    await dragTouch(page, start, { x: start.x, y: start.y - 72 });
    await expect.poll(async () => (await sudokuView(page)).origin[1]).not.toBe(view.origin[1]);
    expect(sudokuGameplay((await sudokuSave(page)).game)).toEqual(before);
  });

  test('a touch drag places a Card Lab card into an empty slot', async ({ page }) => {
    await openReady(page);
    await clickLobbyAction(page, 'cards', true);
    await expect.poll(async () => (await interaction(page)).screen).toBe('Cards');

    const cards = await settledBounds(page);
    const slots = await page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.slotBounds));
    await dragTouch(page, exposedCardPoint(cards, 0), center(slots[0]));
    await expect.poll(async () => (await interaction(page)).slots[0]).toBe(0);
  });
});

test('Baccarat selects Deal by keyboard after choosing a wager', async ({ page }) => {
  await openReady(page);
  await clickLobbyAction(page, 'baccarat');
  await expect.poll(async () => (await interaction(page)).screen).toBe('Baccarat');
  const save = () => page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.session).baccarat);
  await page.keyboard.press('p');
  await expect.poll(async () => (await save()).selected).toBe('player');
  await page.keyboard.press('Enter');
  await expect.poll(async () => (await save()).phase).toBe('settled');
  expect((await save()).bet).toBe(10);
});

test('an idle tab cannot overwrite the writer’s newer save', async ({ page, context }) => {
  await openReady(page);
  await expect(page.locator(canvas)).toHaveAttribute('data-save-writer', 'true');
  const idle = await context.newPage();
  await openReady(idle);
  await expect(idle.locator(canvas)).toHaveAttribute('data-save-writer', 'false');
  await page.bringToFront();
  await clickLobbyAction(page, 'baccarat');
  await expect.poll(async () => (await interaction(page)).screen).toBe('Baccarat');
  await page.keyboard.press('p');
  const savedSelection = () => page.evaluate(() => (
    JSON.parse(localStorage.getItem('gamevex.rust-demo.qa.v1'))?.baccarat.selected
  ));
  await expect.poll(savedSelection).toBe('player');
  await idle.evaluate(() => window.dispatchEvent(new PageTransitionEvent('pagehide')));
  expect(await savedSelection()).toBe('player');
  await page.close();
  await idle.reload();
  await expect(idle.locator(canvas)).toHaveAttribute('data-save-writer', 'true');
  await expect.poll(() => idle.locator(canvas).evaluate((el) => (
    JSON.parse(el.dataset.session || '{}').baccarat?.selected
  ))).toBe('player');
});
