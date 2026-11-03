import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

// Rebuild with: node scripts/generate-backgammon-fixtures.ts [../gamevex] [output]
const scriptDir = dirname(fileURLToPath(import.meta.url));
const root = resolve(process.argv[2] ?? "../gamevex");
const output = resolve(process.argv[3] ?? resolve(scriptDir, "../tests/fixtures/backgammon-oracle.json"));
const sourcePath = "packages/shared/src/games/backgammon/engine.ts";
const source = await readFile(resolve(root, sourcePath));
let oracleGitRevision: string | null = null;
try { oracleGitRevision = execFileSync("git", ["-C", root, "rev-parse", "HEAD"], { encoding: "utf8" }).trim(); } catch { /* hash is still exact */ }
const oracle = await import(pathToFileURL(resolve(root, sourcePath)).href);

let state = 0x6d2b79f5;
const random = () => {
  state |= 0;
  state = (state + 0x6d2b79f5) | 0;
  let t = Math.imul(state ^ (state >>> 15), 1 | state);
  t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
  return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
};
const encodeBoard = (b: any) => ({
  p: b.points.map((p: any) => p.color === "white" ? p.count : p.color === "black" ? -p.count : 0),
  b: [b.bar.white, b.bar.black], o: [b.borneOff.white, b.borneOff.black],
});
const encodeMove = (m: any) => [m.from === "bar" ? 24 : m.from, m.to === "off" ? 25 : m.to, m.die, m.isHit];
const seqKey = (s: any[]) => s.map(m => `${m.from}>${m.to}:${m.die}`).join("|");

// Positions come from a deterministic sequence of legal turns, so both colors
// exercise realistic distributions while every expected transition uses the oracle.
const positions: any[] = [];
const opening = oracle.createInitialBoard();
for (let d1 = 1; d1 <= 6; d1++) for (let d2 = 1; d2 <= 6; d2++) {
  const dice = d1 === d2 ? [d1, d1, d1, d1] : [d1, d2];
  for (const color of ["white", "black"]) positions.push({ id: `opening-${d1}-${d2}-${color}`, board: opening, dice, color });
}
let board = oracle.createInitialBoard();
let turn = "white";
for (let ply = 0; ply < 72; ply++) {
  const dice = [1 + Math.floor(random() * 6), 1 + Math.floor(random() * 6)];
  const roll = dice[0] === dice[1] ? [dice[0], dice[0], dice[0], dice[0]] : dice;
  const color = turn;
  const legal = oracle.getAllLegalSequences(board, roll, color);
  if (ply % 2 === 0) {
    for (const sampleColor of ["white", "black"]) {
      const sampleDice = [1 + Math.floor(random() * 6), 1 + Math.floor(random() * 6)];
      positions.push({ id: `mid-${ply}-${sampleColor}`, board, dice: sampleDice[0] === sampleDice[1] ? [sampleDice[0], sampleDice[0], sampleDice[0], sampleDice[0]] : sampleDice, color: sampleColor });
    }
  }
  const sequence = legal[Math.floor(random() * legal.length)] ?? [];
  for (const move of sequence) board = oracle.applyMove(board, move, color);
  turn = color === "white" ? "black" : "white";
}

// Focused bar-entry and hit cases; board populations are intentionally small,
// since the engine API does not require all 15 checkers to be present.
const empty = () => ({ points: Array.from({ length: 24 }, () => ({ color: null, count: 0 })), bar: { white: 0, black: 0 }, borneOff: { white: 0, black: 0 } });
const barBoard = empty(); barBoard.bar.white = 1; barBoard.points[20] = { color: "black", count: 1 }; barBoard.points[0] = { color: "white", count: 2 };
const hitBoard = empty(); hitBoard.points[10] = { color: "white", count: 2 }; hitBoard.points[7] = { color: "black", count: 1 }; hitBoard.points[2] = { color: "black", count: 2 };
positions.push({ id: "bar-entry-hit", board: barBoard, dice: [4, 3], color: "white" });
positions.push({ id: "direct-hit", board: hitBoard, dice: [3, 5], color: "white" });

const excludedCases: { id: string; reason: string }[] = [];
const cases = positions.flatMap(({ id, board: position, dice, color }) => {
  // Keep reference parity away from the source's known overshoot behavior.
  // Once the moving side has borne checkers off and has all remaining checkers
  // home, any bearing-off turn is conservatively tracked outside parity cases.
  const overshootOpportunity = oracle.allInHome(position, color) && dice.some((die: number) => position.points.some((point: any, i: number) => point.color === color && (color === "white" ? i < die : 23 - i < die)));
  if (overshootOpportunity) {
    excludedCases.push({ id, reason: "bearing-off position excluded because oracle overshoot handling differs from standard rule behavior" });
    return [];
  }
  const legalSequences = oracle.getAllLegalSequences(position, dice, color);
  const moveSequences = legalSequences.map((sequence: any[]) => sequence.map(encodeMove));
  const sampledIndices = [...new Set(Array.from({ length: Math.min(8, legalSequences.length) }, (_, i) =>
    Math.floor(i * legalSequences.length / Math.min(8, legalSequences.length))))];
  const transitions = sampledIndices.map(sequenceIndex => {
    const sequence = legalSequences[sequenceIndex];
    let current = position;
    const boards = [encodeBoard(current)];
    for (const move of sequence) { current = oracle.applyMove(current, move, color); boards.push(encodeBoard(current)); }
    return { sequenceIndex, boards };
  });
  // Hard mode consumes randomness only for ties. Sweeping a fine uniform grid
  // recovers each oracle-selected maximum without claiming tie-order parity.
  const bestChoices = new Set<string>();
  for (let i = 0; i < 1000; i++) bestChoices.add(seqKey(oracle.pickCpuMoveSequence(position, dice, color, "hard", () => i / 1000)));
  return [{
    id, board: encodeBoard(position), dice, color,
    allInHome: { white: oracle.allInHome(position, "white"), black: oracle.allInHome(position, "black") },
    pipCount: { white: oracle.pipCount(position, "white"), black: oracle.pipCount(position, "black") },
    legalSequences: moveSequences,
    sampledTransitions: transitions,
    hardBestSequences: [...bestChoices].map(key => key ? key.split("|").map(part => { const [ends, die] = part.split(":"); const [fromRaw, toRaw] = ends.split(">"); const point = (value: string) => value === "bar" ? 24 : value === "off" ? 25 : Number(value); return [point(fromRaw), point(toRaw), Number(die), false]; }) : []),
  }];
});
const fixture = {
  schemaVersion: 1,
  oracle: sourcePath,
  oracleGitRevision,
  sourceSHA256: createHash("sha256").update(source).digest("hex"),
  generation: { seed: "0x6d2b79f5", plies: 72, description: "all 36 opening rolls for both colors, seeded reachable midgame snapshots for both colors, plus focused bar and hit positions" },
  counts: { cases: cases.length, sequences: cases.reduce((n: number, c: any) => n + c.legalSequences.length, 0), transitionSamples: cases.reduce((n: number, c: any) => n + c.sampledTransitions.length, 0) },
  excludedCases,
  knownDivergences: [{ id: "bearing-off-overshoot", status: "excluded-from-parity-cases", reason: "TypeScript oracle permits overshooting any home checker even while a farther checker remains; correct rule behavior is covered separately in the Rust core regression." }],
  cases,
};
await mkdir(dirname(output), { recursive: true });
await writeFile(output, `${JSON.stringify(fixture)}\n`);
console.log(`Wrote ${cases.length} cases to ${output}`);
