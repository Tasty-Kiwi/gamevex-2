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
    // A test timeout can close the page while a slow renderer is still
    // acknowledging the last CDP event. Preserve that original failure.
    await cdp.detach().catch(() => {});
  }
}

async function cancelTouch(page, at) {
  const point = await screenPoint(page, at.x, at.y);
  const cdp = await page.context().newCDPSession(page);
  const touch = { x: point.x, y: point.y, id: 1, radiusX: 2, radiusY: 2, force: 1 };
  try {
    await cdp.send('Input.dispatchTouchEvent', { type: 'touchStart', touchPoints: [touch] });
    await cdp.send('Input.dispatchTouchEvent', { type: 'touchCancel', touchPoints: [] });
  } finally {
    await cdp.detach();
  }
}

async function cancelActiveTouchDrag(page, at) {
  const start = await screenPoint(page, at.x, at.y);
  const cdp = await page.context().newCDPSession(page);
  const touch = (x, y) => ({ x, y, id: 1, radiusX: 2, radiusY: 2, force: 1 });
  try {
    await cdp.send('Input.dispatchTouchEvent', { type: 'touchStart', touchPoints: [touch(start.x, start.y)] });
    await cdp.send('Input.dispatchTouchEvent', { type: 'touchMove', touchPoints: [touch(start.x + 16, start.y)] });
    await expect.poll(async () => (await view(page)).dragActive).toBe(true);
    await cdp.send('Input.dispatchTouchEvent', { type: 'touchCancel', touchPoints: [] });
    await expect.poll(async () => (await view(page)).dragSource).toBeNull();
  } finally {
    await cdp.detach();
  }
}

async function openPatience(page, family = 'Solitaire') {
  await page.goto('/?qa=fresh&diagnostics');
  const c = page.locator(canvas);
  await expect.poll(async () => c.evaluate((el) => el.dataset.ready === 'true'), {
    timeout: 30_000,
  }).toBe(true);
  await expect.poll(async () => c.evaluate((el) => {
    const save = JSON.parse(el.dataset.session || '{}');
    return Boolean(save.version === 13 && save.solitaire && save.freecell && save.spider);
  })).toBe(true);
  await clickLobbyAction(page, family.toLowerCase());
  await expect.poll(async () => interaction(page).then((i) => i.screen)).toBe(family);
  const variant = family === 'Solitaire' ? 'Klondike1' : family === 'Spider' ? 'Spider1' : 'FreeCell';
  await expect.poll(async () => view(page).then((v) => v.variant)).toBe(variant);
}

async function clickLobbyAction(page, key) {
  const activate = (bounds) => clickBounds(page, bounds);
  await activate(await revealLobbyGame(page, key, activate));
}



// Install after the old document's pagehide save, once only, so subsequent
// reloads exercise persistence rather than restoring the fixture again.
async function seedNextLoad(page, storageKey, session, marker) {
  await page.addInitScript(({ storageKey, session, marker }) => {
    if (!sessionStorage.getItem(marker)) {
      localStorage.setItem(storageKey, JSON.stringify(session));
      sessionStorage.setItem(marker, 'seeded');
    }
  }, { storageKey, session, marker });
}

async function interaction(page) {
  return page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.interaction || '{}'));
}

async function save(page) {
  return page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.session || '{}'));
}

