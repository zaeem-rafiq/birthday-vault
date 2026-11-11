"use client";

import { Suspense, useCallback, useEffect, useState } from "react";
import { useSearchParams } from "next/navigation";
import Link from "next/link";
import { Tree } from "@/components/Tree";
import { config } from "@/lib/config";
import { isDemoVaultId, isMainnetId } from "@/lib/network";
import { accessKeyCount, ftBalance } from "@/lib/near";
import { formatDollars, treeStage } from "@/lib/tree";
import { usePoll } from "@/lib/poll";
import {
  claimAsChild,
  loadDemo,
  refusalText,
  saveDemo,
  sendPlayGift,
  transferToBuyer,
  vaultInfo,
  withdrawAll,
  type Demo,
  type Role,
  type VaultInfo,
} from "@/lib/demo";

type Load =
  | { kind: "loading" }
  | { kind: "missing"; reason: string }
  | { kind: "unreachable" }
  | { kind: "ready"; info: VaultInfo; balance: bigint; keys: number | null };

type Message = { tone: "ok" | "refused"; text: string } | null;

function short(id: string) {
  return id.length > 20 ? `${id.slice(0, 8)}…${id.slice(-6)}` : id;
}

function useNow(ms: number) {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    const t = setInterval(() => setNow(Date.now()), ms);
    return () => clearInterval(t);
  }, [ms]);
  return now;
}

function countdown(msLeft: number) {
  const s = Math.max(0, Math.ceil(msLeft / 1000));
  const m = Math.floor(s / 60);
  return `${m}:${String(s % 60).padStart(2, "0")}`;
}

