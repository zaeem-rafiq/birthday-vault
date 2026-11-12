# Submission draft (DRAFT, not posted, not submitted)

The owner approves all text before use. The owner posts on X and sends the form (PLAN.md Phase 6). Each submission is final.

Per hackathons.md: the link and the idea in one sentence come first, then the two numbers, then what NEAR makes possible, then one complete moment, then the limits in one short paragraph after the strengths.

Open placeholders: `[LIVE LINK]`, `[REPO LINK]`, `[N]`. Fill them from the live run. The two numbers:

- **Problem number:** about 11 percent, or one in nine, of Illinois children were 529-plan beneficiaries over the past decade. Source: NBER Digest, November 2025, checked 11 Nov.
- **Result number:** `[N]` seconds from click to a minted vault with 0 account keys. Measure it on the live site.

## Form fields (from the event page, checked 11 Nov)

| Field | Value |
|---|---|
| projectName | Birthday Vault |
| tracks | NFTs That Do Something |
| nearAddress | `hugegrain4592.near` (the owner's own named account. Read it back letter by letter before sending.) |
| projectUrl | `[REPO LINK]` (public repository, max 300 characters) |
| demoUrl | `[LINK TO THE X POST]` (max 300 characters) |
| idea | The description below (20 to 2,000 characters) |

## Description for the form

Count: 1,922 characters with placeholders. Limit: 2,000. A real link is longer than `[LIVE LINK]`, so recount after filling it.

```text
Live demo, no login: [LIVE LINK]. Birthday Vault is a crypto gift for a child. Relatives add to it from the wallet they already use. Nobody can open it until the child turns 18.

The problem: A relative who keeps dollars (USDC) in a crypto wallet has no simple way to set money aside for a child so that nobody can spend it before a chosen date. About 11 percent of Illinois children, one in nine, were 529 college-plan beneficiaries over the past decade. Source: NBER, Nov 2025. Coinbase requires account holders to be at least 18.

The result: [N] seconds after a click on the demo page, the page mints a vault whose NEAR account has 0 access keys. Nobody holds a key, so only the contract can open it, and the contract checks the date.

How NEAR makes it possible: each vault is one NEP-171 NFT tied to its own NEAR sub-account. The vault code is a global contract published once by hash, so every vault is cheap and its code cannot change. The collection has no admin key and no upgrade path. A relative's gift arrives as USDC through the NEAR Intents 1Click service, from the chain their dollars are already on. The page shows a live quote.

One complete moment: a visitor mints a vault with a 3-minute lock and sends a play gift. The visitor tries to withdraw early, and the contract refuses. When the countdown ends, the visitor transfers the NFT. The contract refuses the old owner, and the new owner withdraws. If a guardian loses their key, the child account can claim the NFT.

Limits: the demo runs on NEAR's test network with play money, a 3-minute lock and a simulated gift. This entry delivered no real gift through NEAR Intents, and nothing is on mainnet. On the lock date the guardian (the parent who made the vault) still holds the NFT and can withdraw first. The balance earns nothing. It works only while this contract and network last. USDC's issuer can freeze an account. Nobody has audited the code.
```

## X post

Rules from the event page: say it was built at NEAR HACKATHON, tag @NEARProtocol, use #NEARHackathon, attach a demo video, lead with the user problem, show one complete action, explain NEAR's role, end with links and a specific question. The first image is a live screen. The owner writes and publishes the post.

### Long version (needs X Premium, or post it as a thread)

```text
1 in 9 Illinois kids was a 529 plan beneficiary over the past decade. Coinbase requires account holders to be 18. So where does a child's birthday money go?

Birthday Vault: a crypto gift relatives can add to from the wallet they already use. Nobody can open it until the child turns 18.

Built at NEAR HACKATHON by @NEARProtocol #NEARHackathon

In the video: a vault refuses an early withdrawal. Click to a vault with 0 account keys: [N] seconds. Each vault is an NFT with its own NEAR account that no one holds a key to. The contract checks the date.

Gifts arrive as USDC through NEAR Intents, from any chain. Limits: test network, play money, simulated gift, unaudited, earns nothing, parents can withdraw first on the lock date.

Try it, no login: [LIVE LINK]
Code: [REPO LINK]

Would you trust a contract, not a person, to hold a gift for 18 years?
```

### Short version (under 280 characters, for a standard account)

Count of the text below with each link counted as 23 characters: 278. Limit: 280.

```text
Built at NEAR HACKATHON @NEARProtocol #NEARHackathon: Birthday Vault. A child's gift nobody can touch before the date the parents choose. Test network, play money. Try it: [LIVE LINK] Code: [REPO LINK] Would you trust a contract with a gift for 18 years?
```

Put the limits in a reply under the short version: "Limits: test network, play money, simulated gift, unaudited, earns nothing, parents can withdraw first on the lock date."

## Open decisions for the owner

1. Does the owner's X account allow long posts? If not, use the short version plus a reply.
2. I removed "not even the parents" from the short post, because the guardian can withdraw first on the lock date. The posts now say the guardian limit.
3. Approve the question that ends the post, or give another.
