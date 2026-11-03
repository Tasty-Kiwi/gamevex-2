import { expect } from '@playwright/test';

const gameCategories = {
  mines: 'puzzle', sudoku: 'puzzle', mahjong: 'puzzle',
  blackjack: 'highLimit', baccarat: 'highLimit', holdem: 'highLimit', roulette: 'highLimit',
  solitaire: 'solitaire', freecell: 'solitaire', spider: 'solitaire',
  hearts: 'cardAndBoard', cribbage: 'cardAndBoard', backgammon: 'cardAndBoard', connect4: 'cardAndBoard', cards: 'cardAndBoard',
};

const read = (page) => page.locator('#gamevex').evaluate(
  (el) => JSON.parse(el.dataset.actionBounds || '{}'),
);

export async function revealLobbyGame(page, key, activate) {
  await expect.poll(() => page.locator('#gamevex').evaluate(
    (el) => JSON.parse(el.dataset.interaction || '{}').screen,
  )).toBe('Lobby');
  const category = gameCategories[key];
  if (!category) throw new Error(`Unknown lobby game ${key}`);
  let bounds = await read(page);
  if (bounds.lobbyCategory !== category) {
    if (bounds.lobbyCategory) {
      await activate(bounds.categoriesBack);
      await expect.poll(async () => (await read(page)).lobbyCategory).toBeNull();
    }
    bounds = await read(page);
    await activate(bounds.categories[category]);
    await expect.poll(async () => (await read(page)).lobbyCategory).toBe(category);
  }
  for (let attempt = 0; attempt < 4; attempt++) {
    bounds = await read(page);
    if (Array.isArray(bounds[key])) return bounds[key];
    const previous = bounds.lobbyScroll;
    await page.mouse.move(190, 360);
    await page.mouse.wheel(0, previous < bounds.lobbyMaxScroll ? 420 : -420);
    await expect.poll(async () => (await read(page)).lobbyScroll).not.toBe(previous);
  }
  throw new Error(`Lobby game ${key} did not become visible`);
}
