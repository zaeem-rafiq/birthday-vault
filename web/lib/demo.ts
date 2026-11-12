// The "Try an example" flow (PLAN.md section 7). The browser makes three throwaway keys
// (guardian, child, buyer), a public hub key starts the demo, and every later step is signed by
// one of the throwaway keys. The demo is saved in localStorage so a reload resumes it.
import { config } from "./config";
import {
  MINT_DEPOSIT,
  ONE_YOCTO,
  accountExists,
  call,
  ftBalance,
  implicitId,
  newKeyPair,
  view,
  LOW_BALANCE_MESSAGE,
  type CallResult,
} from "./near";

export type Role = "guardian" | "child" | "buyer";

export type Demo = {
  nickname: string;
  keys: Record<Role, { publicKey: string; secretKey: string; id: string }>;
  started: boolean;
  vaultId?: string;
  tokenId?: string;
  /** Which ending the visitor chose after the unlock. */
  path?: "transfer" | "claim";
  /** The old owner has already been shown the refusal after the transfer. */
  oldOwnerTried?: boolean;
};

export type VaultInfo = {
  vault_id: string;
  owner_id: string;
  guardian: string;
  child: string;
  unlock_ns: string;
  locked: boolean;
  pending: number;
  mode: string;
  usdc: string;
};

const STORE = "birthday-vault-demo-v1";
export const PLAY_GRANT = BigInt(5_000_000); // the hub gives the guardian $5 of play money
export const GIFT_UNITS = BigInt(1_000_000); // one play gift is $1

export const NICKNAME_RULE = /^[A-Za-z0-9 ]{1,24}$/;

export function validNickname(s: string): boolean {
  return NICKNAME_RULE.test(s) && s.trim().length > 0;
}

// Private windows can refuse storage. The demo then lives in memory: it still runs in this tab,
// and a reload starts over.
let memoryDemo: Demo | null = null;

export function loadDemo(): Demo | null {
  try {
    const raw = localStorage.getItem(STORE);
    if (raw) return JSON.parse(raw) as Demo;
  } catch {
    // Fall through to the copy in memory.
  }
  return memoryDemo;
}

export function saveDemo(d: Demo | null): void {
  memoryDemo = d;
  try {
    if (d) localStorage.setItem(STORE, JSON.stringify(d));
    else localStorage.removeItem(STORE);
  } catch {
    // Storage refused: the copy in memory is enough for this tab.
  }
}

export function newDemo(nickname: string): Demo {
  const make = () => {
    const kp = newKeyPair();
    return { ...kp, id: implicitId(kp.publicKey) };
  };
  return {
    nickname: nickname.trim(),
    keys: { guardian: make(), child: make(), buyer: make() },
    started: false,
  };
}

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

export const BUSY_MESSAGE = "The demo is busy. Try again in a few minutes.";
export const NETWORK_MESSAGE = "The test network did not answer. Try again in a moment.";

/** Step 3: a random public hub key calls start_demo. Retries another key on a nonce clash. */
export async function startOnHub(d: Demo): Promise<CallResult> {
  if (!config.hubKeys.length) return { ok: false, error: BUSY_MESSAGE };
  const order = [...config.hubKeys].sort(() => Math.random() - 0.5);
  let last: CallResult = { ok: false, error: BUSY_MESSAGE };
  for (const key of order.slice(0, 3)) {
    last = await call(
      config.hubId,
      key,
      config.hubId,
      "start_demo",
      {
        guardian_pk: d.keys.guardian.publicKey,
        child_pk: d.keys.child.publicKey,
        buyer_pk: d.keys.buyer.publicKey,
      },
      BigInt(0),
      100,
    );
    if (last.ok) return last;
    if (/limit reached/.test(last.error)) return { ok: false, error: BUSY_MESSAGE };
    if (/already started/.test(last.error)) return { ok: true };
    if (!/nonce|Nonce|allowance|Allowance/.test(last.error)) break;
  }
  return { ok: false, error: last.ok ? "" : last.error };
}

/** Waits until the hub's detached transfers have created and funded the three accounts. */
export async function waitForFunding(d: Demo, timeoutMs = 60_000): Promise<boolean> {
  const end = Date.now() + timeoutMs;
  while (Date.now() < end) {
    try {
      const [g, c, b] = await Promise.all([
        ftBalance(d.keys.guardian.id),
        accountExists(d.keys.child.id),
        accountExists(d.keys.buyer.id),
      ]);
      if (g >= PLAY_GRANT && c && b) return true;
    } catch {
      // A failed read is not an answer; ask again.
    }
    await sleep(1000);
  }
  return false;
}

/**
 * The vault this demo's guardian minted for this demo's child, if any. A vault someone else
 * transferred to the guardian does not count, so a retry never adopts a stranger's vault.
 */
