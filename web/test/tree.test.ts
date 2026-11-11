import { describe, expect, it } from "vitest";
import fc from "fast-check";
import { formatDollars, treeStage } from "@/lib/tree";

const units = fc.bigInt({ min: BigInt(0), max: BigInt(10) ** BigInt(12) });

describe("tree", () => {
  it("starts each stage at $0, $1, $5, $25 and $100", () => {
    const at = (d: number) => treeStage(BigInt(Math.round(d * 1_000_000)));
    expect([at(0), at(0.99), at(1), at(4.99), at(5), at(24.99), at(25), at(99.99), at(100), at(1e6)]).toEqual(
      [0, 0, 1, 1, 2, 2, 3, 3, 4, 4],
    );
  });

  it("never shrinks when the balance grows", () => {
    fc.assert(
      fc.property(units, units, (a, b) => {
        const [lo, hi] = a <= b ? [a, b] : [b, a];
        return treeStage(lo) <= treeStage(hi);
      }),
    );
  });

  it("always returns a stage from 0 to 4", () => {
    fc.assert(fc.property(fc.bigInt(), (u) => treeStage(u) >= 0 && treeStage(u) <= 4));
  });

  it("formats play-money units as dollars", () => {
    expect(formatDollars(BigInt(0))).toBe("$0.00");
    expect(formatDollars(BigInt(1_500_000))).toBe("$1.50");
    expect(formatDollars(BigInt(1_234_567_890))).toBe("$1,234.56");
  });
});