async function view(page) {
  return page.locator(canvas).evaluate((el) => JSON.parse(el.dataset.patienceView || '{}'));
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

async function clickBounds(page, bounds) {
  expect(bounds).toHaveLength(4);
  const point = center(bounds);
  await clickSurface(page, point.x, point.y);
}

async function clickAction(page, key) {
  await expect.poll(async () => {
    const bounds = (await actions(page))[key];
    return Array.isArray(bounds) && bounds.length === 4;
  }).toBe(true);
  await clickBounds(page, (await actions(page))[key]);
}

function gameplay(game) {
  const { elapsed_ms, ...state } = game;
  return state;
}

async function expectBoundsInsideViewport(page, keys) {
  const { viewport, bounds } = await page.locator(canvas).evaluate((el, keys) => ({
    viewport: JSON.parse(el.dataset.interaction).viewport,
    bounds: JSON.parse(el.dataset.actionBounds || '{}'),
  }), keys);
  expect(viewport).toEqual([390, 740]);
  for (const key of keys) {
    const rect = bounds[key];
    expect(rect, `${key} is published`).toHaveLength(4);
    expect(rect[0]).toBeGreaterThanOrEqual(0);
    expect(rect[1]).toBeGreaterThanOrEqual(0);
    expect(rect[0] + rect[2]).toBeLessThanOrEqual(viewport[0] + 0.5);
    expect(rect[1] + rect[3]).toBeLessThanOrEqual(viewport[1] + 0.5);
  }
}

test('Klondike draw, undo, and reload preserve the exact saved gameplay', async ({ page }) => {
  await openPatience(page);
  const before = (await save(page)).solitaire;
  expect(before.variant).toBe('Klondike1');
  await clickAction(page, 'deal');
  await expect.poll(async () => (await save(page)).solitaire.moves).toBe(before.moves + 1);
  const dealt = (await save(page)).solitaire;
  expect(gameplay(dealt)).not.toEqual(gameplay(before));

  await clickAction(page, 'undo');
  await expect.poll(async () => gameplay((await save(page)).solitaire)).toEqual(gameplay(before));
  await clickAction(page, 'deal');
  await expect.poll(async () => (await save(page)).solitaire.moves).toBe(before.moves + 1);
  const persisted = (await save(page)).solitaire;
  await page.reload();
  await expect.poll(async () => page.locator(canvas).evaluate((el) => el.dataset.ready === 'true'), {
    timeout: 30_000,
  }).toBe(true);
  await expect.poll(async () => {
    const current = (await save(page)).solitaire;
    return current ? gameplay(current) : null;
  }).toEqual(gameplay(persisted));
  await expect.poll(async () => interaction(page).then((i) => i.screen)).toBe('Lobby');
});

for (const delayedFrames of [false, true]) {
  test(`FreeCell hint and coordinate selection execute a move through hit bounds${delayedFrames ? ' with delayed frames' : ''}`, async ({ page }) => {
    await openPatience(page, 'FreeCell');
    const initial = (await save(page)).freecell;
    expect(initial.variant).toBe('FreeCell');
    if (delayedFrames) {
      // Delay application frames without delaying pointer input or DOM reads.
      // The hint's variant is already FreeCell before its selection is published.
      await page.evaluate(() => {
        const nextFrame = window.requestAnimationFrame.bind(window);
        window.requestAnimationFrame = (callback) => nextFrame((time) => setTimeout(() => callback(time), 300));
      });
    }
    await clickAction(page, 'hint');
    await expect.poll(async () => (await view(page)).selected).not.toBeNull();
    await expect.poll(async () => (await save(page)).freecell.assisted).toBe(true);
    const hintView = await view(page);
    expect(hintView.selected).not.toBeNull();

    // Hit keys encode the selected source and target piles. Click both through
    // their published card bounds so this exercises the same pointer path.
    const [pile, index] = hintView.selected;
    expect(pile).toMatch(/^(Tableau|Cell|Waste|Foundation)/);
    const sourceKey = `pick${pile}_${index}`;
    await expect.poll(async () => (await actions(page))[sourceKey]).toHaveLength(4);
    const published = await actions(page);
    const candidates = Object.keys(published).filter((key) => key.startsWith('target')
      && Array.isArray(published[key]) && published[key].length === 4);
    expect(candidates.length).toBeGreaterThan(0);
    for (const target of candidates) {
      await clickAction(page, target);
      if ((await save(page)).freecell.moves > initial.moves) break;
    }
    await expect.poll(async () => (await save(page)).freecell.moves).toBeGreaterThan(initial.moves);
  });
}

for (const family of ['FreeCell', 'Spider']) {
  test(`lobby opens ${family} and preserves the Solitaire cabinet`, async ({ page }) => {
    await openPatience(page, 'Solitaire');
    await clickAction(page, 'deal');
    await expect.poll(async () => (await save(page)).solitaire.moves).toBe(1);
    const solitaire = gameplay((await save(page)).solitaire);
    await clickAction(page, 'lobby');
    await expect.poll(async () => interaction(page).then((i) => i.screen)).toBe('Lobby');
    await clickLobbyAction(page, family.toLowerCase());
    await expect.poll(async () => interaction(page).then((i) => i.screen)).toBe(family);
    if (family === 'Spider') {
      await clickAction(page, 'deal');
      await expect.poll(async () => (await save(page)).spider.moves).toBe(1);
    }
    expect(gameplay((await save(page)).solitaire)).toEqual(solitaire);
    await clickAction(page, 'lobby');
    await expect.poll(async () => interaction(page).then((i) => i.screen)).toBe('Lobby');
    await clickLobbyAction(page, 'solitaire');
    await expect.poll(async () => interaction(page).then((i) => i.screen)).toBe('Solitaire');
    await expect.poll(async () => (await save(page)).solitaire.moves).toBe(1);
  });
}

test('canceling and confirming a new Solitaire mode protects other cabinets', async ({ page }) => {
  await openPatience(page, 'Solitaire');
  await clickAction(page, 'deal');
  await expect.poll(async () => (await save(page)).solitaire.moves).toBe(1);
  const before = await save(page);
  await clickAction(page, 'modeKlondike3');
  await expect.poll(async () => interaction(page).then((i) => i.modal)).toBe('patience-replace');
  await clickAction(page, 'cancel');
  expect(gameplay((await save(page)).solitaire)).toEqual(gameplay(before.solitaire));
  expect((await save(page)).freecell).toEqual(before.freecell);
  expect((await save(page)).spider).toEqual(before.spider);
  await clickAction(page, 'modeKlondike3');
  await expect.poll(async () => interaction(page).then((i) => i.modal)).toBe('patience-replace');
  await clickAction(page, 'confirm');
  await expect.poll(async () => (await save(page)).solitaire.variant).toBe('Klondike3');
  expect((await save(page)).freecell).toEqual(before.freecell);
  expect((await save(page)).spider).toEqual(before.spider);
});

test('FreeCell Relaxed mode protects progress and persists as a separate variant', async ({ page }) => {
  await openPatience(page, 'FreeCell');
  expect((await view(page)).modeLabel).toBe('Standard');
  expect((await view(page)).canFinish).toBe(false);
  expect((await actions(page)).finish).toBeUndefined();
  const initial = await save(page);
  const game = initial.freecell;
  const index = game.tableau[0].length - 1;
  await clickAction(page, `pickTableau(0)_${index}`);
  await expect.poll(async () => (await view(page)).selected).not.toBeNull();
  await clickAction(page, 'targetCell(1)');
  await expect.poll(async () => (await save(page)).freecell.moves).toBe(game.moves + 1);
  const progressed = await save(page);

  await clickAction(page, 'modeFreeCellRelaxed');
  await expect.poll(async () => interaction(page).then((i) => i.modal)).toBe('patience-replace');
  await clickAction(page, 'cancel');
  expect(gameplay((await save(page)).freecell)).toEqual(gameplay(progressed.freecell));
  await clickAction(page, 'modeFreeCellRelaxed');
  await expect.poll(async () => interaction(page).then((i) => i.modal)).toBe('patience-replace');
  await clickAction(page, 'confirm');
  await expect.poll(async () => view(page).then((v) => v.variant)).toBe('FreeCellRelaxed');
  expect((await view(page)).modeLabel).toBe('Relaxed');
  const relaxed = await save(page);
  expect(relaxed.solitaire).toEqual(progressed.solitaire);
  expect(relaxed.spider).toEqual(progressed.spider);
  expect(relaxed.patience_records).toEqual(progressed.patience_records);
  await page.reload();
  await expect.poll(async () => page.locator(canvas).evaluate((el) => el.dataset.ready === 'true'), { timeout: 30_000 }).toBe(true);
  await expect.poll(async () => (await save(page)).freecell.variant).toBe('FreeCellRelaxed');
  await clickLobbyAction(page, 'freecell');
  await expect.poll(async () => view(page).then((v) => v.modeLabel)).toBe('Relaxed');
});

test('FreeCell finish completes a provable two card foundation and undo treats it as one move', async ({ page }) => {
  await page.goto('/?qa=fresh&diagnostics');
  await expect.poll(async () => page.locator(canvas).evaluate((el) => el.dataset.ready === 'true'), { timeout: 30_000 }).toBe(true);
  const base = await save(page);
  const key = 'gamevex.rust-demo.qa.v1';
  const nearWin = structuredClone(base);
  nearWin.version = 12;
  nearWin.freecell = {
    ...nearWin.freecell,
    variant: 'FreeCell',
    foundations: [
      [14, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13],
      [14, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13],
      [14, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13],
      [14, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
    ].map((ranks, i) => ranks.map((rank) => ({ rank, suit: ['Spades', 'Hearts', 'Clubs', 'Diamonds'][i], face_up: true }))),
    tableau: [[13, 12].map((rank) => ({ rank, suit: 'Diamonds', face_up: true })), [], [], [], [], [], [], []],
    cells: [null, null, null, null],
    stock: [],
    waste: [],
    history: [],
    moves: 1,
  };
  await seedNextLoad(page, key, nearWin, 'near-win');
  await page.reload();
  await expect.poll(async () => page.locator(canvas).evaluate((el) => el.dataset.ready === 'true'), { timeout: 30_000 }).toBe(true);
  expect((await save(page)).solitaire).toEqual(base.solitaire);
  expect((await save(page)).spider).toEqual(base.spider);
  expect((await save(page)).patience_records).toEqual(base.patience_records);
  await clickLobbyAction(page, 'freecell');
  await expect.poll(async () => view(page).then((v) => v.variant)).toBe('FreeCell');
  await expect.poll(async () => (await view(page)).canFinish).toBe(true);
  await expect.poll(async () => (await actions(page)).finish).toHaveLength(4);
  await clickAction(page, 'finish');
  await expect.poll(async () => (await view(page)).won).toBe(true);
  expect((await view(page)).canFinish).toBe(false);
  const won = gameplay((await save(page)).freecell);
  await clickAction(page, 'undo');
  await expect.poll(async () => gameplay((await save(page)).freecell)).toEqual(gameplay(nearWin.freecell));
  expect((await view(page)).won).toBe(false);
  await clickAction(page, 'finish');
  await expect.poll(async () => (await view(page)).won).toBe(true);
  expect(gameplay((await save(page)).freecell)).toEqual(won);
  await page.reload();
  await expect.poll(async () => page.locator(canvas).evaluate((el) => el.dataset.ready === 'true'), { timeout: 30_000 }).toBe(true);
  await clickLobbyAction(page, 'freecell');
  await expect.poll(async () => (await view(page)).won).toBe(true);
  expect((await view(page)).animating).toBe(false);
});

test('patience reports delayed move and deal animations and honors reduced motion', async ({ page }) => {
  await openPatience(page, 'Solitaire');
  await expect.poll(async () => typeof (await view(page)).animating).toBe('boolean');
  const oldMoves = (await save(page)).solitaire.moves;
  await clickAction(page, 'deal');
  await expect.poll(async () => (await save(page)).solitaire.moves).toBe(oldMoves + 1);
  expect(typeof (await view(page)).animating).toBe('boolean');
  await expect.poll(async () => (await view(page)).animating, { timeout: 3_000 }).toBe(false);

  await clickAction(page, 'lobby');
  await clickLobbyAction(page, 'freecell');
  const normalGame = (await save(page)).freecell;
  const normalIndex = normalGame.tableau[0].length - 1;
  await clickAction(page, `pickTableau(0)_${normalIndex}`);
  await clickAction(page, 'targetCell(0)');
  await expect.poll(async () => (await save(page)).freecell.moves).toBe(normalGame.moves + 1);
  expect(typeof (await view(page)).animating).toBe('boolean');
  await expect.poll(async () => (await view(page)).animating, { timeout: 3_000 }).toBe(false);

  const reduced = await save(page);
  reduced.settings.reduced_motion = true;
  await seedNextLoad(page, 'gamevex.rust-demo.qa.v1', reduced, 'reduced-motion');
  await page.goto('/?qa=fresh&diagnostics');
  await expect.poll(async () => page.locator(canvas).evaluate((el) => el.dataset.ready === 'true'), { timeout: 30_000 }).toBe(true);
  await clickLobbyAction(page, 'freecell');
  const game = (await save(page)).freecell;
  const index = game.tableau[0].length - 1;
  await clickAction(page, `pickTableau(0)_${index}`);
  await expect.poll(async () => (await view(page)).selected).not.toBeNull();
  await clickAction(page, 'targetCell(1)');
  await expect.poll(async () => (await save(page)).freecell.moves).toBe(game.moves + 1);
  expect((await view(page)).animating).toBe(false);
});

test('patience controls fit on a phone and Spider supports horizontal swipe pan', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 740 });
  await openPatience(page);
  for (const family of ['Solitaire', 'FreeCell', 'Spider']) {
    if (family !== 'Solitaire') {
      await clickAction(page, 'lobby');
      await clickLobbyAction(page, family.toLowerCase());
    }
    const modeKeys = family === 'Solitaire' ? ['modeKlondike3']
      : family === 'FreeCell' ? [] : ['modeSpider2', 'modeSpider4'];
    await expect.poll(async () => interaction(page).then((i) => i.screen)).toBe(family);
    const variant = family === 'Solitaire' ? 'Klondike1' : family === 'Spider' ? 'Spider1' : 'FreeCell';
    await expect.poll(async () => view(page).then((v) => v.variant)).toBe(variant);
    const current = await actions(page);
    const controls = [...modeKeys, 'hint', 'new', 'restart'];
    if (family !== 'FreeCell') controls.push('deal');
    if (Array.isArray(current.undo)) controls.push('undo');
    await expectBoundsInsideViewport(page, controls);
  }
  const modeKeys = ['modeSpider1', 'modeSpider2', 'modeSpider4'];
  await expectBoundsInsideViewport(page, [...modeKeys, 'deal', 'hint', 'new', 'restart']);
  await clickAction(page, 'modeSpider4');
  await expect.poll(async () => (await save(page)).spider.variant).toBe('Spider4');

  const before = (await view(page)).pan;
  const currentView = await view(page);
  const clip = currentView.clip;
  await dragTouch(page,
    { x: clip[0] + clip[2] * 0.8, y: clip[1] + clip[3] * 0.7 },
    { x: clip[0] + clip[2] * 0.2, y: clip[1] + clip[3] * 0.7 });
  await expect.poll(async () => view(page).then((v) => v.pan)).not.toEqual(before);
});

