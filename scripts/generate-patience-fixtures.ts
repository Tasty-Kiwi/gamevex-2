import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const oracleRoot = resolve(process.argv[2] ?? "../gamevex");
const output = resolve(process.argv[3] ?? resolve(scriptDir, "../tests/fixtures/patience-oracle.json"));
const sources = [
  "apps/local/src/utils/solitaire.ts",
  "apps/local/src/utils/freecell.ts",
  "apps/local/src/utils/spider.ts",
];
const hashes = await Promise.all(sources.map(async (path) => ({ path, sha256: createHash("sha256").update(await readFile(resolve(oracleRoot, path))).digest("hex") })));
let oracleGitRevision: string | null = null;
try { oracleGitRevision = execFileSync("git", ["-C", oracleRoot, "rev-parse", "HEAD"], { encoding: "utf8" }).trim(); } catch { /* hashes still identify the exact source files */ }
const [solitaire, freecell, spider] = await Promise.all(sources.map((path) => import(pathToFileURL(resolve(oracleRoot, path)).href)));
const card = (rank: string, suit: string, id: string, faceUp = true) => ({ rank, suit, id, faceUp });
const emptySolitaire = () => ({ stock: [], waste: [], foundations: [[], [], [], []], tableau: [[], [], [], [], [], [], []] });
const emptyFreeCell = () => ({ freeCells: [null, null, null, null], foundations: [[], [], [], []], cascades: [[], [], [], [], [], [], [], []] });
const emptySpider = () => ({ stock: [], completedRuns: [], tableau: [[], [], [], [], [], [], [], [], [], []] });
const states: Record<string, unknown> = {};
const expected: Record<string, unknown> = {};

// Klondike draw-three: stock top is the last element; the third card becomes waste top.
const draw3 = emptySolitaire();
draw3.stock = [card("5", "clubs", "draw-1", false), card("A", "spades", "draw-2", false), card("7", "hearts", "draw-3", false)];
const drawn = solitaire.drawFromStock(draw3, 3);
states.klondike_draw3 = draw3;
expected.klondike_draw3 = { stock: drawn.stock.map((c: any) => c.id), waste: drawn.waste.map((c: any) => c.id), waste_top: drawn.waste.at(-1)?.id };
const recycle = { ...emptySolitaire(), waste: [card("2", "clubs", "w1"), card("3", "hearts", "w2"), card("4", "spades", "w3")] };
const recycled = solitaire.recycleWaste(recycle);
states.klondike_recycle = recycle;
expected.klondike_recycle = { stock: recycled.stock.map((c: any) => c.id), next_draw: recycled.stock.at(-1)?.id, waste: recycled.waste.length };
const klondikeStack = emptySolitaire();
klondikeStack.tableau[0] = [card("9", "spades", "k9"), card("8", "hearts", "k8")];
klondikeStack.tableau[1] = [card("9", "clubs", "k9-target")];
klondikeStack.tableau[2] = [card("10", "hearts", "k10-red")];
states.klondike_stack = klondikeStack;
expected.klondike_stack = {
  valid_substack: solitaire.moveCards(klondikeStack, { zone: "tableau", columnIndex: 0, cardIndex: 1 }, "tableau", 1) !== null,
  valid_whole_stack: solitaire.moveCards(klondikeStack, { zone: "tableau", columnIndex: 0, cardIndex: 0 }, "tableau", 2) !== null,
  invalid_same_color: solitaire.moveCards(klondikeStack, { zone: "tableau", columnIndex: 0, cardIndex: 1 }, "tableau", 2) !== null,
};
const klondikeFoundation = emptySolitaire();
klondikeFoundation.foundations[0] = [card("A", "spades", "fa"), card("2", "spades", "f2")];
klondikeFoundation.tableau[0] = [card("3", "hearts", "t3")];
states.klondike_foundation_back = klondikeFoundation;
expected.klondike_foundation_back = solitaire.moveCards(klondikeFoundation, { zone: "foundation", columnIndex: 0 }, "tableau", 0) !== null;

// FreeCell strict supermove capacity excludes the empty destination column.
const freecellValid = emptyFreeCell();
freecellValid.cascades[0] = [card("9", "spades", "f9"), card("8", "hearts", "f8"), card("7", "clubs", "f7")];
freecellValid.cascades[1] = [card("10", "diamonds", "f10")];
states.freecell_valid_stack = freecellValid;
expected.freecell_valid_stack = freecell.moveCards(freecellValid, { zone: "cascade", columnIndex: 0, cardIndex: 0 }, { zone: "cascade", columnIndex: 1 }, "strict") !== null;
const freecellCapacity = emptyFreeCell();
freecellCapacity.freeCells = [card("2", "spades", "cell1"), card("3", "spades", "cell2"), card("4", "spades", "cell3"), card("5", "spades", "cell4")];
freecellCapacity.cascades[0] = [card("9", "spades", "c9"), card("8", "hearts", "c8"), card("7", "clubs", "c7")];
for (let i = 3; i < 8; i++) freecellCapacity.cascades[i] = [card("2", "clubs", `occupied-${i}`)];
states.freecell_empty_target_capacity = freecellCapacity;
expected.freecell_empty_target_capacity = {
  strict_accepts: freecell.moveCards(freecellCapacity, { zone: "cascade", columnIndex: 0, cardIndex: 0 }, { zone: "cascade", columnIndex: 1 }, "strict") !== null,
  relaxed_accepts: freecell.moveCards(freecellCapacity, { zone: "cascade", columnIndex: 0, cardIndex: 0 }, { zone: "cascade", columnIndex: 1 }, "relaxed") !== null,
  strict_capacity: freecell.getMoveCapacity(freecellCapacity, 0, 1, "strict"),
};
const freecellFoundation = emptyFreeCell();
freecellFoundation.foundations[0] = [card("A", "spades", "ffa"), card("2", "spades", "ff2")];
freecellFoundation.cascades[0] = [card("3", "hearts", "ft3")];
states.freecell_foundation_back = freecellFoundation;
expected.freecell_foundation_back = freecell.moveCards(freecellFoundation, { zone: "foundation", columnIndex: 0 }, { zone: "cascade", columnIndex: 0 }, "strict") !== null;

