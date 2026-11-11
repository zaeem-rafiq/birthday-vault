# Demo script (DRAFT, simulated-gift version)

Status: draft for the owner. Nothing is filmed or posted. The owner replied SKIP on 11 Nov, so no real money moves and nothing is on mainnet (PLAN.md, Decisions). This is the alternative opening from PLAN.md section 10.

Length: 2:00. About 250 spoken words. Voice is the owner's (OWNER ONLY). Captions are burned in.

## How to read this file

- `[N]` marks a number that must come from the real run on the live site. Do not film or record the voice until each one is filled. The list of open `[N]` is at the end.
- Every claim on screen comes from a live testnet page. The one outside source on screen is the NBER article.
- Maya is an example name. Never use a real child's name.

## Script

| Time | Spoken | Screen |
|---|---|---|
| 0:00 to 0:08 | This vault holds money for a child. Her parents try to take it out early. The contract says no. | The live vault page. Red banner: "TESTNET, PLAY MONEY". Click "Try to withdraw early". The refusal text appears: "[N: exact refusal text from the run]". Caption: "Live on NEAR's test network. Play money." |
| 0:08 to 0:25 | Maya is seven, so Coinbase won't open her an account. It requires account holders to be at least 18. In Illinois, about one child in nine was a 529 college-plan beneficiary over the past decade. | Split screen. Left: Coinbase user agreement, clause 1.1, the words "at least 18 years old" highlighted. Right: the NBER page, the words "approximately 11 percent of Illinois children" highlighted. Captions give both links. |
| 0:25 to 0:50 | So we built Birthday Vault. A parent types a nickname and picks a lock date. The contract mints one NFT and creates a NEAR account for the money. Here is that account's key list. It is empty. Nobody has a key, so only the contract can open it. | Type a nickname. Click "Try an example". The vault mints in [N] seconds. The tree appears. The line "Account keys: 0" fills the screen. Caption: "The key list is read from the live network." |
| 0:50 to 1:10 | Grandma doesn't need an account or a form. She opens a link and picks the chain her dollars are on. NEAR Intents quotes the price right now. For five dollars the fee is [N] cents and the wait is about [N] seconds. For this demo the gift is play money. | The gift page. Pick Base. Enter 5. The live quote appears with the label "Preview only. Nothing is sent." Then click the play gift. Label: "Simulated gift on NEAR's test network". The balance rises and the tree grows. |
| 1:10 to 1:30 | For this demo the lock is three minutes. Time is up. Now the NFT can move. The guardian sends it to a buyer. The money and the right to withdraw go with it. The old owner tries to withdraw and is refused. The new owner withdraws, and the balance moves. | The countdown reaches zero (cut to the unlocked state). Click "Transfer it". Old owner's withdrawal refused. New owner's withdrawal succeeds. The balance moves. |
| 1:30 to 1:42 | If her parents lose their keys, Maya can still claim it from the account named when the vault was made. | A second demo run. Click "The guardian lost their key". The child claims, then withdraws. |
| 1:42 to 1:52 | Nobody can touch the money before the date her parents chose. On that date her parents can hand it to her, or Maya can claim it herself. | The tree at full size. |
| 1:52 to 2:00 | It holds dollars and earns nothing, so inflation eats at it. On the lock date her parents still hold the vault, so they could take the money first. It works only if this contract and this network last until she is eighteen. The code is unaudited. Everything you saw ran on the test network with play money. No real money moved. | The Limits page, with the first two sentences highlighted. Last frame: the live link and the repository link. |

## Changes from PLAN.md section 10

- The first screen is a live testnet vault refusing an early withdrawal, so a simulated gift is never the first screen (PLAN.md section 10, alternative opening).
- The "real $5 gift" lines are removed, including the plan's 0:05 to 0:30 block that showed a real gift confirming.
- The Intents fee line now uses a live no-money quote (`dry: true`), not a receipt for a real gift. No gift is delivered through 1Click in this entry, so the script never says one was.
- The play gift is labelled "Simulated gift on NEAR's test network" on screen and in the caption.
- The last line says no real money moved.

## Fact check (11 Nov 2026)

| Line | Source | Result |
|---|---|---|
| "Coinbase requires account holders to be at least 18" | Coinbase User Agreement (United States), clause 1.1 Eligibility: "you must be at least 18 years old" | Holds. Read on the live page on 11 Nov. The script says "requires 18", not "won't open an account for a seven-year-old", so it also holds. |
| "About one child in nine ... 529 beneficiary over the past decade" | NBER Digest, November 2025: "approximately 11 percent of Illinois children were beneficiaries of a 529 plan" | Holds. 11 percent is one in nine (11.1). The article does not define "the past decade". Its data cover 2000 to 2023, so the script keeps the article's words and says "over the past decade". |

Links:

- https://www.coinbase.com/legal/user_agreement/united_states
- https://www.nber.org/index%2ephp/digest/202511/why-arent-college-savings-accounts-more-widely-used

The Coinbase page refuses automated downloads (HTTP 403). It was read in a real browser.

## Open `[N]` values (fill from the real run on the live site)

1. Exact text of the refused early withdrawal (0:00 scene).
2. Seconds from click to a minted vault (0:25 scene). Take the median of three runs.
3. Intents fee for a $5 quote, in cents (0:50 scene). Read it from the live `dry: true` quote on the gift page.
4. Intents expected time, in seconds, from the same quote.

## Before filming

- Wait for the public page (Phase 3). The labels above come from PLAN.md section 7, so check each against the live page.
- Open the live site signed out and film from that state.
- Show the key list from the network, not from a screenshot.
- Check the finished video for any key, seed, email or private address.