test('FreeCell mouse press stays pending, then dragging moves an exposed card',async({page})=>{
 await openPatience(page,'FreeCell');const g=(await save(page)).freecell;const i=g.tableau[0].length-1;
 const b=await actions(page);const a=center(b[`pickTableau(0)_${i}`]),z=center(b['targetCell(0)']);
 const from=await screenPoint(page,a.x,a.y),to=await screenPoint(page,z.x,z.y);
 await page.mouse.move(from.x,from.y);await page.mouse.down();
 await expect.poll(async()=> (await view(page)).dragSource).toEqual(['Tableau(0)',i]);
 await expect.poll(async()=> (await view(page)).selected).toBeNull();
 expect((await view(page)).dragActive).toBe(false);
 await page.mouse.up();
 expect(gameplay((await save(page)).freecell)).toEqual(gameplay(g));
 await page.mouse.move(from.x,from.y);await page.mouse.down();
 await page.mouse.move(from.x+12,from.y,{steps:2});
 await expect.poll(async()=> (await view(page)).dragActive).toBe(true);
 await expect.poll(async()=> (await view(page)).dragSource).toEqual(['Tableau(0)',i]);
 await page.mouse.move(to.x,to.y,{steps:8});await page.mouse.up();
 await expect.poll(async()=> (await save(page)).freecell.moves).toBe(1);
 expect((await save(page)).freecell.cells[0]).toEqual(g.tableau[0][i]);
});

