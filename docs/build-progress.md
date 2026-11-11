# Birthday Vault build progress

Objective: finish PLAN.md by 14 Nov 2026 23:59 UTC. Gemma 4 runs locally through Hermes. Claude Code writes the spec and tests for each small step, Gemma writes the code until the tests pass, then a hidden check and an independent review confirm it. If Gemma fails a harder step, that step moves to Claude Code.

Tools live in ~/Documents/Projects/birthday-vault-gemma-test/tools. `arm.py` runs one Hermes session with a goal and gate, `guard.py` is the write guard, `set_goal.py` sets the goal, `watch_hermes.py` is the watch, and `hidden/` holds the independent scorer. Gemma server command is in the owner report. Hermes profile: gemmatesta.

Phase 0 is done. On 11 Nov the owner replied SKIP: the gift is simulated, nothing is deployed on mainnet, and Phase 4 is dropped (PLAN Decisions). Owner prize account for the submission form: `hugegrain4592.near` (Meteor Wallet), created 11 Nov, 1 full-access key, 0 NEAR, which is enough to receive a payout.

| Step | What | Who | State |
|---|---|---|---|
| S1 | Vault under 30 KB with no std | Gemma | Passed 9 Nov (3,341 bytes, 27 minutes); independent review: ship, 1 fix applied |
| S2 | tUSDC from the near-sdk example | Gemma | Passed 9 Nov (8 minutes); reviewed, storage drain fixed |
| S3 | Hostile token | Gemma | Passed 9 Nov (40 minutes, 79 tool calls); no independent review, because it is a test double that is never deployed |
| S4 | Collection state, init, mode rule (T9, T23) | Gemma | Passed 9 Nov (19 minutes). Mutants: 3 of 4 caught; the low-balance refusal is untested, add it in S5. Review with the whole collection after S7. |
| S5 | set_vault_code and mint flow (T3b, T8, T15, T22, T24, T26, T29) | Gemma, then Claude | Gemma passed 9 of 11 in its hour, 181 tool calls. Claude fixed the byte price and the #[near] views, and fixed one broken check. Breaks: 14 of 20 caught at first; 6 new checks added, now 20 of 20. 13 checks. |
| S6 | Withdrawals (T1, T11 second withdrawal, T12-T14, T16, T17, T20) | Gemma | Passed 9 Nov in 26.5 minutes, 16 tool calls, no help. Review added a 13th check for withdraw_near's caller rules. Breaks: 18 of 18 on the reference, 14 of 14 on Gemma's code. |
| S7 | Transfer guard, child claim, standard mint event (T4-T7, T10, T11 transfer and claim, T18, T19, T25) | Gemma | Passed 9 Nov in 36 minutes, 13 tool calls, no help. Breaks: 16 of 16 on the reference, 15 of 15 on Gemma's code (1 skipped, pattern not unique). The mint event is now the standard nft_mint. |
| S8 | Demo hub (T27, T28) | Gemma | Passed 10 Nov in about 11 minutes, no help. 7 checks; 16 of 16 breaks caught on the reference, whose logic Gemma's matches; breaks not rerun on Gemma's code. |
| S9 | Measure gas and storage, update constants, ./check, FEATURES.md, CI | Claude | Done 10 Nov except FEATURES.md and the feature checker, which wait for web end-to-end tests in Phase 3. ./check passes: 82 sandbox tests. Constants unchanged: MINT_COST has room (T24). |

## Open plan change: T3b

T3b as written cannot pass with near-sdk. A contract that uses near-sdk `Promise` links the imports for add-key, delete-key, deploy-contract, delete-account and stake, because `Promise` dispatches on all action types. tusdc.wasm shows all five. Replacement: (1) the collection source contains no call to those actions, checked by a test; (2) the deployed collection has an empty key list after locking; (3) no public method other than `set_vault_code` deploys anything. The owner was told on 9 Nov.

## Method change

From S4 on, skip the full reference contract. Prove each check with mutants of Gemma's passing code. If Gemma cannot pass, check the spec and tests first.

## Moved checks

- T10 and the transfer and claim parts of T11 moved from S6 to S7, because they need nft_transfer and claim_as_child.
- T21 moved to the web phase. The contract returns no balances, so the page does the counting and must ask only the configured USDC.

## Method change, 9 Nov evening

Before Gemma starts a step, Claude proves its checks against a hidden reference and breaks the reference on purpose. S5 lost most of Gemma's hour to a check that could never pass, because the network refused the setup transaction.

## S9 notes, 9 Nov night

- T9 `production_mint_validation` and T24 `mint_state_fits_cost` are in `tests/tests/collection_limits_spec.rs`. T9 writes `bv.near` into sandbox state, because the sandbox has no `near` registrar. At maximum lengths a mint stores 1,056 bytes, so funding, registration and 1.3 times the state need 0.0175 NEAR of the 0.025 NEAR MINT_COST.
- T26 `mint_before_distribution_refunds` has no test of its own. The sandbox cannot hold back global code distribution, so the timing cannot be forced. Its refund path is the failed-creation path, which `mint_fails_cleanly_if_account_exists` covers.
- T27 is covered by `hub_limits_hold`, `hub_auth` and `hub_keys_are_reusable_within_limits`. T28 is `hub_rejects_bad_keys`.
- `./check` covers the secret scan, format, clippy, wasm builds, unit tests and sandbox tests. Phase 3 adds `scripts/check-features`, `FEATURES.md` and the web checks, because every feature row needs a web end-to-end test first.

## Phase 2, 10 Nov early morning

- Testnet: `tusdc.birthday-vault-zr1109.testnet`, `hub.birthday-vault-zr1109.testnet` (8 pool keys, limits 20 per hour and 100 per day) and collection `g1.birthday-vault-zr1109.testnet`. Pool keys are in the gitignored `.env.render.local`.
- `scripts/deploy.sh testnet ... g1` printed DEPLOY CHECK PASS: standard nft_mint event, vault_info found, no keys on `bv-1`.
- `scripts/play-flow.sh testnet g1...` printed PLAY FLOW PASS on `bv-2`: demo start, mint, play gift, early withdrawal refused with `locked`, guardian withdrawal, child claim, child withdrawal, vault empty, no keys on `bv-2`.
- Found on testnet: a 0.1 NEAR guardian could not mint, because an account must hold about 0.001 NEAR per Tgas it attaches. Demo funding is now 0.15, 0.12 and 0.12 NEAR (PLAN Decisions). Each demo costs the hub 0.39 NEAR, so the hub's 1.5 NEAR covers about 3 demos; it needs testnet NEAR before judging.
- The collection keeps its access key until the final public deployment, as the plan allows for testnet; `scripts/lock.sh` is still to write.
- The owner allowed Claude to run `scripts/setup-hub.sh`, `scripts/deploy.sh testnet` and `scripts/play-flow.sh testnet` (`.claude/settings.local.json`, 10 Nov).
