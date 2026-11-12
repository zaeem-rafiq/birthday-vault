// Build-time settings. Next.js inlines each NEXT_PUBLIC_* value only when it is read by its full
// literal name, so every value is read here, once, by name.

function list(value: string | undefined, fallback: string): string[] {
  return (value || fallback)
    .split(",")
    .map((s) => s.trim())
    .filter(Boolean);
}

export const config = {
  /** "testnet" on the public site, "sandbox" in the end-to-end tests. */
  network: process.env.NEXT_PUBLIC_NETWORK || "testnet",
  rpcUrls: list(
    process.env.NEXT_PUBLIC_DEMO_RPC_URLS,
    "https://test.rpc.fastnear.com,https://testnet-rpc.intea.rs",
  ),
  collectionId: process.env.NEXT_PUBLIC_COLLECTION_ID || "",
  hubId: process.env.NEXT_PUBLIC_HUB_ID || "",
  usdcId: process.env.NEXT_PUBLIC_USDC_ID || "",
  /** The public pool of hub keys that may call only start_demo. Never committed. */
  hubKeys: list(process.env.NEXT_PUBLIC_HUB_KEYS, ""),
  /** Lock length for a new demo vault. The public site uses 3 minutes. */
  lockSeconds: Number(process.env.NEXT_PUBLIC_LOCK_SECONDS || "180"),
  /** An existing mainnet account used only as the recipient of no-money preview quotes. */
  quotePreviewRecipient: process.env.NEXT_PUBLIC_QUOTE_PREVIEW_RECIPIENT || "wrap.near",
  oneClickUrl: process.env.NEXT_PUBLIC_ONECLICK_URL || "https://1click.chaindefuser.com/v0",
  explorerUrl:
    process.env.NEXT_PUBLIC_EXPLORER_URL || "https://testnet.nearblocks.io",
};

export const PLAY_DECIMALS = 6;