test('FreeCell touch drag moves a card',async({page})=>{
 await openPatience(page,'FreeCell');const g=(await save(page)).freecell;const i=g.tableau[0].length-1;
 const b=await actions(page);const a=center(b[`pickTableau(0)_${i}`]),z=center(b['targetCell(0)']);
 await dragTouch(page,a,z);
 await expect.poll(async()=> (await save(page)).freecell.moves).toBe(1);
 expect((await save(page)).freecell.cells[0]).toEqual(g.tableau[0][i]);
});

test('FreeCell restart confirms before restoring the same saved deal', async ({ page }) => {
  await openPatience(page, 'FreeCell');
  const initial = (await save(page)).freecell;
  const index = initial.tableau[0].length - 1;
  await clickAction(page, `pickTableau(0)_${index}`);
  await expect.poll(async () => (await view(page)).selected).toEqual(['Tableau(0)', index]);
  await clickAction(page, 'targetCell(0)');
  await expect.poll(async () => (await save(page)).freecell.moves).toBe(1);
  await clickAction(page, 'restart');
  await expect.poll(async () => (await interaction(page)).modal).toBe('patience-replace');
  expect((await save(page)).freecell.moves).toBe(1);
  await clickAction(page, 'confirm');
  await expect.poll(async () => gameplay((await save(page)).freecell)).toEqual(gameplay(initial));
});

