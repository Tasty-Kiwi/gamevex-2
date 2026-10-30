import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const oracleRoot = resolve(process.argv[2] ?? "../gamevex");
const enginePath = resolve(oracleRoot, "packages/shared/src/games/mahjong/engine.ts");
const output = resolve(process.argv[3] ?? resolve(scriptDir, "../tests/fixtures/mahjong-oracle.json"));
const engineSource = await readFile(enginePath);
const oracleSourceSHA256 = createHash("sha256").update(engineSource).digest("hex");
let oracleGitRevision: string | null = null;
try {
  oracleGitRevision = execFileSync("git", ["-C", oracleRoot, "rev-parse", "HEAD"], { encoding: "utf8" }).trim();
} catch {
  // The source hash still identifies the exact engine when no Git checkout is available.
}

const { getMahjongLayoutCoordinates, analyzeBoard, doTilesMatch } = await import(pathToFileURL(enginePath).href);
const layoutNames = ["turtle", "arena", "monument"] as const;
const layouts = Object.fromEntries(layoutNames.map((layout) => [layout, getMahjongLayoutCoordinates(layout)]));

const faces = Array.from({ length: 42 }, (_, face) => {
  if (face < 9) return { suit: "dots", value: face + 1 };
  if (face < 18) return { suit: "bamboo", value: face - 8 };
  if (face < 27) return { suit: "characters", value: face - 17 };
  if (face < 31) return { suit: "winds", value: ["east", "south", "west", "north"][face - 27] };
  if (face < 34) return { suit: "dragons", value: ["red", "green", "white"][face - 31] };
  if (face < 38) return { suit: "flowers", value: ["plum", "orchid", "chrysanthemum", "bamboo"][face - 34] };
  return { suit: "seasons", value: ["spring", "summer", "autumn", "winter"][face - 38] };
});
const node = (id: string, face: number, x: number, y: number, z: number) => ({
  id,
  tileData: { id, suit: faces[face].suit, value: faces[face].value },
  x,
  y,
  z,
  state: "idle" as const,
});
const matchMatrix = faces.map((_, a) =>
  faces.map((__, b) => doTilesMatch(node("a", a, 0, 0, 0), node("b", b, 2, 0, 0))),
);

const blockerInputs = [
  { name: "open", blockers: [] },
  { name: "left_single_blocker", blockers: [[-2, 0, 0]] },
  { name: "right_single_blocker", blockers: [[2, 0, 0]] },
  { name: "both_sides_blocked", blockers: [[-2, 0, 0], [2, 0, 0]] },
  { name: "half_offset_left_and_right", blockers: [[-1, -1, 0], [1, 1, 0]] },
  { name: "left_half_offset_only", blockers: [[-1, 1, 0]] },
  { name: "above_same_xy", blockers: [[0, 0, 1]] },
  { name: "above_half_offsets", blockers: [[-1, 1, 1]] },
  { name: "higher_stack_above", blockers: [[1, -1, 2]] },
  { name: "higher_same_xy", blockers: [[0, 0, 3]] },
  { name: "other_row_does_not_block", blockers: [[0, 2, 0]] },
  { name: "distant_above_does_not_block", blockers: [[2, 0, 1]] },
];
const selectabilityCases = blockerInputs.map(({ name, blockers }) => {
  const board = [node("target", 0, 10, 10, 0), ...blockers.map(([dx, dy, z], i) => node(`b${i}`, 1, 10 + dx, 10 + dy, z))];
  return {
    name,
    tiles: board.map(({ x, y, z }) => ({ x, y, z })),
    expected: analyzeBoard(board).metrics.get("target")!.isSelectable,
  };
});

const fixture = {
  oracle: "packages/shared/src/games/mahjong/engine.ts",
  oracleGitRevision,
  oracleSourceSHA256,
  layouts,
  faces,
  match_matrix: matchMatrix,
  selectability_cases: selectabilityCases,
};
await mkdir(dirname(output), { recursive: true });
await writeFile(output, `${JSON.stringify(fixture, null, 2)}\n`);
console.log(`Wrote Mahjong oracle fixture from ${enginePath} to ${output}`);
console.log(`Oracle Git revision: ${oracleGitRevision ?? "unavailable"}; source SHA-256: ${oracleSourceSHA256}`);
