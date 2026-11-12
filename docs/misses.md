# Misses

A miss is a bug found after Claude reported the work as done: by the owner in a preview, by a review, by GitHub's check after a local pass, or live. The pull request that fixes a miss adds a regression test and a row here. `/go-live` reports the rows since the last release. A miss that repeats in the same layer becomes a check, a hookify rule, or a rules change.

Layers: unit, property, e2e, review, check, preview walk-through, go-live, live monitoring.

| Date | What broke (owner's words) | Found by | Should have been caught by | What changed |
|---|---|---|---|---|
| 2026-11-11 | Every vault NFT's picture pointed at a vault.svg the site never published, so wallets and explorers would show a broken image | ce-code-review (correctness) of the contracts, after Phase 3 was built | e2e: no test fetched the token's media URL | `web/public/vault.svg` added (09830ad); `web/e2e/media.spec.ts#vault NFT picture is served` checks it; owner-steps step 12 checks the Render address |
| 2026-11-11 | The play-money account was funded for about 78 demos, so the demo would stop partway through judging | ce-code-review (reliability), after Phase 2 was reported done | review: setup-hub.sh sized tUSDC with a flat 0.3 NEAR | `setup-hub.sh` sizes tUSDC from the daily cap (efbe6ff); the live account was topped up to 11.77 testnet NEAR; the PLAN section 2 morning check reads tUSDC. No automated test: the balance is live testnet state |
| 2026-11-11 | `deploy.sh` would sign mainnet transactions behind one setting, `OWNER_RUN=1`, after the typed-word script was dropped | ce-code-review (security, testing, standards), after Phase 2 was reported done | review: the SKIP decision removed mainnet.sh without updating deploy.sh | `deploy.sh` refuses mainnet unless `--dry-run` and rejects unknown options (efbe6ff); `./check` runs `bash -n` on every script. The refusal has no automated test, because a broken guard would reach a real signer |