test('an invalid FreeCell touch drop leaves the save unchanged',async({page})=>{
 await openPatience(page,'FreeCell');
 const before=(await save(page)).freecell;
 const index=before.tableau[0].length-1;
 const bounds=await actions(page);
 const source=center(bounds[`pickTableau(0)_${index}`]);
 await dragTouch(page,source,{x:12,y:12});
 await expect.poll(async()=> (await view(page)).dragActive).toBe(false);
 expect(gameplay((await save(page)).freecell)).toEqual(gameplay(before));
});

test('a canceled FreeCell touch drag leaves the save unchanged',async({page})=>{
 await openPatience(page,'FreeCell');
 const before=(await save(page)).freecell;
 const index=before.tableau[0].length-1;
 const bounds=await actions(page);
 const source=center(bounds[`pickTableau(0)_${index}`]);
 await cancelActiveTouchDrag(page,source);
 expect(gameplay((await save(page)).freecell)).toEqual(gameplay(before));
});

test('FreeCell move and undo feedback plays once and does not replay after reload',async({page})=>{
 await openPatience(page,'FreeCell');
 const initialAudio=(await audioStatus(page)).played;
 const game=(await save(page)).freecell;
 const index=game.tableau[0].length-1;
 const bounds=await actions(page);
 const from=center(bounds[`pickTableau(0)_${index}`]);
 const to=center(bounds['targetCell(0)']);
 const start=await screenPoint(page,from.x,from.y),end=await screenPoint(page,to.x,to.y);
 await page.mouse.move(start.x,start.y);await page.mouse.down();
 await page.mouse.move(end.x,end.y,{steps:8});await page.mouse.up();
 await expect.poll(async()=> (await save(page)).freecell.moves).toBe(1);
 await expect.poll(async()=> (await tableFeedback(page)).events.some(e=>e.kind==='Move')).toBe(true);
 await expect.poll(async()=> (await audioStatus(page)).played).toBeGreaterThan(initialAudio);
 await expect.poll(async()=> (await audioStatus(page)).last).toBe('TableMove');
 await clickAction(page,'undo');
 await expect.poll(async()=> (await save(page)).freecell.moves).toBe(0);
 await expect.poll(async()=> (await tableFeedback(page)).events.some(e=>e.kind==='Undo')).toBe(true);
 await expect.poll(async()=> (await audioStatus(page)).played).toBeGreaterThan(initialAudio+1);
 await page.reload();
 await expect.poll(async()=>interaction(page).then(i=>i.screen)).toBe('Lobby');
 const afterReload=(await audioStatus(page)).played;
 await clickLobbyAction(page,'freecell');
 await expect.poll(async()=>interaction(page).then(i=>i.screen)).toBe('FreeCell');
 await expect.poll(async()=> (await audioStatus(page)).played).toBe(afterReload+2); // Category and game navigation clicks only.
 expect((await tableFeedback(page)).events).toHaveLength(0);
 await page.waitForTimeout(500);
 expect((await audioStatus(page)).played).toBe(afterReload+2);
});