async function guardianToken(d: Demo): Promise<string | undefined> {
  const tokens = await view<{ token_id: string }[]>(config.collectionId, "nft_tokens_for_owner", {
    account_id: d.keys.guardian.id,
  });
  for (const t of [...(tokens ?? [])].reverse()) {
    const info = await view<VaultInfo | null>(config.collectionId, "vault_info", { token_id: t.token_id });
    if (info && info.guardian === d.keys.guardian.id && info.child === d.keys.child.id) return t.token_id;
  }
  return undefined;
}

/** Step 4: the guardian mints a vault for the child with a short lock. Reuses one already made. */
export async function mint(d: Demo): Promise<CallResult & { vaultId?: string; tokenId?: string }> {
  try {
    const existing = await guardianToken(d);
    if (existing) return { ok: true, tokenId: existing, vaultId: `${existing}.${config.collectionId}` };
  } catch {
    return { ok: false, error: "The test network did not answer. Press Try an example again." };
  }
  const unlockNs = (BigInt(Date.now()) + BigInt(config.lockSeconds * 1000)) * BigInt(1_000_000);
  const r = await call(
    d.keys.guardian.id,
    d.keys.guardian.secretKey,
    config.collectionId,
    "mint_vault",
    {
      child_id: d.keys.child.id,
      unlock_ns: unlockNs.toString(),
      title: `${d.nickname}'s vault`,
    },
    MINT_DEPOSIT,
    100,
  );
  if (!r.ok) return r;
  let tokenId: string | undefined;
  try {
    tokenId = await guardianToken(d);
  } catch {
    return { ok: false, error: "The test network did not answer. Press Try an example again." };
  }
  if (!tokenId) return { ok: false, error: "The vault was not created. Your deposit was refunded." };
  return { ok: true, tokenId, vaultId: `${tokenId}.${config.collectionId}` };
}

export async function vaultInfo(vaultId: string): Promise<VaultInfo | null> {
  return (await view<VaultInfo | null>(config.collectionId, "vault_info_by_account", {
    vault_id: vaultId,
  })) ?? null;
}

/** Step 5: a $1 play gift from the guardian's play money to the vault. */
export function sendPlayGift(d: Demo): Promise<CallResult> {
  return call(
    d.keys.guardian.id,
    d.keys.guardian.secretKey,
    config.usdcId,
    "ft_transfer",
    { receiver_id: d.vaultId, amount: GIFT_UNITS.toString() },
    ONE_YOCTO,
    30,
  );
}

/** Withdraws the whole vault balance as `who`. Refused while locked, or if `who` is not the owner. */
export async function withdrawAll(d: Demo, who: Role): Promise<CallResult> {
  let balance: bigint;
  try {
    balance = await ftBalance(d.vaultId!);
  } catch {
    return { ok: false, error: NETWORK_MESSAGE };
  }
  // Ask for at least 1 unit, so an empty vault still shows the contract's own answer.
  return call(
    d.keys[who].id,
    d.keys[who].secretKey,
    config.collectionId,
    "withdraw",
    {
      token_id: d.tokenId,
      ft_contract: config.usdcId,
      amount: (balance > BigInt(0) ? balance : BigInt(1)).toString(),
    },
    ONE_YOCTO,
    100,
  );
}

/** Step 8: the guardian sends the NFT, and with it the right to withdraw, to the buyer. */
export function transferToBuyer(d: Demo): Promise<CallResult> {
  return call(
    d.keys.guardian.id,
    d.keys.guardian.secretKey,
    config.collectionId,
    "nft_transfer",
    { receiver_id: d.keys.buyer.id, token_id: d.tokenId },
    ONE_YOCTO,
    30,
  );
}

/** Step 9: the child claims the NFT while the guardian still holds it. */
export function claimAsChild(d: Demo): Promise<CallResult> {
  return call(
    d.keys.child.id,
    d.keys.child.secretKey,
    config.collectionId,
    "claim_as_child",
    { token_id: d.tokenId },
    ONE_YOCTO,
    30,
  );
}

/** Explains a contract refusal in plain words. */
export function refusalText(error: string): string {
  if (/locked/.test(error)) return "Refused by the contract: the vault is still locked.";
  if (/only the owner/.test(error)) return "Refused by the contract: only the current owner can withdraw.";
  if (/busy/.test(error)) return "Refused: a withdrawal is still settling. Try again in a moment.";
  if (/no longer holds/.test(error)) return "Refused: the guardian no longer holds this vault.";
  if (error === LOW_BALANCE_MESSAGE || error === NETWORK_MESSAGE) return error;
  return `Refused: ${error}`;
}
