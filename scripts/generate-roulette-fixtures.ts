import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const root = resolve(process.argv[2] ?? "../gamevex");
const output = resolve(process.argv[3] ?? resolve(scriptDir, "../tests/fixtures/roulette-oracle.json"));
const sourcePath = "apps/local/src/utils/roulette.ts";
const source = await readFile(resolve(root, sourcePath));
let oracleGitRevision: string | null = null;
try { oracleGitRevision = execFileSync("git", ["-C", root, "rev-parse", "HEAD"], { encoding: "utf8" }).trim(); } catch { /* source hash identifies oracle */ }
const oracle = await import(pathToFileURL(resolve(root, sourcePath)).href);
const types = ["red", "black", "odd", "even", "low", "high", "dozen1", "dozen2", "dozen3"] as const;
const bets = [
  ...Array.from({ length: 37 }, (_, number) => ({ type: "number", number })),
  ...types.map(type => ({ type, number: null })),
];
const cases = bets.flatMap((bet, betIndex) => Array.from({ length: 37 }, (_, number) => {
  const input = { type: bet.type, number: bet.number, stake: 5 };
  const result = { number, color: oracle.getRouletteNumberColor(number) };
  const settlement = oracle.settleRouletteBet(1_000, input, result);
  const betValue = { type: bet.type, ...(bet.number === null ? {} : { number: bet.number }), stake: 5 };
  return {
    id: `bet-${String(betIndex).padStart(2, "0")}-pocket-${String(number).padStart(2, "0")}`,
    bet: betValue,
    number,
    color: result.color,
    label: oracle.getRouletteBetLabel(betValue),
    multiplier: oracle.getRoulettePayoutMultiplier(bet.type),
    expected: settlement,
  };
}));
// The upstream TypeScript oracle has no column bet types. Keep these cases
// explicitly Rust-only and derive their outcomes from the requested roulette
// column definition, rather than presenting them as oracle comparisons.
const rustOnlyColumnCases = (["column1", "column2", "column3"] as const).flatMap((type, columnIndex) =>
  Array.from({ length: 37 }, (_, number) => {
    const won = number !== 0 && number % 3 === (columnIndex + 1) % 3;
    const multiplier = 2;
    const payout = won ? 5 * (multiplier + 1) : 0;
    const net = payout - 5;
    return {
      bet: type,
      number,
      label: `COLUMN ${columnIndex + 1}`,
      multiplier,
      won,
      payout,
      net,
      balance: 1_000 + net,
    };
  }),
);
const fixture = {
  schemaVersion: 1,
  oracle: sourcePath,
  oracleGitRevision,
  sourceSHA256: createHash("sha256").update(source).digest("hex"),
  counts: { bets: bets.length, pockets: 37, cases: cases.length, balance: 1_000, stake: 5 },
  cases,
  rustOnlyColumnCases,
};
await mkdir(dirname(output), { recursive: true });
await writeFile(output, `${JSON.stringify(fixture, null, 2)}\n`);
console.log(`Wrote ${cases.length} cases to ${output}`);
