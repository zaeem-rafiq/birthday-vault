"use client";

import { Suspense, useEffect, useState } from "react";
import { useSearchParams } from "next/navigation";
import Link from "next/link";
import { config } from "@/lib/config";
import { isDemoVaultId, isMainnetId } from "@/lib/network";
import { vaultInfo } from "@/lib/demo";
import {
  CHAINS,
  fetchPreviewQuote,
  previewQuoteBody,
  validRefund,
  type Chain,
  type Quote,
} from "@/lib/quote";

type Found = "checking" | "found" | "missing";

function GiftView() {
  const id = useSearchParams().get("id") ?? "";
  const [found, setFound] = useState<Found>("checking");
  const [chain, setChain] = useState<Chain>(CHAINS[0]);
  const [amount, setAmount] = useState("5");
  const [refund, setRefund] = useState("");
  const [quote, setQuote] = useState<Quote | null>(null);
  const [quoteError, setQuoteError] = useState("");
  const [loading, setLoading] = useState(false);
  const [attempt, setAttempt] = useState(0);

  useEffect(() => {
    let live = true;
    (async () => {
      if (isMainnetId(id) || !isDemoVaultId(id, config.collectionId)) return setFound("missing");
      const info = await vaultInfo(id).catch(() => null);
      if (live) setFound(info ? "found" : "missing");
    })();
    return () => {
      live = false;
    };
  }, [id]);

  const dollars = Number(amount);
  const amountOk = Number.isFinite(dollars) && dollars >= 1 && dollars <= 10_000;
  const refundTo = validRefund(chain, refund);

  // A fresh no-money quote whenever the chain, amount or refund address changes.
  useEffect(() => {
    if (found !== "found" || !amountOk || !refundTo) return;
    let live = true;
    const t = setTimeout(async () => {
      setLoading(true);
      setQuoteError("");
      try {
        const q = await fetchPreviewQuote(previewQuoteBody(chain, dollars, refundTo, id));
        if (live) setQuote(q);
      } catch (e) {
        if (live) {
          setQuote(null);
          setQuoteError(e instanceof Error ? e.message : "NEAR Intents did not answer in time. Try again in a minute.");
        }
      } finally {
        if (live) setLoading(false);
      }
    }, 400);
    return () => {
      live = false;
      clearTimeout(t);
    };
  }, [found, chain, dollars, amountOk, refundTo, id, attempt]);

  if (found === "checking") return <p className="pt-16 text-center text-muted">Finding the vault…</p>;
  if (found === "missing") {
    return (
      <div className="pt-16 text-center" data-testid="vault-missing">
        <h1 className="font-display text-3xl font-semibold">Vault not found</h1>
        <p className="mt-3 text-muted">We could not find that vault, so there is nothing to send a gift to.</p>
        <Link href="/" className="mt-6 inline-block underline underline-offset-4 text-leaf-dark">
          Try an example instead
        </Link>
      </div>
    );
  }

  const fee = quote ? Number(quote.amountInFormatted) - Number(quote.amountOutFormatted) : 0;

  return (
    <div className="pt-4 md:pt-10 max-w-xl">
      <div role="note" className="rounded-lg bg-alert px-4 py-2 text-center text-sm font-bold tracking-wide text-white">
        TESTNET, PLAY MONEY
      </div>
      <h1 className="mt-6 font-display text-3xl sm:text-4xl font-semibold">Send a gift from another wallet</h1>
      <p className="mt-3 text-muted">
        A relative pays USDC from the chain their wallet already uses. NEAR Intents converts it
        and delivers dollars to the vault on NEAR.
      </p>
      <p className="mt-4 rounded-lg border-2 border-ink px-4 py-3 font-semibold" data-testid="preview-label">
        Preview only. Nothing is sent.
      </p>

      <form className="mt-6 space-y-5" onSubmit={(e) => e.preventDefault()}>
        <fieldset>
          <legend className="text-sm font-medium">Pay from</legend>
          <div className="mt-2 grid grid-cols-2 sm:grid-cols-4 gap-2">
            {CHAINS.map((c) => (
              <label
                key={c.key}
                className={`cursor-pointer rounded-xl border px-3 py-3 text-center font-medium ${
                  chain.key === c.key ? "border-leaf bg-paper ring-2 ring-leaf" : "border-line"
                }`}
              >
                <input
                  type="radio"
                  name="chain"
                  value={c.key}
                  checked={chain.key === c.key}
                  onChange={() => setChain(c)}
                  className="sr-only"
                />
                USDC on {c.name}
              </label>
            ))}
          </div>
        </fieldset>

        <div>
          <label htmlFor="amount" className="block text-sm font-medium">
            Amount in dollars (at least $1)
          </label>
          <input
            id="amount"
            inputMode="decimal"
            value={amount}
            onChange={(e) => setAmount(e.target.value)}
            className="mt-2 w-40 rounded-xl border border-line bg-paper px-4 py-3 text-lg"
          />
          {!amountOk && <p className="mt-1 text-sm text-alert">Enter an amount from $1 to $10,000.</p>}
        </div>

        <div>
          <label htmlFor="refund" className="block text-sm font-medium">
            Your {chain.name} address, for refunds
          </label>
          <input
            id="refund"
            value={refund}
            onChange={(e) => setRefund(e.target.value)}
            autoComplete="off"
            spellCheck={false}
            placeholder={chain.kind === "evm" ? "0x…" : "Your Solana address"}
            className="mt-2 w-full rounded-xl border border-line bg-paper px-4 py-3 font-mono text-sm"
          />
          {refund && !refundTo && (
            <p className="mt-1 text-sm text-alert">That is not a valid {chain.name} address.</p>
          )}
        </div>
      </form>

      <section className="mt-6 rounded-xl border border-line bg-paper p-5" aria-live="polite" data-testid="quote">
        <h2 className="font-semibold">Live price</h2>
        {!refundTo && <p className="mt-2 text-muted">Enter a refund address to see a live price.</p>}
        {refundTo && loading && <p className="mt-2 text-muted">Asking NEAR Intents…</p>}
        {refundTo && !loading && quoteError && (
          <div className="mt-2" data-testid="quote-error">
            <p className="text-alert">{quoteError}</p>
            <button
              type="button"
              onClick={() => setAttempt((n) => n + 1)}
              className="mt-2 rounded-lg border-2 border-ink px-4 py-2 font-semibold hover:bg-cream"
            >
              Try again
            </button>
          </div>
        )}
        {refundTo && amountOk && !loading && quote && (
          <dl className="mt-3 grid grid-cols-2 gap-y-2 text-sm">
            <dt className="text-muted">You pay</dt>
            <dd className="text-right font-semibold">{quote.amountInFormatted} USDC on {chain.name}</dd>
            <dt className="text-muted">The vault would receive</dt>
            <dd className="text-right font-semibold" data-testid="amount-out">
              {quote.amountOutFormatted} USDC
            </dd>
            <dt className="text-muted">At least</dt>
            <dd className="text-right">{(Number(quote.minAmountOut) / 1_000_000).toFixed(2)} USDC</dd>
            <dt className="text-muted">Fees</dt>
            <dd className="text-right">{fee < 0.01 ? "under $0.01" : `$${fee.toFixed(2)}`}</dd>
            {quote.timeEstimate != null && (
              <>
                <dt className="text-muted">Time</dt>
                <dd className="text-right">about {quote.timeEstimate} seconds</dd>
              </>
            )}
          </dl>
        )}
        <p className="mt-4 text-xs text-muted">
          On this demo the quote names a stand-in NEAR account, because NEAR Intents has no test
          network. No deposit address is created and no money moves.
        </p>
      </section>

      <Link href={`/vault/?id=${encodeURIComponent(id)}`} className="mt-6 inline-block underline underline-offset-4 text-leaf-dark">
        Back to the vault
      </Link>
    </div>
  );
}

export default function GiftPage() {
  return (
    <Suspense fallback={<p className="pt-16 text-center text-muted">Loading…</p>}>
      <GiftView />
    </Suspense>
  );
}
