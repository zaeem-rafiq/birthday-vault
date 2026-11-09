# tUSDC contract: exact specification

Write only `contracts/tusdc/src/lib.rs`. `tests/tests/tusdc_spec.rs` holds the 8 checks it must pass. Start from `docs/sources/near-sdk-examples/fungible-token-lib.rs`, and keep its `FungibleTokenCore`, `FungibleTokenResolver`, `StorageManagement` and metadata provider blocks unchanged. The token library is in `docs/sources/near-contract-standards-5.30.1/`.

**State:** `struct Contract { token: FungibleToken, metadata: LazyOption<FungibleTokenMetadata>, hub: AccountId }`. Keep `#[derive(PanicOnDefault)]` and `#[near(contract_state)]`.

**Constant:** `const FAUCET_AMOUNT: u128 = 5_000_000;`

**Metadata:** spec `FT_METADATA_SPEC`, name `Test USDC (play money)`, symbol `tUSDC`, decimals 6, icon `None`, reference `None`, reference_hash `None`.

**Remove** `new_default_meta`, the example's `new`, and the SVG icon constant.

| Method | Who may call | What it does |
|---|---|---|
| `#[init] pub fn new(hub_id: AccountId) -> Self` | Anyone, once | Sets `hub` to `hub_id`. Mints nothing. Uses `StorageKey::FungibleToken` and `StorageKey::Metadata`. |
| `pub fn faucet(&mut self, account_id: AccountId)` | Hub only | Registers `account_id` with `self.token.internal_register_account` if `self.token.accounts.contains_key(&account_id)` is false. Then deposits `FAUCET_AMOUNT` with `self.token.internal_deposit` and emits `FtMint`. |
| `pub fn register(&mut self, account_id: AccountId)` | Hub only | Registers `account_id` if it is not registered. It does nothing if it is registered. |

**Hub only** means `require!(env::predecessor_account_id() == self.hub, "only the hub");`.

Neither method takes a deposit. The contract's own balance pays for storage.
