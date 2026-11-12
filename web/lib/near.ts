// Chain access for the demo network: view calls through a failover RPC provider, and signed
// calls from the throwaway demo keys kept in this browser. Never JSON-RPC batch arrays.
import {
  Account,
  FailoverRpcProvider,
  JsonRpcProvider,
  KeyPair,
  type KeyPairString,
} from "near-api-js";
import { config } from "./config";

let provider: FailoverRpcProvider | null = null;

export function demoProvider(): FailoverRpcProvider {
  if (!provider) {
    // near-api-js retries a failed request 12 times by default, about two minutes per address,
    // so a page would look frozen. Three quick retries (about 3.5 s), then the next address.
    provider = new FailoverRpcProvider(
      config.rpcUrls.map((url) => new JsonRpcProvider({ url }, { retries: 3, wait: 500, backoff: 2 })),
    );
  }
  return provider;
}

export async function view<T>(
  contractId: string,
  method: string,
  args: Record<string, unknown> = {},
): Promise<T> {
  return (await demoProvider().callFunction({ contractId, method, args })) as T;
}

export const TGAS = BigInt(1_000_000_000_000);
export const ONE_YOCTO = BigInt(1);
export const MINT_DEPOSIT = BigInt("25000000000000000000000"); // 0.025 NEAR

/** The lowercase hex implicit account id for an ed25519 key (64 characters). */
export function implicitId(publicKey: string): string {
  const b58 = publicKey.replace(/^ed25519:/, "");
  return bytesToHex(base58Decode(b58));
}

const B58 = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

export function base58Decode(s: string): Uint8Array {
  let n = BigInt(0);
  for (const c of s) {
    const i = B58.indexOf(c);
    if (i < 0) throw new Error("bad base58");
    n = n * BigInt(58) + BigInt(i);
  }
  const out = new Uint8Array(32);
  for (let i = 31; i >= 0; i--) {
    out[i] = Number(n & BigInt(255));
    n >>= BigInt(8);
  }
  return out;
}

function bytesToHex(b: Uint8Array): string {
  return Array.from(b, (x) => x.toString(16).padStart(2, "0")).join("");
}

export type CallResult = { ok: true } | { ok: false; error: string };

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

// The demo accounts hold little NEAR, and the chain wants the whole gas allowance up front
// (about 0.1 NEAR for a 100 Tgas withdrawal). Unused gas from the previous call comes back a
// few blocks later, so a quick second click can be refused for balance. Waiting fixes it.
const BALANCE_WAIT_MS = 20_000;
export const LOW_BALANCE_MESSAGE =
  "This example account is out of test NEAR for network fees. Start a new example.";

/**
 * Signs and sends one function call. A contract refusal comes back as `{ ok: false }`, and so
 * does every other failure: this function never throws, so a button can never stay stuck.
 */
export async function call(
  signerId: string,
  secretKey: string,
  contractId: string,
  methodName: string,
  args: Record<string, unknown>,
  deposit: bigint,
  gasTgas: number,
): Promise<CallResult> {
  const deadline = Date.now() + BALANCE_WAIT_MS;
  for (;;) {
    try {
      const account = new Account(signerId, demoProvider(), secretKey as KeyPairString);
      await account.callFunctionRaw({
        contractId,
        methodName,
        args,
        deposit,
        gas: BigInt(gasTgas) * TGAS,
        waitUntil: "EXECUTED_OPTIMISTIC",
      });
      return { ok: true };
    } catch (e) {
      const text = errorText(e);
      if (!/not enough balance/i.test(text)) return { ok: false, error: text };
      if (Date.now() > deadline) return { ok: false, error: LOW_BALANCE_MESSAGE };
      await sleep(1500);
    }
  }
}

/** Every message inside an error, including the per-provider errors a failover error wraps. */
function messages(e: unknown, depth = 0): string[] {
  if (depth > 4 || e == null) return [];
  if (typeof e !== "object") return [String(e)];
  const out: string[] = [];
  const o = e as { message?: unknown; errors?: unknown; cause?: unknown };
  if (typeof o.message === "string") out.push(o.message);
  if (Array.isArray(o.errors)) o.errors.forEach((x) => out.push(...messages(x, depth + 1)));
  if (o.cause) out.push(...messages(o.cause, depth + 1));
  try {
    out.push(JSON.stringify(e, (_k, v) => (typeof v === "bigint" ? v.toString() : v)));
  } catch {
    // Some errors hold cycles; their messages above are enough.
  }
  return out;
}

/** Pulls the contract's own panic message out of a failed transaction, when there is one. */
export function errorText(e: unknown): string {
  const all = messages(e).join(" ");
  const m = all.match(/Smart contract panicked: ([^"\\]+)/);
  if (m) return m[1].trim();
  if (/not enough balance/i.test(all)) return "Not enough balance";
  // A failover error only says "Exceeded N providers"; the real cause is the last inner error.
  const inner = (e as { errors?: unknown[] } | null)?.errors;
  if (Array.isArray(inner) && inner.length) return errorText(inner[inner.length - 1]);
  return messages(e)[0] ?? "Unknown error";
}

/** The play-USDC balance. Throws when the network cannot be read, so callers never act on a guess. */
export async function ftBalance(accountId: string): Promise<bigint> {
  const v = await view<string>(config.usdcId, "ft_balance_of", { account_id: accountId });
  return BigInt(v || "0");
}

export async function accountExists(accountId: string): Promise<boolean> {
  try {
    await demoProvider().viewAccount({ accountId });
    return true;
  } catch {
    return false;
  }
}

export async function accessKeyCount(accountId: string): Promise<number> {
  const list = await demoProvider().viewAccessKeyList({ accountId });
  return list.keys.length;
}

export function newKeyPair(): { publicKey: string; secretKey: string } {
  const kp = KeyPair.fromRandom("ed25519");
  return { publicKey: kp.getPublicKey().toString(), secretKey: kp.toString() };
}
