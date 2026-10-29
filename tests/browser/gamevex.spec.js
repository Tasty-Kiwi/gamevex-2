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
    return Boolean(save.version === 4 && save.minesweeper && save.blackjack && save.baccarat && save.sudoku);
  })).toBe(true);
}

async function enterCards(page) {
  await openReady(page);
  const actions = await page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.actionBounds));
  const p = center(actions.cards);
  await clickSurface(page, p.x, p.y);
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

// Rust publishes logical surface bounds. Convert those to CSS points by
// inverting the same radial CRT mapping used by pointer input.
async function screenPoint(page, x, y) {
  const point = await page.locator(canvas).evaluate((el, p) => {
    const rect = el.getBoundingClientRect();
    const k = Number(el.dataset.curvature || 0);
    const qx = (p.x / rect.width) * 2 - 1;
    const qy = (p.y / rect.height) * 2 - 1;
    let lo = 0;
    const qRadius = Math.hypot(qx, qy);
    let hi = Math.max(1, qRadius);
    for (let i = 0; i < 32; i++) {
      const r = (lo + hi) / 2;
      if (r * (1 + k * r * r) < qRadius) lo = r;
      else hi = r;
    }
    const scale = qRadius ? ((lo + hi) / 2) / qRadius : 1;
    return { x: rect.left + rect.width * (1 + qx * scale) / 2, y: rect.top + rect.height * (1 + qy * scale) / 2 };
  }, { x, y });
  return point;
}

async function clickSurface(page, x, y, button = 'left') {
  const point = await screenPoint(page, x, y);
  await page.mouse.click(point.x, point.y, { button });
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
  const actions = await page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.actionBounds));
  const p = center(actions.mines);
  await clickSurface(page, p.x, p.y);
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
  const actions = await page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.actionBounds));
  const p = center(actions.blackjack);
  await clickSurface(page, p.x, p.y);
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
  const lobby = await page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.actionBounds));
  const reenter = center(lobby.blackjack);
  await clickSurface(page, reenter.x, reenter.y);
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

  const restoredActions = await page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.actionBounds));
  const blackjack = center(restoredActions.blackjack);
  await clickSurface(page, blackjack.x, blackjack.y);
  await expect.poll(async () => (await interaction(page)).screen).toBe('Blackjack');
  await expect.poll(async () => blackjackSave(page)).toEqual(beforeReload);
});

test('Baccarat settles, persists, and starts the next round with the same balance', async ({ page }) => {
  await openReady(page);
  const lobby = await page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.actionBounds));
  const baccarat = center(lobby.baccarat);
  await clickSurface(page, baccarat.x, baccarat.y);
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
  })).toBe(true);

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
  const restoredLobby = await page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.actionBounds));
  const restoredEntry = center(restoredLobby.baccarat);
  await clickSurface(page, restoredEntry.x, restoredEntry.y);
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

test('casino audio starts on a gesture, caches cues, and releases finished voices', async ({ page }) => {
  await openReady(page);
  const c = page.locator(canvas);
  const audio = () => c.evaluate((el) => el.dataset.audio ? JSON.parse(el.dataset.audio) : null);
  await expect.poll(async () => (await audio())?.played).toBe(0);
  expect((await audio()).backend).toBe('WebAudio');
  expect((await audio()).running).toBe(false);
  const lobby = await c.evaluate((el) => JSON.parse(el.dataset.actionBounds));
  const entry = center(lobby.baccarat);
  await clickSurface(page, entry.x, entry.y);
  await expect.poll(async () => (await interaction(page)).screen).toBe('Baccarat');
  for (const side of ['player', 'banker', 'tie', 'player']) {
    const bounds = await c.evaluate((el, side) => JSON.parse(el.dataset.actionBounds)[side], side);
    const p = center(bounds);
    await clickSurface(page, p.x, p.y);
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
  const lobby = await page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.actionBounds));
  const entry = center(lobby.sudoku);
  await clickSurface(page, entry.x, entry.y);
  await expect.poll(async () => (await interaction(page)).screen).toBe('Sudoku');
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
  const restored = await page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.actionBounds));
  const entry = center(restored.sudoku);
  await clickSurface(page, entry.x, entry.y);
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
    const bounds = await page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.actionBounds));
    const entry = center(bounds[mode]);
    await clickSurface(page, entry.x, entry.y);
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

  await clickSudokuAction(page, 'size');
  await page.keyboard.press('Escape');
  await expect.poll(async () => (await interaction(page)).screen).toBe('Sudoku');
  expect(sudokuGameplay((await sudokuSave(page)).game)).toEqual(sudokuGameplay(before));
});

test('16x16 Sudoku accepts A through G values and pans the clipped board', async ({ page }) => {
  await enterSudoku(page);
  for (let i = 0; i < 4; i++) {
    const game = (await sudokuSave(page)).game;
    if (game.size === 16) break;
    await clickSudokuAction(page, 'size');
    await expect.poll(async () => (await sudokuSave(page)).game.size).not.toBe(game.size);
  }
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
  for (let i = 0; i < sizeCycle.length; i++) {
    const game = (await sudokuSave(page)).game;
    expect(sizeCycle).toContain(game.size);
    await expectSudokuBoundsInsideViewport(page);
    await clickSudokuAction(page, 'size');
    const next = game.size === 4 ? 6 : game.size === 6 ? 9 : game.size === 9 ? 16 : 4;
    await expect.poll(async () => (await sudokuSave(page)).game.size).toBe(next);
  }
  await expectSudokuBoundsInsideViewport(page);
});
