import { expect, test } from '@playwright/test';
import { screenPoint } from './interaction.js';
import { revealLobbyGame } from './lobby.js';

const canvas = '#gamevex';
const read = (page, key) => page.locator(canvas).evaluate((el, key) => JSON.parse(el.dataset[key] || '{}'), key);
const view = page => read(page, 'connect4View');
async function click(page, bounds) {
  expect(bounds).toHaveLength(4);
  const point = await screenPoint(page, bounds[0] + bounds[2] / 2, bounds[1] + bounds[3] / 2);
  await page.mouse.click(point.x, point.y);
}
async function action(page, key) {
  await expect.poll(async () => Boolean((await read(page, 'connect4Bounds'))[key])).toBe(true);
  await click(page, (await read(page, 'connect4Bounds'))[key]);
}
async function open(page) {
  await page.goto('/?qa=fresh&diagnostics');
  await expect.poll(() => page.locator(canvas).getAttribute('data-ready'), { timeout: 30000 }).toBe('true');
  await click(page, await revealLobbyGame(page, 'connect4', bounds => click(page, bounds)));
  await expect.poll(async () => (await read(page, 'interaction')).screen).toBe('Connect4');
}

test('Connect 4 restores a legacy board as CPU-only, wins, and confirms replacement', async ({ page }) => {
  await open(page);
  expect((await read(page, 'connect4Bounds')).mode).toBeUndefined();
  const session = await read(page, 'session');
  const board = Array(42).fill(0);
  for (const row of [3, 4, 5]) { board[row * 7] = 1; board[row * 7 + 1] = 2; }
  session.connect4 = { board, turn: 'Red', mode: 'TwoPlayers', status: 'Playing', history: [0, 1, 0, 1, 0, 1] };
  await page.addInitScript(session => {
    if (!sessionStorage.getItem('connect4-legacy')) {
      localStorage.setItem('gamevex.rust-demo.qa.v1', JSON.stringify(session));
      sessionStorage.setItem('connect4-legacy', 'true');
    }
  }, session);
  await open(page);
  expect((await view(page)).mode).toBe('Computer');
  await action(page, 'new');
  await expect.poll(async () => (await read(page, 'interaction')).modal).toBe('connect4-replace');
  await action(page, 'cancel');
  expect((await view(page)).history).toEqual([0, 1, 0, 1, 0, 1]);
  await action(page, 'column0');
  await expect.poll(async () => (await view(page)).status).toEqual({ Won: 'Red' });
  await expect.poll(async () => (await view(page)).animating).toBe(false);
  await page.screenshot({ path: test.info().outputPath('connect4-win-desktop.png') });
  await open(page);
  expect((await view(page)).status).toEqual({ Won: 'Red' });
  await action(page, 'new');
  await expect.poll(async () => (await view(page)).history.length).toBe(0);
});

test('Connect 4 computer responds and keyboard input works on a narrow Paper board', async ({ page }) => {
  await page.setViewportSize({ width: 360, height: 568 });
  await open(page);
  await page.keyboard.press('4');
  await expect.poll(async () => (await view(page)).history.length).toBe(2);
  expect((await view(page)).turn).toBe('Red');
  const session = await read(page, 'session');
  session.settings.color_scheme = 'Paper';
  session.settings.reduced_motion = true;
  await page.addInitScript(session => {
    if (!sessionStorage.getItem('connect4-paper')) {
      localStorage.setItem('gamevex.rust-demo.qa.v1', JSON.stringify(session));
      sessionStorage.setItem('connect4-paper', 'true');
    }
  }, session);
  await open(page);
  expect((await read(page, 'session')).settings.color_scheme).toBe('Paper');
  await page.keyboard.press('ArrowLeft');
  await page.keyboard.press('Enter');
  await expect.poll(async () => (await view(page)).history.length).toBe(4);
  await page.screenshot({ path: test.info().outputPath('connect4-paper-mobile.png') });
  const bounds = await read(page, 'connect4Bounds');
  for (const b of Object.values(bounds)) {
    expect(b[0]).toBeGreaterThanOrEqual(0);
    expect(b[1]).toBeGreaterThanOrEqual(0);
    expect(b[0] + b[2]).toBeLessThanOrEqual(360);
    expect(b[1] + b[3]).toBeLessThanOrEqual(568);
  }
});

test.describe('Connect 4 touch controls', () => {
  test.use({ hasTouch: true, viewport: { width: 390, height: 844 } });
  test('a column tap plays against the CPU and new game requires confirmation', async ({ page }) => {
    await open(page);
    expect((await read(page, 'connect4Bounds')).mode).toBeUndefined();
    const b = (await read(page, 'connect4Bounds')).column2;
    const point = await screenPoint(page, b[0] + b[2] / 2, b[1] + b[3] / 2);
    await page.touchscreen.tap(point.x, point.y);
    await expect.poll(async () => (await view(page)).history.length).toBe(2);
    expect((await view(page)).history[0]).toBe(2);
    await action(page, 'new');
    await expect.poll(async () => (await read(page, 'interaction')).modal).toBe('connect4-replace');
    await action(page, 'confirm');
    await expect.poll(async () => (await view(page)).mode).toBe('Computer');
    await expect.poll(async () => (await view(page)).history).toEqual([]);
  });
});
