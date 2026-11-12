"use client";

import { useEffect, useRef } from "react";

/**
 * Polls `fn` like PLAN.md section 7 says: every `baseMs` (10 s on testnet) with ±20% jitter,
 * every 2 s for 30 s after `kick()`, 60 s after an error, and never while the tab is hidden.
 * Returns `kick`, which reads now and polls fast for the next 30 seconds.
 */
export function usePoll(fn: () => Promise<void>, baseMs: number): () => void {
  const kickRef = useRef<() => void>(() => {});

  useEffect(() => {
    let timer: ReturnType<typeof setTimeout> | null = null;
    let fastUntil = 0;
    let stopped = false;

    const schedule = (failed: boolean) => {
      if (stopped) return;
      if (timer) clearTimeout(timer);
      const base = failed ? 60_000 : Date.now() < fastUntil ? 2_000 : baseMs;
      timer = setTimeout(tick, base * (0.8 + Math.random() * 0.4));
    };

    async function tick() {
      if (document.hidden) return schedule(false);
      let failed = false;
      try {
        await fn();
      } catch {
        failed = true;
      }
      schedule(failed);
    }

    const onVisible = () => {
      if (!document.hidden) tick();
    };

    kickRef.current = () => {
      fastUntil = Date.now() + 30_000;
      tick();
    };
    tick();
    document.addEventListener("visibilitychange", onVisible);
    return () => {
      stopped = true;
      document.removeEventListener("visibilitychange", onVisible);
      if (timer) clearTimeout(timer);
    };
  }, [fn, baseMs]);

  return () => kickRef.current();
}
