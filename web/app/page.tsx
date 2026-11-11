"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { Tree } from "@/components/Tree";
import {
  loadDemo,
  mint,
  newDemo,
  saveDemo,
  startOnHub,
  validNickname,
  waitForFunding,
  type Demo,
} from "@/lib/demo";

type Phase = "idle" | "starting" | "funding" | "minting" | "error";

const PHASE_TEXT: Record<Phase, string> = {
  idle: "",
  starting: "Setting up three example accounts on NEAR's test network…",
  funding: "Giving the example parent $5 of play money…",
  minting: "Planting the vault…",
  error: "",
};

export default function Home() {
  const router = useRouter();
  const [nickname, setNickname] = useState("");
  const [phase, setPhase] = useState<Phase>("idle");
  const [error, setError] = useState("");
  const [saved, setSaved] = useState<Demo | null>(null);

  useEffect(() => {
    // Read storage after the first paint, so the static HTML and the first render match.
    const t = setTimeout(() => setSaved(loadDemo()), 0);
    return () => clearTimeout(t);
  }, []);

  const busy = phase === "starting" || phase === "funding" || phase === "minting";
  const nicknameOk = validNickname(nickname);

  async function start(e: React.FormEvent) {
    e.preventDefault();
    if (!nicknameOk || busy) return;
    setError("");
    // Resume a demo this browser already started, so a retry never spends a second hub start.
    let d = saved && !saved.vaultId && saved.nickname === nickname.trim() ? saved : newDemo(nickname);
    saveDemo(d);
    if (!d.started) {
      setPhase("starting");
      const r = await startOnHub(d);
      if (!r.ok) {
        setPhase("error");
        setError(r.error);
        return;
      }
      d = { ...d, started: true };
      saveDemo(d);
    }
    setPhase("funding");
    if (!(await waitForFunding(d))) {
      setPhase("error");
      setError("The test network is slow right now. Press Try an example again to continue.");
      return;
    }
    setPhase("minting");
    const m = await mint(d);
    if (!m.ok) {
      setPhase("error");
      setError(m.error);
      return;
    }
    d = { ...d, vaultId: m.vaultId, tokenId: m.tokenId };
    saveDemo(d);
    router.push(`/vault/?id=${encodeURIComponent(m.vaultId!)}`);
  }

  return (
    <div className="grid md:grid-cols-2 gap-4 md:gap-12 items-center pt-0 md:pt-16">
      <div className="flex justify-center order-1 md:order-2">
        <div className="w-36 sm:w-56 md:w-80">
          <Tree stage={2} />
        </div>
      </div>
      <div className="order-2 md:order-1">
        <h1 className="font-display text-4xl sm:text-5xl font-semibold leading-tight">
          A gift that waits until they turn 18.
        </h1>
        <p className="mt-4 text-lg text-muted">
          Relatives add money from the wallet they already use. Nobody can open it until the
          child is grown up, not even the parents.
        </p>

        {saved?.vaultId && phase === "idle" && (
          <p className="mt-6">
            <Link
              href={`/vault/?id=${encodeURIComponent(saved.vaultId)}`}
              className="underline underline-offset-4 text-leaf-dark font-medium"
            >
              Continue {saved.nickname}&apos;s example vault
            </Link>
          </p>
        )}

        <form onSubmit={start} className="mt-8 space-y-3" aria-describedby="nickname-help">
          <label htmlFor="nickname" className="block text-sm font-medium">
            Child&apos;s nickname
          </label>
          <input
            id="nickname"
            name="nickname"
            value={nickname}
            onChange={(e) => setNickname(e.target.value)}
            maxLength={24}
            autoComplete="off"
            placeholder="Maya"
            disabled={busy}
            className="w-full max-w-sm rounded-xl border border-line bg-paper px-4 py-3 text-lg"
          />
          <p id="nickname-help" className="text-sm text-muted">
            Use a nickname. This is public. Letters, digits and spaces, up to 24.
          </p>
          <button
            type="submit"
            disabled={!nicknameOk || busy}
            className="mt-2 w-full max-w-sm rounded-xl bg-leaf px-6 py-4 text-lg font-semibold text-white hover:bg-leaf-dark disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {busy ? "Working…" : "Try an example"}
          </button>
          <p className="text-sm text-muted">
            Free and no login. It runs on NEAR&apos;s test network with play money and a 3-minute lock.
          </p>
        </form>

        <div aria-live="polite" className="mt-4 min-h-6">
          {busy && <p data-testid="progress">{PHASE_TEXT[phase]}</p>}
          {phase === "error" && (
            <p role="alert" className="text-alert font-medium">
              {error}
            </p>
          )}
        </div>
      </div>
    </div>
  );
}
