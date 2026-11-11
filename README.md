# Birthday Vault

**Live demo: [LIVE LINK]** (testnet, play money, no login. Click "Try an example".)

A crypto gift for a child that relatives can add to from the wallet they already use, and nobody can open until the child turns 18.

Built at NEAR HACKATHON. Category: NFTs That Do Something.

> DRAFT. I fill the `[BRACKETS]` placeholders after the public page is live. I have posted and submitted nothing.

## The problem and the result

- **Problem:** A relative who keeps dollars (USDC) in a crypto wallet has no simple way to set money aside for a child so that nobody can spend it before a chosen date. Two facts show the gap: about 11 percent of Illinois children, roughly one in nine, were beneficiaries of a 529 college-savings plan over the past decade. Source: [NBER Digest, November 2025](https://www.nber.org/index%2ephp/digest/202511/why-arent-college-savings-accounts-more-widely-used). Coinbase requires account holders to be at least 18. Source: [user agreement, clause 1.1](https://www.coinbase.com/legal/user_agreement/united_states). A child has no easy place to hold a gift.
- **Result:** [N] seconds from the "Try an example" click to a minted vault whose NEAR account has 0 access keys. Also, [N] sandbox tests pass. They include the attack tests listed in PLAN.md section 6.

## Screenshots

1. [SCREENSHOT 1: the home page with the tree and the "Try an example" button.] Caption: [N]
2. [SCREENSHOT 2: a vault page. The red "TESTNET, PLAY MONEY" banner, the balance, the lock countdown and "Account keys: 0".]
3. [SCREENSHOT 3: the early withdrawal refused, with the contract's own message.]
4. [SCREENSHOT 4: the gift page with a live no-money NEAR Intents quote labelled "Preview only. Nothing is sent."]
5. [SCREENSHOT 5: the simulated gift labelled "Simulated gift on NEAR's test network", and the tree grown.]
6. [SCREENSHOT 6: after the lock ends, the NFT transferred and the old owner refused.]
7. [SCREENSHOT 7: the Limits page.]

## Try it: do X, see Y

Steps assume the live page. Check each against the page before submitting.

1. Open [LIVE LINK] in any browser. See one tree and one button.
2. Type a nickname of up to 24 letters, digits or spaces. Use a nickname, not a real name, because everything on the network is public. See the line "Use a nickname. This is public."
3. Click **Try an example**. See a vault minted for a 3-minute lock and a red "TESTNET, PLAY MONEY" banner.
4. Look at "Account keys". See `0`. The vault's NEAR account has no key, so only the contract can move its money.
5. Click **Send a play gift**. See the balance rise and the tree grow. The page labels it "Simulated gift on NEAR's test network".
6. Click **Try to withdraw early**. See the contract refuse.
7. Open the gift page and pick a chain. See a live quote from NEAR Intents, marked "Preview only. Nothing is sent."
8. Wait for the countdown to end.
9. Click **Transfer it**. See the NFT move to a buyer. Click the old owner's withdraw. See it refused. Click the buyer's withdraw. See the balance move.
10. Or click **The guardian lost their key**. See the child account claim the NFT, then withdraw.

## How NEAR is used

- **NEP-171 NFT.** Each vault is one NFT in a collection contract. The collection holds the only owner record.
- **One NEAR account per vault, with no keys.** The collection creates the account `bv-<n>.<collection>` and deploys the vault code to it from a global contract, by hash. The code adds no key, so the account can change only through its own code.
- **The vault obeys only its parent.** The collection checks the lock date, the owner and a pending-withdrawal counter before it tells the vault to send USDC.
- **NEAR Intents 1Click.** The gift page asks 1Click for a quote that turns USDC on Base, Arbitrum, Ethereum or Solana into USDC on NEAR.
- **Global contracts.** Many vaults share one published code hash, which makes each vault cheap and its code immutable.

## Strengths

- The contract, not a company, enforces the date. There is no admin key and no upgrade path on the collection or on a vault.
- A sale during a withdrawal cannot drain the money. A `pending` counter blocks transfers and second withdrawals until the first one settles.
- If the guardian loses their key, the child account can claim the NFT after the lock date.
- The test suite attacks the contracts: early withdrawal, sale races, a hostile token, a failed mint and others. The tests are in `tests/tests/` and follow PLAN.md section 6.
- The public page needs no login and no real money.

## Limits

The balance earns nothing, and inflation reduces it. The vault works only if this contract and this network last until the lock date. USDC's issuer can freeze an account. Nobody has audited the code. The public demo runs on NEAR's test network with play money and a 3-minute lock. The demo simulates the gift. This entry delivered no real gift through NEAR Intents, and I deployed nothing on mainnet. Nobody can correct a wrong child account. On the lock date, the guardian (the parent who made the vault) still holds the NFT and can withdraw everything first. The child can claim the NFT only while the guardian still holds it. See `/limits` on the live page for the full text.

## New code against baseline

- **Baseline:** the `init` commit `5a06c27` (9 Nov 2026) is an empty repository. Tag `pre-hackathon`: [I CREATE THIS TAG].
- **New code:** everything in `contracts/`, `tests/`, `scripts/`, `web/` and `docs/`, written from 9 Nov 2026.
- **Diff since baseline:** [LINK TO COMPARE VIEW: `pre-hackathon...hackathon-submission`]
- **Derived code:** `contracts/tusdc` starts from the `fungible-token/ft` example in [near/near-sdk-rs](https://github.com/near/near-sdk-rs), tag `near-sdk-v5.30.1`. The pending-counter idea comes from [Deed](https://github.com/ClaudeOpusPremiumAgent/deed) (MIT). I wrote the code here new and copied no Deed file.
- **Other projects by the same person in this hackathon:** [NONE, or NAME IT HERE].

## Libraries, tools and AI-generated code

- **Contracts (Rust 1.95.0):** `near-sdk` 5.30.1, `near-contract-standards` 5.30.1. The vault uses `near-sys` with no std.
- **Tests:** `near-workspaces` 0.24.0, `tokio`, `anyhow`.
- **Web:** Next.js 16.4.5, React 19.2.9, `near-api-js` 7.4.0, TypeScript 5.9.3. [ADD Tailwind, `qrcode`, Vitest, `fast-check` and Playwright as the web phase adds them.]
- **Tools:** `cargo-near` 0.23.0, `near-cli-rs` 0.31.1, gitleaks.
- **AI-generated code:** AI tools wrote all of the code in this repository, under the owner's direction. Gemma 4 wrote the first version of the vault, the tUSDC contract, the hostile token, the collection steps S4 to S8 and the demo hub. It ran locally through Hermes Agent. Claude Code wrote the specifications and tests, reviewed and fixed that code, and wrote the scripts and the rest. `docs/build-progress.md` records each step. [ADD the web phase author.]
- **Boilerplate and templates:** [CONFIRM when the web phase merges. List any template used for `web/`.]

## Other chains and partner protocols

- **Other chains:** Base, Arbitrum, Ethereum and Solana, only as the source of a gift in a NEAR Intents quote. The demo sends nothing on them. This project does not run an existing product on another chain.
- **Partner protocol:** [NEAR Intents](https://docs.near-intents.org) 1Click API (`https://1click.chaindefuser.com/v0`), for quotes. It is a public endpoint and needs no key.

## Rebuild steps

You need a Mac or Linux machine with Rust 1.95.0 (`rust-toolchain.toml` pins it), `cargo-near` 0.23.0, gitleaks and Node 22.22.2 or later.

1. Clone the repository and open it.
2. Run `./check`. It runs the secret scan, format, lint, the wasm builds and the sandbox tests. When it works, it prints `CHECK PASS`.
3. To build the site, run `cd web && npm ci && npm run build`. See the static site in `web/out`. [CONFIRM the commands and the settings the live site needs.]
4. To rehearse on testnet, read `scripts/deploy.sh`, `scripts/setup-hub.sh` and `scripts/play-flow.sh`. They need a funded testnet account and the `near` CLI.

A wasm built locally matches the deployed code hash only with the same Rust and cargo-near versions as the deployment. The hash printed by `scripts/deploy.sh` is the one to compare.

## License

[LICENSE: I add the file with the public page (Phase 3).]
