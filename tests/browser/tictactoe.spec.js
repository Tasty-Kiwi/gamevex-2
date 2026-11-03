import { expect, test } from '@playwright/test';
import { screenPoint } from './interaction.js';
import { revealLobbyGame } from './lobby.js';

const canvas = '#gamevex';
const read = (page, key) => page.locator(canvas).evaluate((el, key) => JSON.parse(el.dataset[key] || '{}'), key);
const view = page => read(page, 'tictactoeView');
async function click(page, bounds) {
  expect(bounds).toHaveLength(4);
  const point = await screenPoint(page, bounds[0] + bounds[2] / 2, bounds[1] + bounds[3] / 2);
  await page.mouse.click(point.x, point.y);
}
async function action(page, key) {
  await expect.poll(async () => Boolean((await read(page, 'tictactoeBounds'))[key])).toBe(true);
  await click(page, (await read(page, 'tictactoeBounds'))[key]);
}
async function open(page) {
  await page.goto('/?qa=fresh&diagnostics');
  await expect.poll(() => page.locator(canvas).getAttribute('data-ready'), { timeout: 30000 }).toBe('true');
  await click(page, await revealLobbyGame(page, 'tictactoe', bounds => click(page, bounds)));
  await expect.poll(async () => (await read(page, 'interaction')).screen).toBe('TicTacToe');
}

test('Tic-tac-toe plays a draw, confirms replacement, and restores its board', async ({ page }) => {
  await open(page);
  const initial = await read(page, 'session');
  for (const [i, cell] of [0, 1, 6, 5, 7].entries()) {
    await action(page, `cell${cell}`);
    await expect.poll(async () => (await view(page)).history.length).toBe(Math.min(2 * (i + 1), 9));
    if (i === 0) {
      await action(page, 'new');
      await expect.poll(async () => (await read(page, 'interaction')).modal).toBe('tictactoe-replace');
      await action(page, 'cancel');
      expect((await view(page)).history).toEqual([0, 4]);
    }
  }
  await expect.poll(async () => (await view(page)).status).toBe('Draw');
  await page.screenshot({ path: test.info().outputPath('tictactoe-draw-desktop.png') });
  const saved = await view(page);
  const final = await read(page, 'session');
  for (const key of Object.keys(initial).filter(key => key !== 'tictactoe')) expect(final[key]).toEqual(initial[key]);
  await open(page);
  expect(await view(page)).toEqual(saved);
  await action(page, 'new');
  await expect.poll(async () => (await view(page)).history.length).toBe(0);
});

test('Tic-tac-toe computer responds and keyboard input works on a narrow Paper board', async ({ page }) => {
  await page.setViewportSize({ width: 360, height: 568 });
  await open(page);
  await page.keyboard.press('5');
  await expect.poll(async () => (await view(page)).history.length).toBe(2);
  expect((await view(page)).turn).toBe('Human');
  const session = await read(page, 'session');
  session.settings.color_scheme = 'Paper';
  session.settings.reduced_motion = true;
  await page.addInitScript(session => {
    if (!sessionStorage.getItem('tictactoe-paper')) {
      localStorage.setItem('gamevex.rust-demo.qa.v1', JSON.stringify(session));
      sessionStorage.setItem('tictactoe-paper', 'true');
    }
  }, session);
  await open(page);
  expect((await read(page, 'session')).settings.color_scheme).toBe('Paper');
  await page.keyboard.press('ArrowLeft');
  await page.keyboard.press('Enter');
  await expect.poll(async () => (await view(page)).history.length).toBe(4);
  await page.screenshot({ path: test.info().outputPath('tictactoe-paper-mobile.png') });
  const bounds = await read(page, 'tictactoeBounds');
  for (const b of Object.values(bounds)) {
    expect(b[0]).toBeGreaterThanOrEqual(0);
    expect(b[1]).toBeGreaterThanOrEqual(0);
    expect(b[0] + b[2]).toBeLessThanOrEqual(360);
    expect(b[1] + b[3]).toBeLessThanOrEqual(568);
  }
});

test.describe('Tic-tac-toe touch controls', () => {
  test.use({ hasTouch: true, viewport: { width: 390, height: 844 } });
  test('a cell tap plays against the CPU and new game requires confirmation', async ({ page }) => {
    await open(page);
    expect((await read(page, 'tictactoeBounds')).mode).toBeUndefined();
    const b = (await read(page, 'tictactoeBounds')).cell2;
    const point = await screenPoint(page, b[0] + b[2] / 2, b[1] + b[3] / 2);
    await page.touchscreen.tap(point.x, point.y);
    await expect.poll(async () => (await view(page)).history.length).toBe(2);
    expect((await view(page)).history[0]).toBe(2);
    await action(page, 'new');
    await expect.poll(async () => (await read(page, 'interaction')).modal).toBe('tictactoe-replace');
    await action(page, 'confirm');
    await expect.poll(async () => (await view(page)).history).toEqual([]);
  });
});
