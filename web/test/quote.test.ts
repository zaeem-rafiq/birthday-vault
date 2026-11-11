import { afterEach, describe, expect, it, vi } from "vitest";
import fc from "fast-check";
import {
  CHAINS,
  QUOTE_ADDRESS_MESSAGE,
  QUOTE_SLOW_MESSAGE,
  fetchPreviewQuote,
  previewQuoteBody,
  quoteRecipient,
  validRefund,
} from "@/lib/quote";

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

describe("quote failures", () => {
  const body = () => previewQuoteBody(CHAINS[0], 5, "0x1111111111111111111111111111111111111111", "bv-1.bv.test.near");
  const reply = (status: number, json: unknown) =>
    vi.stubGlobal("fetch", vi.fn(async () => new Response(JSON.stringify(json), { status })));

  afterEach(() => vi.unstubAllGlobals());

  it("asks to check the refund address when 1Click refuses it", async () => {
    reply(400, { message: "Internal server error" });
    await expect(fetchPreviewQuote(body())).rejects.toThrow(QUOTE_ADDRESS_MESSAGE);
    reply(400, { message: "refundTo is not valid" });
    await expect(fetchPreviewQuote(body())).rejects.toThrow(QUOTE_ADDRESS_MESSAGE);
  });

  it("says the service is slow on a server error or a timeout", async () => {
    reply(500, { message: "Failed to receive response within timeout" });
    await expect(fetchPreviewQuote(body())).rejects.toThrow(QUOTE_SLOW_MESSAGE);
    vi.stubGlobal("fetch", vi.fn((_u: string, init: RequestInit) => new Promise((_r, reject) => {
      init.signal?.addEventListener("abort", () => reject(new DOMException("aborted", "AbortError")));
    })));
    await expect(fetchPreviewQuote(body(), 50)).rejects.toThrow(QUOTE_SLOW_MESSAGE);
  });
});
