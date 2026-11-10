# Collection contract, step S4: exact specification

Write only `contracts/collection/src/lib.rs`. `tests/tests/collection_init_spec.rs` holds the 9 checks for this step. Later steps add minting, withdrawals and transfers, so follow the names here exactly. PLAN.md section 5.2 is the full design. The NFT library is in `docs/sources/near-contract-standards-5.30.1/`.

**Imports you will need:** `near_contract_standards::non_fungible_token::{NonFungibleToken, Token, TokenId}`, `near_contract_standards::non_fungible_token::metadata::{NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata, NFT_METADATA_SPEC}`, `near_sdk::collections::{LazyOption, LookupMap}`, `near_sdk::json_types::U64`, `near_sdk::{env, near, require, AccountId, BorshStorageKey, CryptoHash, NearToken, PanicOnDefault, Promise}`.

**Storage keys:** `#[near] #[derive(BorshStorageKey)] enum StorageKey { Owners, TokenMetadata, Enumeration, ContractMetadata, Vaults }`.

**Types**

```rust
#[near(serializers = [borsh, json])]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Mode { Demo, Production }

#[near(serializers = [borsh, json])]
#[derive(Clone)]
pub struct VaultState { pub guardian: AccountId, pub child: AccountId, pub unlock_ns: u64, pub created_ns: u64, pub pending: u32 }

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Collection {
    nft: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
    vault_code_hash: Option<CryptoHash>,
    usdc: AccountId,
    mode: Mode,
    min_lock_ns: u64,
    max_lock_ns: u64,
    next_serial: u64,
    vaults: LookupMap<TokenId, VaultState>,
}
```

**Helper rules**

- `fn parent_of(id: &AccountId) -> String`: everything after the first dot, or the empty string if there is none.
- **Self or parent:** `env::predecessor_account_id()` equals `env::current_account_id()`, or equals `parent_of(&env::current_account_id())`. Otherwise panic with `only self or parent`.
- **Mainnet id:** an id is mainnet when it ends with `.near` and does not end with `.test.near`.

**Methods for this step**

| Method | Rule |
|---|---|
| `#[init] pub fn new(usdc: AccountId, mode: Mode, min_lock_ns: U64, max_lock_ns: U64) -> Self` | Self or parent. Demo needs the collection id to end with `.testnet`, `.sandbox` or `.test.near`, and `0 < min_lock_ns <= max_lock_ns`. Production needs a mainnet id, `min_lock_ns >= 86_400_000_000_000` and `max_lock_ns <= 1_262_304_000_000_000_000`. Otherwise panic with `mode does not match network` or `bad lock limits`. Create `NonFungibleToken::new(StorageKey::Owners, env::current_account_id(), Some(StorageKey::TokenMetadata), Some(StorageKey::Enumeration), None::<StorageKey>)`. `vault_code_hash` is `None` and `next_serial` is 1. |
| `pub fn set_vault_code(&mut self, #[serializer(borsh)] vault_code: Vec<u8>)` | Self or parent. Panic with `vault code already set` if the hash is set. Require `env::account_balance()` at least `vault_code.len() * 0.0001 NEAR + 0.1 NEAR`, else panic with `not enough balance`. Store `env::sha256(&vault_code)` as a `CryptoHash`. Then `Promise::new(env::current_account_id()).deploy_global_contract(vault_code).detach()`. |
| `pub fn contract_info(&self) -> near_sdk::serde_json::Value` | JSON `{"mode": "Demo" or "Production", "usdc", "min_lock_ns": string, "max_lock_ns": string, "vault_code_hash": lowercase hex string or null, "next_serial": number}`. |
| `pub fn is_mainnet_id(&self, account_id: String) -> bool` | The mainnet id rule above. |
| `pub fn nft_total_supply(&self) -> U128` | From `self.nft.owner_by_id.len()`. |
| `nft_metadata` | `impl NonFungibleTokenMetadataProvider for Collection` returns the stored metadata. |

**Contract metadata:** spec `NFT_METADATA_SPEC`, symbol `VAULT`, name `Birthday Vault (DEMO, play money)` in Demo and `Birthday Vault` in Production, icon `Some(ICON.to_string())` with `const ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 8 8'%3E%3Crect width='8' height='8' fill='%23c58b3a'/%3E%3C/svg%3E";`, and `None` for `base_uri`, `reference` and `reference_hash`.

**Forbidden anywhere in the source:** `add_full_access_key`, `add_access_key`, `deploy_contract`, `delete_account`, `stake`, `delete_key`. `deploy_global_contract` and `use_global_contract` are allowed.