test('FreeCell Undo feedback expires across slow application frames',async({page})=>{
 await openPatience(page,'FreeCell');
 const initialAudio=(await audioStatus(page)).played;
 const game=(await save(page)).freecell;
 const index=game.tableau[0].length-1;
 const bounds=await actions(page);
 const from=center(bounds[`pickTableau(0)_${index}`]);
 const to=center(bounds['targetCell(0)']);
 const start=await screenPoint(page,from.x,from.y),end=await screenPoint(page,to.x,to.y);
 await page.mouse.move(start.x,start.y);await page.mouse.down();
 await page.mouse.move(end.x,end.y,{steps:8});await page.mouse.up();
 await expect.poll(async()=> (await save(page)).freecell.moves).toBe(1);
 await expect.poll(async()=> (await tableFeedback(page)).events.some(e=>e.kind==='Move')).toBe(true);
 await page.evaluate(() => {
  const nextFrame=window.requestAnimationFrame.bind(window);
  window.requestAnimationFrame=(callback)=>nextFrame((time)=>setTimeout(()=>callback(time),1_200));
 });
 await clickAction(page,'undo');
 await expect.poll(async()=> (await save(page)).freecell.moves).toBe(0);
 await expect.poll(async()=> (await audioStatus(page)).last).toBe('TableUndo');
 await expect.poll(async()=> (await audioStatus(page)).played).toBeGreaterThan(initialAudio+1);
 await expect.poll(async()=> (await tableFeedback(page)).events,{timeout:5_000}).toHaveLength(0);
 await expect.poll(async()=> (await audioStatus(page)).voices).toBe(0);
});

