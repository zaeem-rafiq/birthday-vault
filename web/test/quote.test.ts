import { describe, expect, it } from "vitest";
import fc from "fast-check";
import { CHAINS, previewQuoteBody, quoteRecipient, validRefund } from "@/lib/quote";

describe("1Click preview quotes", () => {
  it("never sends a testnet or sandbox id to 1Click", () => {
    fc.assert(
      fc.property(fc.stringMatching(/^[a-z0-9-]{1,20}$/), fc.constantFrom(".testnet", ".test.near", ".sandbox"), (name, suffix) => {
        const body = previewQuoteBody(CHAINS[0], 5, "0x1111111111111111111111111111111111111111", `bv-1.${name}${suffix}`);
        return body.recipient === "wrap.near" && !body.recipient.endsWith(suffix);
      }),
    );
  });

  it("names a mainnet vault as itself", () => {
    expect(quoteRecipient("bv-1.g1.alice.near")).toBe("bv-1.g1.alice.near");
  });

  it("only ever asks for dry quotes", () => {
    const body = previewQuoteBody(CHAINS[3], 1, "So11111111111111111111111111111111111111112", "bv-1.bv.test.near");
    expect(body.dry).toBe(true);
    expect(body.amount).toBe("1000000");
    expect(body.swapType).toBe("EXACT_INPUT");
    expect(body.slippageTolerance).toBe(100);
  });

  it("checks refund addresses for the chosen chain", () => {
    expect(validRefund(CHAINS[0], "0xAbCDEF0123456789abcdef0123456789ABCDEF01")).toBe("0xabcdef0123456789abcdef0123456789abcdef01");
    expect(validRefund(CHAINS[0], "0x123")).toBeNull();
    expect(validRefund(CHAINS[3], "0x1111111111111111111111111111111111111111")).toBeNull();
    expect(validRefund(CHAINS[3], "So11111111111111111111111111111111111111112")).not.toBeNull();
  });
});
