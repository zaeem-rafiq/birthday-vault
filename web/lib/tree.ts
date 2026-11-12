// The tree is a pure function of the vault's balance (PLAN.md section 7).
// Stages start at $0, $1, $5, $25 and $100. Balances are in play-USDC units (6 decimals).

export const STAGES = [
  { min: 0, name: "Seed" },
  { min: 1, name: "Sprout" },
  { min: 5, name: "Sapling" },
  { min: 25, name: "Young tree" },
  { min: 100, name: "Tree" },
] as const;

const UNIT = BigInt(1_000_000);

/** Returns the stage index, 0 to 4, for a balance in 6-decimal units. */
export function treeStage(balanceUnits: bigint): number {
  const dollars = balanceUnits < BigInt(0) ? BigInt(0) : balanceUnits / UNIT;
  let stage = 0;
  STAGES.forEach((s, i) => {
    if (dollars >= BigInt(s.min)) stage = i;
  });
  return stage;
}

/** Formats 6-decimal units as dollars, for example 1500000 -> "$1.50". */
export function formatDollars(units: bigint): string {
  const neg = units < BigInt(0);
  const abs = neg ? -units : units;
  const whole = abs / UNIT;
  const cents = (abs % UNIT) / BigInt(10_000);
  return `${neg ? "-" : ""}$${whole.toLocaleString("en-US")}.${cents.toString().padStart(2, "0")}`;
}