test.describe('Phone card selection',()=>{
 test.use({viewport:{width:390,height:740},hasTouch:true});
 test('FreeCell taps select and move a card',async({page})=>{
  await openPatience(page,'FreeCell');const g=(await save(page)).freecell;const i=g.tableau[0].length-1;
  const b=await actions(page);const a=center(b[`pickTableau(0)_${i}`]);await touchSurface(page,a.x,a.y);
  await expect.poll(async()=> (await view(page)).selected).toEqual(['Tableau(0)',i]);
  const z=center((await actions(page))['targetCell(0)']);await touchSurface(page,z.x,z.y);
  await expect.poll(async()=> (await save(page)).freecell.moves).toBe(1);
 });
 test('a phone touch drag moves a FreeCell card',async({page})=>{
  await openPatience(page,'FreeCell');const g=(await save(page)).freecell;const i=g.tableau[0].length-1;
  const b=await actions(page);const source=center(b[`pickTableau(0)_${i}`]),target=center(b['targetCell(0)']);
  await dragTouch(page,source,target);
  await expect.poll(async()=> (await save(page)).freecell.moves).toBe(1);
 });
 test('canceling an active touch drag clears its source without changing the save',async({page})=>{
  await openPatience(page,'FreeCell');
  const before=(await save(page)).freecell;
  const last=before.tableau[0].length-1;
  const source=center((await actions(page))[`pickTableau(0)_${last}`]);
  await cancelActiveTouchDrag(page,source);
  await expect.poll(async()=> (await view(page)).dragActive).toBe(false);
  expect(gameplay((await save(page)).freecell)).toEqual(gameplay(before));
 });
 test('a canceled category tap stays on the grid and a swipe reveals Roulette',async({page})=>{
  await page.setViewportSize({ width: 360, height: 560 });
  await page.goto('/?qa=fresh&diagnostics');
  await expect.poll(async()=>page.locator(canvas).evaluate(el=>el.dataset.ready==='true'),{timeout:30_000}).toBe(true);
  const category=center((await actions(page)).categories.highLimit);
  await cancelTouch(page,category);
  await expect.poll(async()=>interaction(page).then(i=>i.screen)).toBe('Lobby');
  expect((await actions(page)).lobbyCategory).toBeNull();
  await touchSurface(page, category.x, category.y);
  await expect.poll(async () => (await actions(page)).lobbyCategory).toBe('highLimit');
  const before=(await actions(page)).lobbyScroll;
  const clip=(await actions(page)).lobbyClip;
  await dragTouch(page,{x:clip[0]+clip[2]/2,y:clip[1]+clip[3]*0.85},{x:clip[0]+clip[2]/2,y:clip[1]+clip[3]*0.2});
  await expect.poll(async()=>(await actions(page)).lobbyScroll).toBeGreaterThan(before);
  const roulette = center((await actions(page)).roulette);
  await touchSurface(page, roulette.x, roulette.y);
  await expect.poll(async()=>interaction(page).then(i=>i.screen)).toBe('Roulette');
 });
});
