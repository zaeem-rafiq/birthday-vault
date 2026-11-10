# Birthday Vault build progress

Objective: finish PLAN.md by 14 Nov 2026 23:59 UTC. Gemma 4 runs locally through Hermes. Claude Code writes the spec and tests for each small step, Gemma writes the code until the tests pass, then a hidden check and an independent review confirm it. If Gemma fails a harder step, that step moves to Claude Code.

Tools live in ~/Documents/Projects/birthday-vault-gemma-test/tools. `arm.py` runs one Hermes session with a goal and gate, `guard.py` is the write guard, `set_goal.py` sets the goal, `watch_hermes.py` is the watch, and `hidden/` holds the independent scorer. Gemma server command is in the owner report. Hermes profile: gemmatesta.

Phase 0 is done. Owner still owes: a named .near account from Meteor Wallet by the evening of 11 Nov.

| Step | What | Who | State |
|---|---|---|---|
| S1 | Vault under 30 KB with no std | Gemma | Passed 9 Nov (3,341 bytes, 27 minutes); independent review: ship, 1 fix applied |
| S2 | tUSDC from the near-sdk example | Gemma | Passed 9 Nov (8 minutes); reviewed, storage drain fixed |
| S3 | Hostile token | Gemma | Passed 9 Nov (40 minutes, 79 tool calls); no independent review, because it is a test double that is never deployed |
| S4 | Collection state, init, mode rule (T9, T23) | Gemma | Passed 9 Nov (19 minutes). Mutants: 3 of 4 caught; the low-balance refusal is untested, add it in S5. Review with the whole collection after S7. |
| S5 | set_vault_code and mint flow (T3b, T8, T15, T22, T24, T26, T29) | Gemma | Next |
| S6 | Withdrawals (T1, T10-T14, T16, T17, T20, T21) | Gemma | |
| S7 | Transfer guard and child claim (T4-T7, T18, T19, T25) | Gemma | |
| S8 | Demo hub (T27, T28) | Gemma | |
| S9 | Measure gas and storage, update constants, ./check, FEATURES.md, CI | Claude | |

## Open plan change: T3b

T3b as written cannot pass with near-sdk. A contract that uses near-sdk `Promise` links the imports for add-key, delete-key, deploy-contract, delete-account and stake, because `Promise` dispatches on all action types. tusdc.wasm shows all five. Replacement: (1) the collection source contains no call to those actions, checked by a test; (2) the deployed collection has an empty key list after locking; (3) no public method other than `set_vault_code` deploys anything. The owner was told on 9 Nov.

## Method change

From S4 on, skip the full reference contract. Prove each check with mutants of Gemma's passing code. If Gemma cannot pass, check the spec and tests first.
