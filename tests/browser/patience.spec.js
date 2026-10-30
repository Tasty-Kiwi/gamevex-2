import { expect, test } from '@playwright/test';

const canvas = '#gamevex';

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

async function openPatience(page, family = 'Solitaire') {
  await page.goto('/?qa=fresh&diagnostics');
  const c = page.locator(canvas);
  await expect.poll(async () => c.evaluate((el) => el.dataset.ready === 'true'), {
    timeout: 30_000,
  }).toBe(true);
  await expect.poll(async () => c.evaluate((el) => {
    const save = JSON.parse(el.dataset.session || '{}');
    return Boolean(save.version === 6 && save.solitaire && save.freecell && save.spider);
  })).toBe(true);
  const lobby = await c.evaluate((el) => JSON.parse(el.dataset.actionBounds || '{}'));
  await clickBounds(page, lobby.solitaire);
  await expect.poll(async () => interaction(page).then((i) => i.screen)).toBe('Solitaire');
  if (family !== 'Solitaire') await clickAction(page, `family${family}`);
  await expect.poll(async () => interaction(page).then((i) => i.screen)).toBe(family);
  const variant = family === 'Solitaire' ? 'Klondike1' : family === 'Spider' ? 'Spider1' : 'FreeCell';
  await expect.poll(async () => view(page).then((v) => v.variant)).toBe(variant);
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

test('FreeCell hint and coordinate selection execute a move through hit bounds', async ({ page }) => {
  await openPatience(page, 'FreeCell');
  const initial = (await save(page)).freecell;
  expect(initial.variant).toBe('FreeCell');
  await clickAction(page, 'hint');
  await expect.poll(async () => view(page)).toMatchObject({ variant: 'FreeCell' });
  const hintView = await view(page);
  expect(hintView.selected).not.toBeNull();

  // Hit keys encode the selected source and target piles. Click both through
  // their published card bounds so this exercises the same pointer path.
  const [pile, index] = (await view(page)).selected;
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

test('switching patience families keeps each game and canceling a new mode protects all cabinets', async ({ page }) => {
  await openPatience(page, 'Solitaire');
  await clickAction(page, 'deal');
  await expect.poll(async () => (await save(page)).solitaire.moves).toBe(1);
  const afterDeal = await save(page);
  await clickAction(page, 'familyFreeCell');
  await expect.poll(async () => interaction(page).then((i) => i.screen)).toBe('FreeCell');
  const freecell = (await save(page)).freecell;
  await clickAction(page, 'familySpider');
  await expect.poll(async () => interaction(page).then((i) => i.screen)).toBe('Spider');
  await clickAction(page, 'deal');
  await expect.poll(async () => (await save(page)).spider.moves).toBe(1);
  const spider = (await save(page)).spider;
  await clickAction(page, 'familySolitaire');
  await expect.poll(async () => interaction(page).then((i) => i.screen)).toBe('Solitaire');
  await expect.poll(async () => (await save(page)).solitaire.moves).toBe(afterDeal.solitaire.moves);
  expect(gameplay((await save(page)).solitaire)).toEqual(gameplay(afterDeal.solitaire));

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
  expect(spider.variant).toBe('Spider1');
  expect(freecell.variant).toBe('FreeCell');
});

test('all family controls fit on a phone and Spider supports horizontal swipe pan', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 740 });
  await openPatience(page);
  for (const family of ['Solitaire', 'FreeCell', 'Spider']) {
    const familyKey = `family${family}`;
    const modeKeys = family === 'Solitaire' ? ['modeKlondike3']
      : family === 'FreeCell' ? [] : ['modeSpider2', 'modeSpider4'];
    await clickAction(page, familyKey);
    await expect.poll(async () => interaction(page).then((i) => i.screen)).toBe(family);
    const variant = family === 'Solitaire' ? 'Klondike1' : family === 'Spider' ? 'Spider1' : 'FreeCell';
    await expect.poll(async () => view(page).then((v) => v.variant)).toBe(variant);
    const current = await actions(page);
    const controls = [familyKey, ...modeKeys, 'hint', 'new', 'restart'];
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

test('FreeCell mouse drag moves an exposed card into a free cell',async({page})=>{
 await openPatience(page,'FreeCell');const g=(await save(page)).freecell;const i=g.tableau[0].length-1;
 const b=await actions(page);const a=center(b[`pickTableau(0)_${i}`]),z=center(b['targetCell(0)']);
 const from=await screenPoint(page,a.x,a.y),to=await screenPoint(page,z.x,z.y);
 await page.mouse.move(from.x,from.y);await page.mouse.down();
 await expect.poll(async()=> (await view(page)).selected).toEqual(['Tableau(0)',i]);
 await page.mouse.move(to.x,to.y,{steps:8});await page.mouse.up();
 await expect.poll(async()=> (await save(page)).freecell.moves).toBe(1);
 expect((await save(page)).freecell.cells[0]).toEqual(g.tableau[0][i]);
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
});
