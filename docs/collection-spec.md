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
| `pub fn set_vault_code(&mut self, #[serializer(borsh)] vault_code: Vec<u8>)` | Self or parent. Panic with `vault code already set` if the hash is set. Require the free balance, `env::account_balance()` minus `env::storage_usage() * env::storage_byte_cost()`, to be at least `vault_code.len() * 0.0001 NEAR + 0.1 NEAR`, else panic with `not enough balance`. Store `env::sha256(&vault_code)` as a `CryptoHash`. Then `Promise::new(env::current_account_id()).deploy_global_contract(vault_code).detach()`. |
| `pub fn contract_info(&self) -> near_sdk::serde_json::Value` | JSON `{"mode": "Demo" or "Production", "usdc", "min_lock_ns": string, "max_lock_ns": string, "vault_code_hash": lowercase hex string or null, "next_serial": number}`. |
| `pub fn is_mainnet_id(&self, account_id: String) -> bool` | The mainnet id rule above. |
| `pub fn nft_total_supply(&self) -> U128` | From `self.nft.owner_by_id.len()`. |
| `nft_metadata` | `impl NonFungibleTokenMetadataProvider for Collection` returns the stored metadata. |

**Contract metadata:** spec `NFT_METADATA_SPEC`, symbol `VAULT`, name `Birthday Vault (DEMO, play money)` in Demo and `Birthday Vault` in Production, icon `Some(ICON.to_string())` with `const ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 8 8'%3E%3Crect width='8' height='8' fill='%23c58b3a'/%3E%3C/svg%3E";`, and `None` for `base_uri`, `reference` and `reference_hash`.

**Forbidden anywhere in the source:** `add_full_access_key`, `add_access_key`, `deploy_contract`, `delete_account`, `stake`, `delete_key`. `deploy_global_contract` and `use_global_contract` are allowed.

# Step S5: minting

Write only `contracts/collection/src/lib.rs`. `tests/tests/collection_mint_spec.rs` holds the 11 checks for this step, and the 9 checks from step S4 must still pass. Step S5 also changes `set_vault_code` to use the free balance rule written above. PLAN.md section 5.2 is the full design.

**Constants (yoctoNEAR and gas)**

```rust
const USDC_REGISTRATION: NearToken = NearToken::from_yoctonear(1_250_000_000_000_000_000_000);
const VAULT_FUNDING: NearToken = NearToken::from_yoctonear(2_500_000_000_000_000_000_000);
const MINT_COST: NearToken = NearToken::from_yoctonear(25_000_000_000_000_000_000_000);
const REG_GAS: Gas = Gas::from_tgas(10);
const MINT_CALLBACK_GAS: Gas = Gas::from_tgas(30);
const VAULT_MEDIA: &str = "https://birthday-vault.onrender.com/vault.svg";
```

**`#[payable] pub fn mint_vault(&mut self, child_id: AccountId, unlock_ns: U64, title: Option<String>) -> Promise`**

The caller is the guardian, the payer and the first owner. Check in this order, and panic with the message shown:

1. `env::attached_deposit() >= MINT_COST`, else `deposit below mint cost`.
2. `self.vault_code_hash` is set, else `vault code not set`.
3. `guardian != child_id`, else `guardian and child must differ`.
4. Neither the guardian nor the child equals the collection or ends with `.<collection>`, else `invalid account`.
5. `now = env::block_timestamp()`. With `checked_add`, `now + min_lock_ns <= unlock_ns <= now + max_lock_ns`, else `bad unlock time`.
6. The title, if any, is at most 64 bytes, else `title too long`.

Then take `serial = self.next_serial` and add 1 to `next_serial`. It is never reused, even when the mint later fails. `token_id = format!("bv-{serial}")`. `vault = format!("{token_id}.{collection}")`, which must be at most 64 characters, else panic with `vault id too long`.

Build the promises exactly as in PLAN.md section 5.2:

```rust
let reg = Promise::new(self.usdc.clone()).function_call(
    "storage_deposit".to_string(),
    near_sdk::serde_json::to_vec(&near_sdk::serde_json::json!({"account_id": vault, "registration_only": true})).unwrap(),
    USDC_REGISTRATION, REG_GAS);
let mk = Promise::new(vault_id).create_account().use_global_contract(hash).transfer(VAULT_FUNDING);
mk.and(reg).then(Self::ext(env::current_account_id()).with_static_gas(MINT_CALLBACK_GAS)
    .on_vault_created(token_id, guardian, child_id, unlock_ns, title, payer, U128(deposit)))
```

**`#[private] pub fn on_vault_created(&mut self, token_id: TokenId, guardian: AccountId, child: AccountId, unlock_ns: U64, title: Option<String>, payer: AccountId, deposit: U128) -> bool`**

- Never read `attached_deposit` here.
- `let created = !matches!(env::promise_result_checked(0, 0), Err(PromiseError::Failed));` and the same with index 1 for `registered`.
- If both are true: record `before = env::storage_usage()`. Call `self.nft.internal_mint_with_refund(token_id.clone(), guardian.clone(), Some(metadata), None)`. Emit `NftMint { owner_id: &guardian, token_ids: &[&token_id], memo: None }` by hand. Insert `VaultState { guardian, child, unlock_ns, created_ns: env::block_timestamp(), pending: 0 }`. Then `storage_cost = (env::storage_usage() - before) * env::storage_byte_cost()`. Refund `deposit - VAULT_FUNDING - USDC_REGISTRATION - storage_cost` to the payer with `Promise::new(payer).transfer(..)` if it is above 1 yocto. Return true.
- Otherwise mint nothing. Refund `deposit`, minus `VAULT_FUNDING` if `created`, minus `USDC_REGISTRATION` if `registered`. Return false.

**Token metadata:** title `title` or `Birthday Vault`, description `A birthday gift that stays locked until the child's birthday.`, media `VAULT_MEDIA`, `copies: Some(1)`, `issued_at: Some(env::block_timestamp_ms().to_string())`, and `None` for the other fields.

**Views**

| Method | Returns |
|---|---|
| `pub fn vault_info(&self, token_id: TokenId) -> Option<serde_json::Value>` | `null` if there is no `VaultState`. Otherwise `{"vault_id", "owner_id", "guardian", "child", "unlock_ns": string, "locked": now < unlock_ns, "pending", "mode", "usdc"}`. |
| `pub fn vault_info_by_account(&self, vault_id: AccountId) -> Option<serde_json::Value>` | The same data. `vault_id` must end with `.<collection>`. The token id is the part before that suffix. `null` otherwise. |
| `nft_token`, `nft_tokens`, `nft_supply_for_owner`, `nft_tokens_for_owner` | Delegate to `self.nft`. Write `impl NonFungibleTokenEnumeration for Collection` by hand. Add `pub fn nft_token(&self, token_id: TokenId) -> Option<Token>`. |

**Imports you will add:** `near_sdk::{Gas, PromiseError}`, `near_sdk::json_types::U128`, `near_contract_standards::non_fungible_token::events::NftMint`, `near_contract_standards::non_fungible_token::enumeration::NonFungibleTokenEnumeration`.