// Spider permits mixed-suit rank placement, while a movable stack must be same-suit.
const spiderMixed = emptySpider();
spiderMixed.tableau[0] = [card("9", "spades", "s9"), card("8", "hearts", "s8")];
spiderMixed.tableau[1] = [card("10", "clubs", "s10")];
states.spider_mixed_stack = spiderMixed;
expected.spider_mixed_stack = {
  mixed_suit_run_movable: spider.getMovableCards(spiderMixed, { zone: "tableau", columnIndex: 0, cardIndex: 0 }) !== null,
  mixed_suit_rank_placement: spider.canPlaceOnTableau(spiderMixed.tableau[0][0], spiderMixed.tableau[1]),
};
const spiderSingle = emptySpider();
spiderSingle.tableau[0] = [card("9", "spades", "single-9")];
spiderSingle.tableau[1] = [card("10", "clubs", "single-10")];
states.spider_mixed_suit_placement = spiderSingle;
expected.spider_mixed_suit_placement = spider.moveCards(spiderSingle, { zone: "tableau", columnIndex: 0, cardIndex: 0 }, { zone: "tableau", columnIndex: 1 }) !== null;
const spiderDeal = emptySpider();
spiderDeal.stock = Array.from({ length: 10 }, (_, i) => card("2", "spades", `stock-${i}`, false));
spiderDeal.tableau[4] = [card("K", "clubs", "only-empty-blocker")];
states.spider_stock_empty_column = spiderDeal;
expected.spider_stock_empty_column = { can_deal: spider.canDealFromStock(spiderDeal), deal_succeeds: spider.dealFromStock(spiderDeal) !== null };
const spiderAutoDeal = emptySpider();
spiderAutoDeal.stock = [card("A", "spades", "auto-ace", false), ...Array.from({ length: 9 }, (_, i) => card("2", "hearts", `auto-other-${i}`, false))];
spiderAutoDeal.tableau[0] = [
  card("K", "spades", "run-k"), card("Q", "spades", "run-q"), card("J", "spades", "run-j"), card("10", "spades", "run-10"), card("9", "spades", "run-9"), card("8", "spades", "run-8"), card("7", "spades", "run-7"), card("6", "spades", "run-6"), card("5", "spades", "run-5"), card("4", "spades", "run-4"), card("3", "spades", "run-3"), card("2", "spades", "run-2"),
];
for (let i = 1; i < 10; i++) spiderAutoDeal.tableau[i] = [card("3", "clubs", `other-${i}`)];
states.spider_autocomplete_on_deal = spiderAutoDeal;
const spiderAfterDeal = spider.dealFromStock(spiderAutoDeal);
expected.spider_autocomplete_on_deal = { completed_runs: spiderAfterDeal?.completedRuns.length, first_column_length: spiderAfterDeal?.tableau[0].length, stock_length: spiderAfterDeal?.stock.length };

const spiderChain = emptySpider();
const fullRun = ["K", "Q", "J", "10", "9", "8", "7", "6", "5", "4", "3", "2", "A"];
spiderChain.tableau[0] = [...fullRun, ...fullRun].map((rank, index) => card(rank, "spades", `chain-${index}`));
spiderChain.tableau[1] = [card("9", "spades", "chain-move")];
spiderChain.tableau[2] = [card("10", "clubs", "chain-target")];
states.spider_chained_runs_after_move = spiderChain;
const spiderAfterChainMove = spider.moveCards(spiderChain, { zone: "tableau", columnIndex: 1, cardIndex: 0 }, { zone: "tableau", columnIndex: 2 });
expected.spider_chained_runs_after_move = { completed_runs: spiderAfterChainMove?.completedRuns.length, first_column_length: spiderAfterChainMove?.tableau[0].length };

const fixture = {
  oracle: sources,
  oracleGitRevision,
  oracleSourceSHA256: hashes,
  states,
  expected,
};
await mkdir(dirname(output), { recursive: true });
await writeFile(output, `${JSON.stringify(fixture, null, 2)}\n`);
console.log(`Wrote patience fixtures from ${oracleRoot} to ${output}`);
console.log(`Oracle Git revision: ${oracleGitRevision ?? "unavailable"}; source hashes: ${hashes.map((entry) => `${entry.path}=${entry.sha256}`).join(", ")}`);
