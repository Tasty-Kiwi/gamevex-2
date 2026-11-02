import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const root = resolve(process.argv[2] ?? "../gamevex");
const output = resolve(process.argv[3] ?? resolve(scriptDir, "../tests/fixtures/holdem-oracle.json"));
const oraclePath = "packages/shared/src/games/holdem/engine.ts";
const source = await readFile(resolve(root, oraclePath));
const sourceSHA256 = createHash("sha256").update(source).digest("hex");
let oracleGitRevision: string | null = null;
try { oracleGitRevision = execFileSync("git", ["-C", root, "rev-parse", "HEAD"], { encoding: "utf8" }).trim(); } catch { /* source hash identifies the oracle */ }
const oracle = await import(pathToFileURL(resolve(root, oraclePath)).href);
const suits = ["clubs", "diamonds", "hearts", "spades"] as const;
type Face = { rank: number; suit: typeof suits[number] };
const all: Face[] = suits.flatMap(suit => Array.from({length:13},(_,i)=>({rank:i+2,suit})));
const rankLabel = (rank:number) => rank===14?"A":rank===13?"K":rank===12?"Q":rank===11?"J":String(rank);
const faceId = (c:Face) => `${rankLabel(c.rank)}-${c.suit}`;
const toOracle = (c:Face) => ({rank:rankLabel(c.rank),suit:c.suit,id:faceId(c)});
const toRust = (c:Face) => ({rank:c.rank,suit:c.suit[0].toUpperCase()+c.suit.slice(1),face_up:true});
const oracleToRust = (c:any) => ({rank:c.rank==="A"?14:c.rank==="K"?13:c.rank==="Q"?12:c.rank==="J"?11:Number(c.rank),suit:c.suit[0].toUpperCase()+c.suit.slice(1),face_up:true});
let state=0x484f4c44;
const random=()=>{state^=state<<13;state^=state>>>17;state^=state<<5;return state>>>0;};
const sample=(pool:Face[],n:number)=>{const copy=[...pool],out:Face[]=[];while(out.length<n)out.push(copy.splice(random()%copy.length,1)[0]);return out;};
const special: Face[][] = [
  [14,11,9,7,3].map((rank,i)=>({rank,suit:["spades","hearts","diamonds","clubs","spades"][i] as Face["suit"]})),
  [{rank:14,suit:"clubs"},{rank:14,suit:"diamonds"},{rank:9,suit:"hearts"},{rank:7,suit:"spades"},{rank:3,suit:"clubs"}],
  [{rank:14,suit:"clubs"},{rank:14,suit:"diamonds"},{rank:9,suit:"hearts"},{rank:9,suit:"spades"},{rank:3,suit:"clubs"}],
  [{rank:14,suit:"clubs"},{rank:14,suit:"diamonds"},{rank:14,suit:"hearts"},{rank:9,suit:"spades"},{rank:3,suit:"clubs"}],
  [10,9,8,7,6].map(rank=>({rank,suit:"spades"})),
  [14,11,9,7,3].map(rank=>({rank,suit:"spades"})),
  [{rank:14,suit:"clubs"},{rank:14,suit:"diamonds"},{rank:14,suit:"hearts"},{rank:9,suit:"clubs"},{rank:9,suit:"spades"}],
  [{rank:14,suit:"clubs"},{rank:14,suit:"diamonds"},{rank:14,suit:"hearts"},{rank:14,suit:"spades"},{rank:9,suit:"clubs"}],
  [10,9,8,7,6].map(rank=>({rank,suit:"spades"})),
  [{rank:14,suit:"clubs"},{rank:14,suit:"diamonds"},{rank:14,suit:"hearts"},{rank:13,suit:"clubs"},{rank:13,suit:"diamonds"},{rank:13,suit:"hearts"},{rank:2,suit:"spades"}],
  [{rank:14,suit:"clubs"},{rank:2,suit:"diamonds"},{rank:3,suit:"hearts"},{rank:4,suit:"clubs"},{rank:5,suit:"spades"}],
];
const cases: {id:string;cards:Face[]}[] = special.map((cards,i)=>({id:`known-${String(i).padStart(2,"0")}`,cards}));
while(cases.length<special.length+512) cases.push({id:`random-${String(cases.length-special.length).padStart(3,"0")}`,cards:sample(all,7)});
const handCases=cases.map(({id,cards})=>{
  const result=oracle.getBestHandEvaluation(cards.map(toOracle));
  if(!result) throw new Error(`Oracle rejected ${id}`);
  const byId=new Map(cards.map(face=>[faceId(face),face]));
  return {id,cards:cards.map(toRust),expected:{score:result.score,cards:result.cards.map(oracleToRust),highlights:result.highlightIds.map((x:string)=>toRust(byId.get(x)!)),label:result.label}};
});
const cpuCases:any[]=[];
const originalRandom=Math.random;
const phases=["preflop","flop","turn","river"] as const;
for(let index=0;index<128;index++){
  let randomState=(0x43505531 ^ Math.imul(index+1,0x9e3779b1))>>>0;
  Math.random=()=>{randomState^=randomState<<13;randomState^=randomState>>>17;randomState^=randomState<<5;return (randomState>>>0)/0x1_0000_0000;};
  let table=oracle.createIdleHoldemTable(4);
  table=oracle.startHoldemHand(table,200);
  const phase=phases[index%phases.length];
  let actor=3;
  if(phase!=="preflop"){
    const count=phase==="flop"?3:phase==="turn"?4:5;
    while(table.board.length<count){table.burnedCards.push(table.deck.pop()!);const card=table.deck.pop();if(card)table.board.push(card);}
    table.phase=phase;
    actor=1;
    table.currentBet=10;
    table.lastFullRaise=10;
    for(const seat of table.seats){seat.streetCommitted=0;seat.canRaise=!seat.folded&&!seat.isAllIn;}
    table.seats[2].streetCommitted=10;
    table.actingSeatIndex=actor;
    table.actionQueue=[actor,2,3,4,0];
  }
  const toCall=table.currentBet-table.seats[actor].streetCommitted;
  const pot=table.seats.reduce((sum:number,seat:any)=>sum+seat.handCommitted,0);
  const pressure=toCall/Math.max(1,pot+toCall);
  if(pressure<0.32) throw new Error(`unexpected random CPU branch pressure ${pressure} in ${index}`);
  const action=oracle.pickAiAction(table,actor,()=>0.25);
  const after=oracle.applyHoldemAction(table,action);
  const cardArray=(list:any[])=>list.map(c=>toRust({rank:c.rank==="A"?14:c.rank==="K"?13:c.rank==="Q"?12:c.rank==="J"?11:Number(c.rank),suit:c.suit}));
  cpuCases.push({id:`cpu-${String(index).padStart(3,"0")}`,phase,actor,pressure,cards:{board:cardArray(table.board),deck:cardArray(table.deck),burned:cardArray(table.burnedCards)},table:{buyIn:table.buyIn,dealerIndex:table.dealerIndex,smallBlindIndex:table.smallBlindIndex,bigBlindIndex:table.bigBlindIndex,actingSeatIndex:table.actingSeatIndex,actionQueue:table.actionQueue,currentBet:table.currentBet,lastFullRaise:table.lastFullRaise,handNumber:table.handNumber,seats:table.seats.map((seat:any)=>({stack:seat.stack,holeCards:cardArray(seat.holeCards),folded:seat.folded,isAllIn:seat.isAllIn,streetCommitted:seat.streetCommitted,handCommitted:seat.handCommitted,canRaise:seat.canRaise,lastAction:seat.lastAction??""}))},expected:{action:{type:action.type,total:action.total??null},phase:after.phase,actingSeatIndex:after.actingSeatIndex,actionQueue:after.actionQueue,currentBet:after.currentBet,lastFullRaise:after.lastFullRaise,board:cardArray(after.board),pot:after.seats.reduce((sum:number,seat:any)=>sum+seat.handCommitted,0),seats:after.seats.map((seat:any)=>({stack:seat.stack,folded:seat.folded,streetCommitted:seat.streetCommitted,handCommitted:seat.handCommitted,canRaise:seat.canRaise,lastAction:seat.lastAction??""}))}});
}
Math.random=originalRandom;
const transitionCases:any[]=[];
const transitionActions=["check","call","fold","bet"] as const;
for(let index=0;index<32;index++){
  let randomState=(0x54524e53 ^ Math.imul(index+1,0x9e3779b1))>>>0;
  Math.random=()=>{randomState^=randomState<<13;randomState^=randomState>>>17;randomState^=randomState<<5;return (randomState>>>0)/0x1_0000_0000;};
  let table=oracle.startHoldemHand(oracle.createIdleHoldemTable(4),200);
  while(table.board.length<3){table.burnedCards.push(table.deck.pop()!);const card=table.deck.pop();if(card)table.board.push(card);}
  table.phase="flop";
  const kind=transitionActions[index%4];
  const actor=0;
  for(const seat of table.seats){seat.streetCommitted=0;seat.canRaise=!seat.folded&&!seat.isAllIn;}
  table.currentBet=0;
  if(kind==="call"||kind==="fold") { table.currentBet=10;table.seats[2].streetCommitted=10; }
  table.actingSeatIndex=actor;
  table.actionQueue=[actor,1,2,3,4];
  const action={type:kind,...(kind==="bet"?{total:20}:{})};
  const after=oracle.applyHoldemAction(table,action);
  const cardArray=(list:any[])=>list.map(c=>toRust({rank:c.rank==="A"?14:c.rank==="K"?13:c.rank==="Q"?12:c.rank==="J"?11:Number(c.rank),suit:c.suit}));
  const seatArray=(seats:any[])=>seats.map(seat=>({stack:seat.stack,holeCards:cardArray(seat.holeCards),folded:seat.folded,isAllIn:seat.isAllIn,streetCommitted:seat.streetCommitted,handCommitted:seat.handCommitted,canRaise:seat.canRaise,lastAction:seat.lastAction??""}));
  const input={buyIn:table.buyIn,dealerIndex:table.dealerIndex,smallBlindIndex:table.smallBlindIndex,bigBlindIndex:table.bigBlindIndex,actingSeatIndex:table.actingSeatIndex,actionQueue:table.actionQueue,currentBet:table.currentBet,lastFullRaise:table.lastFullRaise,handNumber:table.handNumber,seats:seatArray(table.seats)};
  const expected={phase:after.phase,actingSeatIndex:after.actingSeatIndex,actionQueue:after.actionQueue,currentBet:after.currentBet,lastFullRaise:after.lastFullRaise,board:cardArray(after.board),pot:after.seats.reduce((sum:number,seat:any)=>sum+seat.handCommitted,0),seats:seatArray(after.seats)};
  transitionCases.push({id:`transition-${String(index).padStart(2,"0")}`,action,board:cardArray(table.board),deck:cardArray(table.deck),burned:cardArray(table.burnedCards),phase:table.phase,table:input,expected});
}
Math.random=originalRandom;
const fixture={schemaVersion:1,oracle:oraclePath,oracleGitRevision,sourceSHA256,counts:{knownHands:special.length,randomSevenCardHands:512,cpuActionCases:cpuCases.length,transitionCases:transitionCases.length,cpuBranchPolicy:"randomized branches excluded by call pressure >= 0.32"},handCases,cpuCases,transitionCases};
await mkdir(dirname(output),{recursive:true});
await writeFile(output,`${JSON.stringify(fixture,null,2)}\n`);
console.log(`Wrote ${handCases.length} Hold'em cases to ${output}`);
console.log(`Oracle revision ${oracleGitRevision??"unavailable"}; SHA-256 ${sourceSHA256}`);