function VaultView() {
  const id = useSearchParams().get("id") ?? "";
  const [load, setLoad] = useState<Load>({ kind: "loading" });
  const [demo, setDemo] = useState<Demo | null>(null);
  const [message, setMessage] = useState<Message>(null);
  const [working, setWorking] = useState("");
  const [grew, setGrew] = useState(0);
  // The lock state the current message and gift badge belong to. When the lock ends, both are
  // stale ("still locked", "+$1.00"), so they are cleared during render (React's pattern for
  // resetting state when an input changes).
  const [shownFor, setShownFor] = useState<boolean | null>(null);
  const now = useNow(1000);

  useEffect(() => {
    const t = setTimeout(() => {
      const d = loadDemo();
      setDemo(d && d.vaultId === id ? d : null);
    }, 0);
    return () => clearTimeout(t);
  }, [id]);

  const refresh = useCallback(async () => {
    if (isMainnetId(id)) {
      setLoad({ kind: "missing", reason: "Real mainnet vaults are not shown on this demo site." });
      return;
    }
    if (!isDemoVaultId(id, config.collectionId)) {
      setLoad({ kind: "missing", reason: "This is not a Birthday Vault address." });
      return;
    }
    try {
      const info = await vaultInfo(id);
      if (!info) {
        setLoad({ kind: "missing", reason: "We could not find that vault." });
        return;
      }
      const [balance, keys] = await Promise.all([
        ftBalance(id),
        accessKeyCount(id).catch(() => null),
      ]);
      setLoad({ kind: "ready", info, balance, keys });
    } catch (e) {
      // Keep showing the last good reading; only a first read with no answer shows the error.
      setLoad((prev) => (prev.kind === "ready" ? prev : { kind: "unreachable" }));
      throw e;
    }
  }, [id]);

  const kick = usePoll(refresh, 10_000);

  if (load.kind === "loading") {
    return <p className="pt-16 text-center text-muted">Reading the vault from the network…</p>;
  }
  if (load.kind === "unreachable") {
    return (
      <div className="pt-16 text-center" data-testid="vault-unreachable">
        <h1 className="font-display text-3xl font-semibold">The test network did not answer</h1>
        <p className="mt-3 text-muted">Your vault is safe. This page could not read it just now.</p>
        <button
          onClick={() => {
            setLoad({ kind: "loading" });
            kick();
          }}
          className="mt-6 rounded-xl bg-leaf px-6 py-3 font-semibold text-white hover:bg-leaf-dark"
        >
          Try again
        </button>
      </div>
    );
  }
  if (load.kind === "missing") {
    return (
      <div className="pt-16 text-center" data-testid="vault-missing">
        <h1 className="font-display text-3xl font-semibold">Vault not found</h1>
        <p className="mt-3 text-muted">{load.reason}</p>
        <Link href="/" className="mt-6 inline-block underline underline-offset-4 text-leaf-dark">
          Try an example instead
        </Link>
      </div>
    );
  }

  const { info, balance, keys } = load;
  const unlockMs = Number(BigInt(info.unlock_ns) / BigInt(1_000_000));
  const locked = info.locked && now < unlockMs + 2000;
  if (shownFor !== locked) {
    setShownFor(locked);
    if (shownFor !== null) {
      setMessage(null);
      setGrew(0);
    }
  }
  const stage = treeStage(balance);
  const roleOf = (acct: string): string => {
    if (!demo) return short(acct);
    if (acct === demo.keys.guardian.id) return "the parent (guardian)";
    if (acct === demo.keys.child.id) return `${demo.nickname} (the child)`;
    if (acct === demo.keys.buyer.id) return "the new owner";
    return short(acct);
  };

  async function act(label: string, run: () => Promise<{ ok: boolean; error?: string }>, okText: string) {
    setWorking(label);
    setMessage(null);
    let r: { ok: boolean; error?: string };
    try {
      r = await run();
    } catch (e) {
      r = { ok: false, error: e instanceof Error ? e.message : String(e) };
    }
    setWorking("");
    setMessage(r.ok ? { tone: "ok", text: okText } : { tone: "refused", text: refusalText(r.error ?? "") });
    kick();
    return r.ok;
  }

  function remember(patch: Partial<Demo>) {
    if (!demo) return;
    const d = { ...demo, ...patch };
    saveDemo(d);
    setDemo(d);
  }

  const owner = info.owner_id;
  const mine = (r: Role) => demo?.keys[r].id === owner;
  const done = !locked && balance === BigInt(0) && (mine("buyer") || mine("child")) && demo?.path;

  return (
    <div className="pt-4 md:pt-10">
      <div
        role="note"
        className="rounded-lg bg-alert px-4 py-2 text-center text-sm font-bold tracking-wide text-white"
      >
        TESTNET, PLAY MONEY
      </div>

      <div className="mt-6 grid md:grid-cols-2 gap-8 items-center">
        <div className="flex flex-col items-center">
          <div className="w-40 sm:w-56 md:w-72" id="tree">
            <Tree stage={stage} />
          </div>
          {grew > 0 && locked && (
            <p key={grew} className="mt-1 rounded-full bg-leaf px-3 py-1 text-sm font-semibold text-white" data-testid="grew">
              +$1.00 play gift
            </p>
          )}
        </div>
        <div>
          <h1 className="font-display text-3xl sm:text-4xl font-semibold">
            {demo ? `${demo.nickname}'s vault` : "A Birthday Vault"}
          </h1>
          <dl className="mt-6 grid grid-cols-2 gap-4">
            <div className="rounded-xl bg-paper border border-line p-4">
              <dt className="text-sm text-muted">Balance</dt>
              <dd className="text-2xl font-semibold" data-testid="balance">
                {formatDollars(balance)}
              </dd>
            </div>
            <div className="rounded-xl bg-paper border border-line p-4">
              <dt className="text-sm text-muted">{locked ? "Opens in" : "Lock"}</dt>
              <dd className="text-2xl font-semibold" data-testid="lock">
                {locked ? countdown(unlockMs - now) : "Open"}
              </dd>
            </div>
            <div className="rounded-xl bg-paper border border-line p-4">
              <dt className="text-sm text-muted">Account keys</dt>
              <dd className="text-2xl font-semibold" data-testid="keys">
                {keys ?? "…"}
              </dd>
            </div>
            <div className="rounded-xl bg-paper border border-line p-4">
              <dt className="text-sm text-muted">Held by</dt>
              <dd className="text-base font-semibold break-words" data-testid="owner">
                {roleOf(owner)}
              </dd>
            </div>
          </dl>
          <p className="mt-3 text-sm text-muted">
            {keys === 0
              ? "No one holds a key to this vault's account. Only the contract can open it, and it checks the date."
              : ""}
          </p>

          <div className="mt-6 space-y-3" data-testid="actions">
            {!demo && (
              <Link
                href={`/gift/?id=${encodeURIComponent(id)}`}
                className="block w-full rounded-xl bg-leaf px-6 py-4 text-center text-lg font-semibold text-white hover:bg-leaf-dark"
              >
                Send a gift
              </Link>
            )}

            {demo && locked && (
              <>
                <button
                  onClick={async () => {
                    if (await act("gift", () => sendPlayGift(demo), "Play gift of $1.00 received. Watch the tree.")) {
                      setGrew((n) => n + 1);
                      document.getElementById("tree")?.scrollIntoView({ behavior: "smooth", block: "center" });
                    }
                  }}
                  disabled={!!working}
                  className="w-full rounded-xl bg-leaf px-6 py-4 text-lg font-semibold text-white hover:bg-leaf-dark disabled:opacity-50"
                >
                  {working === "gift" ? "Sending…" : "Send a $1 play gift"}
                </button>
                <button
                  onClick={() => act("early", () => withdrawAll(demo, "guardian"), "Withdrawn.")}
                  disabled={!!working || balance === BigInt(0)}
                  className="w-full rounded-xl border-2 border-ink px-6 py-3 font-semibold hover:bg-paper disabled:opacity-40"
                >
                  {working === "early" ? "Asking the contract…" : "Try to withdraw early"}
                </button>
                <Link href={`/gift/?id=${encodeURIComponent(id)}`} className="block text-center text-sm underline underline-offset-4 text-muted">
                  How a relative would pay from another wallet
                </Link>
              </>
            )}

            {demo && !locked && mine("guardian") && (
              <>
                <p className="font-medium">The lock has ended. What happens next?</p>
                <button
                  onClick={async () => {
                    if (await act("transfer", () => transferToBuyer(demo), "The vault now belongs to the new owner, and so does the right to withdraw."))
                      remember({ path: "transfer" });
                  }}
                  disabled={!!working}
                  className="w-full rounded-xl bg-leaf px-6 py-4 text-lg font-semibold text-white hover:bg-leaf-dark disabled:opacity-50"
                >
                  {working === "transfer" ? "Transferring…" : "Transfer it"}
                </button>
                <button
                  onClick={async () => {
                    if (await act("claim", () => claimAsChild(demo), `${demo.nickname} claimed the vault with the child account named when it was made.`))
                      remember({ path: "claim" });
                  }}
                  disabled={!!working}
                  className="w-full rounded-xl border-2 border-ink px-6 py-3 font-semibold hover:bg-paper disabled:opacity-50"
                >
                  {working === "claim" ? "Claiming…" : "The guardian lost their key"}
                </button>
              </>
            )}

            {demo && !locked && mine("buyer") && !done && (
              <>
                {!demo.oldOwnerTried && (
                  <button
                    onClick={async () => {
                      await act("old", () => withdrawAll(demo, "guardian"), "Withdrawn.");
                      remember({ oldOwnerTried: true });
                    }}
                    disabled={!!working}
                    className="w-full rounded-xl border-2 border-ink px-6 py-3 font-semibold hover:bg-paper disabled:opacity-50"
                  >
                    {working === "old" ? "Asking the contract…" : "Old owner tries to withdraw"}
                  </button>
                )}
                <button
                  onClick={() => act("buyer", () => withdrawAll(demo, "buyer"), "The new owner withdrew the money.")}
                  disabled={!!working || balance === BigInt(0)}
                  className="w-full rounded-xl bg-leaf px-6 py-4 text-lg font-semibold text-white hover:bg-leaf-dark disabled:opacity-50"
                >
                  {working === "buyer" ? "Withdrawing…" : "New owner withdraws"}
                </button>
              </>
            )}

            {demo && !locked && mine("child") && !done && (
              <button
                onClick={() => act("child", () => withdrawAll(demo, "child"), `${demo.nickname} withdrew the money.`)}
                disabled={!!working || balance === BigInt(0)}
                className="w-full rounded-xl bg-leaf px-6 py-4 text-lg font-semibold text-white hover:bg-leaf-dark disabled:opacity-50"
              >
                {working === "child" ? "Withdrawing…" : `${demo.nickname} withdraws`}
              </button>
            )}

            {done && (
              <div className="rounded-xl border border-line bg-paper p-4" data-testid="done">
                <p className="font-semibold">The vault is empty. The money went to {roleOf(owner)}.</p>
                <Link
                  href="/"
                  onClick={() => saveDemo(null)}
                  className="mt-3 inline-block underline underline-offset-4 text-leaf-dark"
                >
                  Start a new example
                </Link>
              </div>
            )}
          </div>

          <div aria-live="polite" className="mt-4 min-h-6">
            {message && (
              <p
                data-testid="message"
                role={message.tone === "refused" ? "alert" : "status"}
                className={message.tone === "refused" ? "font-semibold text-alert" : "font-semibold text-leaf-dark"}
              >
                {message.text}
              </p>
            )}
          </div>

          <p className="mt-6 text-xs text-muted break-all">
            Vault account:{" "}
            <a className="underline" href={`${config.explorerUrl}/address/${id}`} target="_blank" rel="noreferrer">
              {id}
            </a>
          </p>
        </div>
      </div>
    </div>
  );
}

export default function VaultPage() {
  return (
    <Suspense fallback={<p className="pt-16 text-center text-muted">Loading…</p>}>
      <VaultView />
    </Suspense>
  );
}
