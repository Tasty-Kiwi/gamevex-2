import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const oracleRoot = resolve(process.argv[2] ?? "../gamevex");
const output = resolve(process.argv[3] ?? resolve(scriptDir, "../tests/fixtures/cribbage-oracle.json"));
const oraclePath = "apps/local/src/utils/cribbage.ts";
const source = await readFile(resolve(oracleRoot, oraclePath));
const sourceSHA256 = createHash("sha256").update(source).digest("hex");
let oracleGitRevision: string | null = null;
try { oracleGitRevision = execFileSync("git", ["-C", oracleRoot, "rev-parse", "HEAD"], { encoding: "utf8" }).trim(); } catch { /* source hash still identifies the oracle */ }
const oracle = await import(pathToFileURL(resolve(oracleRoot, oraclePath)).href);
const suits = ["clubs", "diamonds", "hearts", "spades"] as const;
const rankLabels = ["A", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K"] as const;
type Face = { rank: number; suit: string };
const all: Face[] = suits.flatMap((suit) => rankLabels.map((label) => ({ rank: label === "A" ? 14 : label === "J" ? 11 : label === "Q" ? 12 : label === "K" ? 13 : Number(label), suit })));
const label = (rank: number) => rank === 14 ? "A" : rank === 11 ? "J" : rank === 12 ? "Q" : rank === 13 ? "K" : String(rank);
const toOracle = (c: Face) => ({ rank: label(c.rank), suit: c.suit, id: `${label(c.rank)}-${c.suit}` });
const toRust = (c: Face) => ({ rank: c.rank, suit: c.suit[0].toUpperCase() + c.suit.slice(1), face_up: true });
const cardId = (c: Face) => `${c.rank}-${c.suit}`;
let state = 0x43524942;
const random = () => { state ^= state << 13; state ^= state >>> 17; state ^= state << 5; return state >>> 0; };
const sample = (pool: Face[], n: number): Face[] => {
  const copy = [...pool]; const result: Face[] = [];
  while (result.length < n) result.push(copy.splice(random() % copy.length, 1)[0]);
  return result;
};

const hands: any[] = [];
const specialHands: { hand: Face[]; starter: Face; isCrib: boolean }[] = [
  { hand: [{rank:5,suit:"clubs"},{rank:5,suit:"diamonds"},{rank:5,suit:"hearts"},{rank:11,suit:"spades"}], starter:{rank:5,suit:"spades"}, isCrib:false }, // 29
  { hand: [{rank:2,suit:"clubs"},{rank:3,suit:"diamonds"},{rank:3,suit:"hearts"},{rank:4,suit:"spades"}], starter:{rank:4,suit:"clubs"}, isCrib:false }, // double run
  { hand: [{rank:11,suit:"clubs"},{rank:2,suit:"diamonds"},{rank:7,suit:"hearts"},{rank:9,suit:"spades"}], starter:{rank:4,suit:"clubs"}, isCrib:false }, // nobs
  { hand: [{rank:2,suit:"clubs"},{rank:4,suit:"clubs"},{rank:7,suit:"clubs"},{rank:9,suit:"clubs"}], starter:{rank:13,suit:"clubs"}, isCrib:true }, // crib flush
  { hand: [{rank:2,suit:"clubs"},{rank:4,suit:"clubs"},{rank:7,suit:"clubs"},{rank:9,suit:"clubs"}], starter:{rank:13,suit:"spades"}, isCrib:false }, // hand flush
];
for (const item of specialHands) hands.push(item);
while (hands.length < 128) {
  const five = sample(all, 5);
  hands.push({ hand: five.slice(0, 4), starter: five[4], isCrib: random() % 2 === 0 });
}
const handCases = hands.map(({ hand, starter, isCrib }, index) => {
  const result = oracle.scoreCribbageHand(hand.map(toOracle), toOracle(starter), isCrib);
  return { id: `hand-${String(index).padStart(3, "0")}`, hand: hand.map(toRust), starter: toRust(starter), is_crib: isCrib, expected: result };
});

const discardCases: any[] = [];
for (let index = 0; index < 128; index++) {
  const six = sample(all, 6);
  for (const dealer of [0, 1]) {
    const result = oracle.chooseCpuDiscard(six.map(toOracle), dealer === 1);
    discardCases.push({ id: `discard-${String(index).padStart(3, "0")}-dealer-${dealer}`, dealer, hand: six.map(toRust), keep: result.keep.map((c: any) => toRust({ rank: c.rank === "A" ? 14 : c.rank === "J" ? 11 : c.rank === "Q" ? 12 : c.rank === "K" ? 13 : Number(c.rank), suit: c.suit })), discard: result.discard.map((c: any) => toRust({ rank: c.rank === "A" ? 14 : c.rank === "J" ? 11 : c.rank === "Q" ? 12 : c.rank === "K" ? 13 : Number(c.rank), suit: c.suit })) });
  }
}

const pegCases: any[] = [];
for (let index = 0; index < 128; index++) {
  let seq: Face[] = []; let count = 0;
  const sequenceDeck = sample(all, 12);
  for (const c of sequenceDeck) { if (count + oracle.getCribbageCardValue(toOracle(c)) <= 27 && seq.length < 7) { seq.push(c); count += oracle.getCribbageCardValue(toOracle(c)); } }
  if (count === 0 || seq.length === 0) throw new Error("empty generated pegging sequence");
  const used = new Set(seq.map(cardId));
  const legalPool = all.filter(c => !used.has(cardId(c)) && count + (c.rank === 14 ? 1 : Math.min(c.rank,10)) <= 31);
  const rest = all.filter(c => !used.has(cardId(c)));
  const lead = sample(legalPool, 1);
  const hand = lead.concat(sample(rest.filter(c => !lead.some(x => cardId(x) === cardId(c))), 5));
  if (hand.some(c => !c)) throw new Error(`invalid generated peg hand ${index}: ${JSON.stringify({count,seq,legalPool:legalPool.length,rest:rest.length,hand})}`);
  const chosen = oracle.chooseCpuPegCard(hand.map(toOracle), seq.map(toOracle), count);
  if (!chosen) throw new Error("oracle unexpectedly has no legal CPU pegging card");
  const expectedFace = all.find(c => toOracle(c).id === chosen.id)!;
  pegCases.push({ id: `peg-${String(index).padStart(3,"0")}`, sequence: seq.map(toRust), count, hand: hand.map(toRust), expected: toRust(expectedFace) });
}

const fixture = { schemaVersion: 1, oracle: oraclePath, oracleGitRevision, sourceSHA256, counts: { hands: handCases.length, cpuDiscards: discardCases.length, cpuPegging: pegCases.length }, handCases, discardCases, pegCases };
await mkdir(dirname(output), { recursive: true });
await writeFile(output, `${JSON.stringify(fixture, null, 2)}\n`);
console.log(`Wrote ${handCases.length} hands, ${discardCases.length} CPU discards, and ${pegCases.length} CPU pegging cases to ${output}`);
console.log(`Oracle revision ${oracleGitRevision ?? "unavailable"}; SHA-256 ${sourceSHA256}`);
