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

export function loadDemo(): Demo | null {
  try {
    const raw = localStorage.getItem(STORE);
    return raw ? (JSON.parse(raw) as Demo) : null;
  } catch {
    return null;
  }
}

export function saveDemo(d: Demo | null): void {
  try {
    if (d) localStorage.setItem(STORE, JSON.stringify(d));
    else localStorage.removeItem(STORE);
  } catch {
    // Private windows can refuse storage. The demo still runs; a reload just starts over.
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
    const [g, c, b] = await Promise.all([
      ftBalance(d.keys.guardian.id),
      accountExists(d.keys.child.id),
      accountExists(d.keys.buyer.id),
    ]);
    if (g >= PLAY_GRANT && c && b) return true;
    await sleep(1000);
  }
  return false;
}

/** Step 4: the guardian mints a vault for the child with a short lock. */
export async function mint(d: Demo): Promise<CallResult & { vaultId?: string; tokenId?: string }> {
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
  const tokens = await view<{ token_id: string }[]>(config.collectionId, "nft_tokens_for_owner", {
    account_id: d.keys.guardian.id,
  });
  const tokenId = tokens?.[tokens.length - 1]?.token_id;
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
  const balance = await ftBalance(d.vaultId!);
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
  if (error === LOW_BALANCE_MESSAGE) return error;
  return `Refused: ${error}`;
}
