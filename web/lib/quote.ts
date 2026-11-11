// No-money preview quotes from the NEAR Intents 1Click API (PLAN.md section 7, gift page).
// This site only ever asks for `dry: true` quotes. It never requests a deposit address,
// because real gifts are off on the public site and the demo runs on testnet.
import { config } from "./config";
import { isMainnetId } from "./network";

export const CHAINS = [
  { key: "base", name: "Base", asset: "nep141:base-0x833589fcd6edb6e08f4c7c32d4f71b54bda02913.omft.near", kind: "evm" },
  { key: "arb", name: "Arbitrum", asset: "nep141:arb-0xaf88d065e77c8cc2239327c5edb3a432268e5831.omft.near", kind: "evm" },
  { key: "eth", name: "Ethereum", asset: "nep141:eth-0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48.omft.near", kind: "evm" },
  { key: "sol", name: "Solana", asset: "nep141:sol-5ce3bf3a31af18be40ba30f721101b4341690186.omft.near", kind: "solana" },
] as const;

export type Chain = (typeof CHAINS)[number];

export const NEAR_USDC = "nep141:17208628f84f5d6ad33f0da3bbbeb27ffcb398eac501a31bd6ad2011e36133a1";

/** Normalizes a refund address for the chain, or returns null if it is not valid there. */
export function validRefund(chain: Chain, address: string): string | null {
  const a = address.trim();
  if (chain.kind === "evm") return /^0x[0-9a-fA-F]{40}$/.test(a) ? a.toLowerCase() : null;
  return /^[1-9A-HJ-NP-Za-km-z]{32,44}$/.test(a) ? a : null;
}

/**
 * The recipient a quote may name. A demo (testnet) vault id must never reach 1Click, so demo
 * vaults use the preview recipient, an existing mainnet account. Only a mainnet id is itself.
 */
export function quoteRecipient(vaultId: string, preview = config.quotePreviewRecipient): string {
  if (isMainnetId(vaultId)) return vaultId;
  // The stand-in must itself be a mainnet account, whatever the build setting says.
  return isMainnetId(preview) ? preview : "wrap.near";
}

export type QuoteBody = {
  dry: true;
  swapType: "EXACT_INPUT";
  slippageTolerance: number;
  originAsset: string;
  depositType: "ORIGIN_CHAIN";
  destinationAsset: string;
  amount: string;
  refundTo: string;
  refundType: "ORIGIN_CHAIN";
  recipient: string;
  recipientType: "DESTINATION_CHAIN";
  deadline: string;
};

export function previewQuoteBody(
  chain: Chain,
  dollars: number,
  refundTo: string,
  vaultId: string,
  nowMs = Date.now(),
): QuoteBody {
  return {
    dry: true,
    swapType: "EXACT_INPUT",
    slippageTolerance: 100,
    originAsset: chain.asset,
    depositType: "ORIGIN_CHAIN",
    destinationAsset: NEAR_USDC,
    amount: String(Math.round(dollars * 1_000_000)),
    refundTo,
    refundType: "ORIGIN_CHAIN",
    recipient: quoteRecipient(vaultId),
    recipientType: "DESTINATION_CHAIN",
    deadline: new Date(nowMs + 24 * 3600 * 1000).toISOString(),
  };
}

export type Quote = {
  amountInFormatted: string;
  amountOutFormatted: string;
  minAmountOut: string;
  amountInUsd?: string;
  amountOutUsd?: string;
  timeEstimate?: number;
  refundFee?: string;
};

export const QUOTE_SLOW_MESSAGE = "NEAR Intents did not answer in time. Try again in a minute.";
export const QUOTE_ADDRESS_MESSAGE =
  "Check your refund address. It must be a wallet address on the chosen chain.";
export const QUOTE_TIMEOUT_MS = 15_000;

/** Asks 1Click for a dry quote. Any failure becomes one plain message; it never hangs. */
export async function fetchPreviewQuote(body: QuoteBody, timeoutMs = QUOTE_TIMEOUT_MS): Promise<Quote> {
  const abort = new AbortController();
  const timer = setTimeout(() => abort.abort(), timeoutMs);
  try {
    const res = await fetch(`${config.oneClickUrl}/quote`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
      signal: abort.signal,
    });
    const json = await res.json().catch(() => ({}));
    if (res.ok && json.quote) return json.quote as Quote;
    // 1Click answers 400 for a refund address it will not use, sometimes with the text
    // "Internal server error" (seen for a token contract address on 11 Nov 2026).
    if (res.status >= 400 && res.status < 500) throw new Error(QUOTE_ADDRESS_MESSAGE);
    throw new Error(QUOTE_SLOW_MESSAGE);
  } catch (e) {
    if (e instanceof Error && e.message === QUOTE_ADDRESS_MESSAGE) throw e;
    throw new Error(QUOTE_SLOW_MESSAGE);
  } finally {
    clearTimeout(timer);
  }
}
