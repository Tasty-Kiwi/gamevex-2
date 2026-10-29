import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

const oracleRoot = resolve(process.argv[2] ?? "../gamevex");
const sudokuPath = resolve(oracleRoot, "packages/shared/src/puzzle/sudoku.ts");
const hintsPath = resolve(oracleRoot, "packages/shared/src/puzzle/sudokuHints.ts");
const sudokuSource = await readFile(sudokuPath);
const hintsSource = await readFile(hintsPath);
const sourceSHA256 = createHash("sha256").update(sudokuSource).update(hintsSource).digest("hex");
const { generatePuzzle, hasConflictAt, isSolvedGrid } = await import(`${sudokuPath}`);
const { findSudokuHint } = await import(`${hintsPath}`);

const seededRandom = (seed: number) => {
  let state = seed >>> 0;
  return () => {
    state = (Math.imul(state, 1664525) + 1013904223) >>> 0;
    return state / 0x1_0000_0000;
  };
};
const withSeed = <T,>(seed: number, run: () => T): T => {
  const originalRandom = Math.random;
  Math.random = seededRandom(seed);
  try { return run(); } finally { Math.random = originalRandom; }
};
const sizes = [4, 6, 9, 16] as const;
const difficulties = ["easy", "medium", "hard", "expert"] as const;
const generated: unknown[] = [];
for (const size of sizes) for (const difficulty of difficulties) {
  const seeds = size === 16 ? [42] : [8, 42];
  for (const inputSeed of seeds) {
    const puzzle = withSeed(inputSeed, () => generatePuzzle(size, difficulty));
    generated.push({ size, difficulty, inputSeed, puzzle: puzzle.puzzle, solution: puzzle.solution });
  }
}

const solutions = [4, 6, 9].map((size) => ({ size, solution: withSeed(42, () => generatePuzzle(size, "medium")).solution }));
const categories = ["naked", "hiddenRow", "hiddenColumn", "hiddenBox"] as const;
const hintCases: Record<string, unknown> = {};
for (const category of categories) {
  let found: { board: number[][]; hint: unknown } | undefined;
  let foundSize = 0;
  for (let searchSeed = 1; searchSeed < 30_000 && !found; searchSeed++) for (const { size, solution } of solutions) {
    const random = seededRandom(searchSeed + size * 31);
    const board = solution.map((row) => row.map((v) => random() < 0.5 ? v : 0));
    const hint = findSudokuHint(board, size);
    if (!hint) continue;
    const explanation = hint.explanation;
    const actual = explanation.includes("only one legal value") ? "naked"
      : explanation.includes("only one cell in row") ? "hiddenRow"
      : explanation.includes("only one cell in column") ? "hiddenColumn"
      : explanation.includes("only one cell in this box") ? "hiddenBox" : "";
    if (actual === category) { found = { board, hint }; foundSize = size; break; }
  }
  if (!found) throw new Error(`Could not discover ${category} hint fixture`);
  hintCases[category] = { size: foundSize, board: found.board, hint: found.hint };
}

const solution = solutions[0].solution;
const conflictBoard = solution.map((row) => [...row]);
conflictBoard[0][0] = conflictBoard[0][1];
const impossibleBoard = Array.from({ length: 4 }, () => Array(4).fill(0));
// A conflict-free partial board where one empty cell has no candidate.
// Search deterministic random valid partial boards rather than hand-coding a fragile pattern.
let impossibleFound = false;
for (let seed = 1; seed < 100_000 && !impossibleFound; seed++) {
  const random = seededRandom(seed);
  const board = Array.from({ length: 4 }, () => Array(4).fill(0));
  const positions = Array.from({ length: 16 }, (_, i) => i).sort(() => random() - 0.5);
  for (const index of positions) {
    if (random() > 0.5) continue;
    const r = Math.floor(index / 4), c = index % 4;
    const values = [1, 2, 3, 4].sort(() => random() - 0.5);
    for (const value of values) {
      board[r][c] = value;
      if (!hasConflictAt(board, r, c, 4)) break;
      board[r][c] = 0;
    }
  }
  if (board.some((row) => row.some((value) => value === 0)) && findSudokuHint(board, 4) === null) {
    const hasZeroCandidate = board.some((row, r) => row.some((value, c) => value === 0 &&
      !Array.from({ length: 4 }, (_, i) => i + 1).some((n) => {
        for (let i = 0; i < 4; i++) if (board[r][i] === n || board[i][c] === n) return false;
        const sr = r - r % 2, sc = c - c % 2;
        for (let br = sr; br < sr + 2; br++) for (let bc = sc; bc < sc + 2; bc++) if (board[br][bc] === n) return false;
        return true;
      })));
    if (hasZeroCandidate) { Object.assign(impossibleBoard, board); impossibleFound = true; }
  }
}
if (!impossibleFound) throw new Error("Could not discover impossible-board fixture");

hintCases.noHint = { size: 4, board: solution, hint: findSudokuHint(solution, 4) };
hintCases.conflict = { size: 4, board: conflictBoard, hint: findSudokuHint(conflictBoard, 4) };
hintCases.impossible = { size: 4, board: impossibleBoard, hint: findSudokuHint(impossibleBoard, 4) };
const sixteen = withSeed(42, () => generatePuzzle(16, "medium")).puzzle;
hintCases.unsupported16 = { size: 16, board: sixteen, hint: findSudokuHint(sixteen, 16) };

const checks = [
  { name: "solved-4x4", size: 4, board: solution, solved: isSolvedGrid(solution, 4), conflictAt: null },
  { name: "duplicate-row-4x4", size: 4, board: conflictBoard, solved: isSolvedGrid(conflictBoard, 4), conflictAt: 0, conflict: hasConflictAt(conflictBoard, 0, 0, 4) },
  { name: "empty-4x4", size: 4, board: impossibleBoard.map((row) => row.map(() => 0)), solved: isSolvedGrid(impossibleBoard.map((row) => row.map(() => 0)), 4), conflictAt: 0, conflict: hasConflictAt(impossibleBoard, 0, 0, 4) },
];
const out = { schemaVersion: 1, sourceSHA256, oracle: "packages/shared/src/puzzle/sudoku.ts + sudokuHints.ts", generated, checks, hintCases };
const output = resolve("tests/fixtures/sudoku-parity.json");
await mkdir(resolve(output, ".."), { recursive: true });
await writeFile(output, `${JSON.stringify(out)}\n`);
console.log(`Wrote ${generated.length} generated cases and ${Object.keys(hintCases).length} hint cases to ${output}`);
