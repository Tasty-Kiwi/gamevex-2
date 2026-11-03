import { expect, test } from '@playwright/test';
import { screenPoint } from './interaction.js';
import { revealLobbyGame } from './lobby.js';

const read = (page, key) => page.locator('#gamevex').evaluate((el, key) => JSON.parse(el.dataset[key] || '{}'), key);
const games = [{ key: 'connect4', screen: 'Connect4', move: 'column0', sibling: 'tictactoe' },
  { key: 'tictactoe', screen: 'TicTacToe', move: 'cell0', sibling: 'connect4' }];
async function point(page, b, fraction = 0.5) {
  return screenPoint(page, b[0] + b[2] * fraction, b[1] + b[3] / 2);
}
async function click(page, b, fraction) {
  const p = await point(page, b, fraction);
  await page.mouse.click(p.x, p.y);
}
async function open(page, game) {
  await page.goto('/?qa=fresh&diagnostics');
  await expect.poll(() => page.locator('#gamevex').getAttribute('data-ready'), { timeout: 30000 }).toBe('true');
  await click(page, await revealLobbyGame(page, game.key, b => click(page, b)));
  await expect.poll(async () => (await read(page, 'interaction')).screen).toBe(game.screen);
}
const bounds = (page, game) => read(page, game.key + 'Bounds');
const view = (page, game) => read(page, game.key + 'View');
async function level(page, game, expected) {
  await expect.poll(async () => (await view(page, game)).difficulty).toBe(expected);
}
for (const game of games) {
  test(`${game.screen} difficulty buttons preserve the board and saved selection`, async ({ page }) => {
    await open(page, game);
    await click(page, (await bounds(page, game))[game.move]);
    await expect.poll(async () => (await view(page, game)).history.length).toBe(2);
    const before = await view(page, game);
    const sibling = (await read(page, 'session'))[game.sibling];
    for (const selection of [1, 3, 5, 2]) {
      await click(page, (await bounds(page, game))[`difficulty${selection}`]);
      await level(page, game, selection);
    }
    const focus = (await read(page, 'interaction')).focusIndex;
    await page.keyboard.press('Tab');
    await expect.poll(async () => (await read(page, 'interaction')).focusIndex).toBe(focus + 1);
    await page.keyboard.press('Enter');
    await level(page, game, 3);
    const after = await view(page, game);
    for (const field of ['board', 'turn', 'status', 'history']) expect(after[field]).toEqual(before[field]);
    expect((await read(page, 'session'))[game.sibling]).toEqual(sibling);
    await page.screenshot({ path: test.info().outputPath(`${game.key}-buttons-desktop.png`) });
    await open(page, game);
    await level(page, game, 3);
    await click(page, (await bounds(page, game)).new);
    await expect.poll(async () => (await read(page, 'interaction')).modal).toBe(`${game.key}-replace`);
    await click(page, (await bounds(page, game)).confirm);
    await expect.poll(async () => (await view(page, game)).history.length).toBe(0);
    await level(page, game, 3);
  });
}

test.describe('touch difficulty buttons', () => {
  test.use({ hasTouch: true, viewport: { width: 360, height: 568 } });
  for (const game of games) {
    test(`${game.screen} difficulty buttons fit and support touch and cancellation`, async ({ page }) => {
      await open(page, game);
      for (const selection of [1, 4]) {
        const p = await point(page, (await bounds(page, game))[`difficulty${selection}`]);
        await page.touchscreen.tap(p.x, p.y);
        await level(page, game, selection);
      }
      const b = (await bounds(page, game)).difficulty2;
      const p = await point(page, b);
      const cdp = await page.context().newCDPSession(page);
      await cdp.send('Input.dispatchTouchEvent', { type: 'touchStart', touchPoints: [{ x: p.x, y: p.y, id: 1 }] });
      await cdp.send('Input.dispatchTouchEvent', { type: 'touchCancel', touchPoints: [] });
      await level(page, game, 4);
      expect((await view(page, game)).history).toEqual([]);
      for (const b of Object.values(await bounds(page, game))) {
        expect(b[0]).toBeGreaterThanOrEqual(0);
        expect(b[0] + b[2]).toBeLessThanOrEqual(360);
        expect(b[1] + b[3]).toBeLessThanOrEqual(568);
      }
      await page.screenshot({ path: test.info().outputPath(`${game.key}-buttons-mobile.png`) });
      await cdp.detach();
    });
  }
});
