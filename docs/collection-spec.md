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

Write only `contracts/collection/src/lib.rs`. `tests/tests/collection_mint_spec.rs` holds the 13 checks for this step, and the 9 checks from step S4 must still pass. Step S5 also changes `set_vault_code` to use the free balance rule written above. The rule uses 0.0001 NEAR per byte, which is ten times `env::storage_byte_cost()`. PLAN.md section 5.2 is the full design.

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
| `nft_token`, `nft_tokens`, `nft_supply_for_owner`, `nft_tokens_for_owner` | Delegate to `self.nft`. Write `#[near] impl NonFungibleTokenEnumeration for Collection` by hand. Without `#[near]` the methods cannot be called. Mark `impl NonFungibleTokenMetadataProvider for Collection` with `#[near]` too, and keep no plain `nft_total_supply` or `nft_metadata` method beside them. Add `pub fn nft_token(&self, token_id: TokenId) -> Option<Token>`. |

**Imports you will add:** `near_sdk::{Gas, PromiseError}`, `near_sdk::json_types::U128`, `near_contract_standards::non_fungible_token::events::NftMint`, `near_contract_standards::non_fungible_token::enumeration::NonFungibleTokenEnumeration`.

# Step S6: withdrawals

Write only `contracts/collection/src/lib.rs`. `tests/tests/collection_withdraw_spec.rs` holds the 13 checks for this step. The 22 checks from steps S4 and S5 must still pass. PLAN.md section 5.4 is the full design.

**Constants**

```rust
const SEND_GAS: Gas = Gas::from_tgas(40);
const CALLBACK_GAS: Gas = Gas::from_tgas(30);
const MIN_WITHDRAW_GAS: Gas = Gas::from_tgas(90);
```

**`#[payable] pub fn withdraw(&mut self, token_id: TokenId, ft_contract: AccountId, amount: U128, receiver_id: Option<AccountId>) -> Promise`**

Check in this order, and panic with the message shown:

1. `near_sdk::assert_one_yocto()`. It panics with `Requires attached deposit of exactly 1 yoctoNEAR`.
2. `env::prepaid_gas() >= MIN_WITHDRAW_GAS`, else `not enough gas`.
3. `let mut v = self.vaults.get(&token_id)`, else `no such vault`.
4. `env::block_timestamp() >= v.unlock_ns`, else `locked`.
5. `owner = self.nft.owner_by_id.get(&token_id).unwrap()`. `env::predecessor_account_id() == owner`, else `only the owner`.
6. `v.pending == 0`, else `busy: a withdrawal is still settling`.
7. `amount.0 > 0`, else `amount must be positive`.
8. `receiver = receiver_id.unwrap_or(owner)`. `vault = format!("{token_id}.{collection}")`. The receiver must not equal the vault, the collection or `ft_contract`, else `invalid receiver`.

Then add 1 to `v.pending`, save it with `self.vaults.insert(&token_id, &v)`, and return:

```rust
Promise::new(vault.parse().unwrap()).function_call(
    "ft_send".to_string(),
    near_sdk::serde_json::to_vec(&near_sdk::serde_json::json!({"token": ft_contract, "receiver": receiver, "amount": amount.0.to_string()})).unwrap(),
    NearToken::from_yoctonear(0), SEND_GAS)
.then(Self::ext(env::current_account_id()).with_static_gas(CALLBACK_GAS)
    .on_withdraw(token_id, Some(ft_contract), receiver, amount))
```

**`#[payable] pub fn withdraw_near(&mut self, token_id: TokenId, amount: U128, receiver_id: Option<AccountId>) -> Promise`**

The same checks 1 to 8, except that the receiver must not equal the vault or the collection. Call `near_send` on the vault with `{"receiver": receiver, "amount": amount.0.to_string()}`, `SEND_GAS` and no deposit. Then call `on_withdraw(token_id, None, receiver, amount)` the same way.

**`#[private] pub fn on_withdraw(&mut self, token_id: TokenId, ft_contract: Option<AccountId>, receiver_id: AccountId, amount: U128) -> bool`**

- `let ok = !matches!(env::promise_result_checked(0, 0), Err(PromiseError::Failed));`. A `TooLong` result counts as success.
- If a `VaultState` exists for `token_id`, set `pending = pending.saturating_sub(1)` and insert it once. This method must never panic.
- Log one event with `env::log_str`: `EVENT_JSON:` followed by `{"standard":"birthday_vault","version":"1.0.0","event":"withdraw","data":{"token_id":..,"ft_contract":..,"receiver_id":..,"amount":"<digits>","ok":ok}}`. `ft_contract` is `null` for a NEAR withdrawal.
- Return `ok`.

The vault already keeps stray tokens and sends stray NFTs back. `tests/tests/vault_spec.rs` checks that. Nothing can leave a vault before `unlock_ns`, because `withdraw` and `withdraw_near` are the only way out.

# Step S7: transfer guard, child claim and standard events

Write only `contracts/collection/src/lib.rs`. `tests/tests/collection_transfer_spec.rs` holds the 11 checks for this step. The 35 checks from steps S4 to S6 must still pass. PLAN.md sections 5.3 and 5.5 are the full design.

**Imports you will add:** `near_contract_standards::non_fungible_token::core::NonFungibleTokenCore` (already there), `near_contract_standards::non_fungible_token::events::{NftMint, NftTransfer}`, `near_sdk::PromiseOrValue`.

**Standard mint event.** Delete the `CollectionEvents` enum. In `on_vault_created`, emit the library's event instead: `NftMint { owner_id: &guardian, token_ids: &[token_id.as_str()], memo: None }.emit();`. It logs `{"standard":"nep171","version":"1.0.0","event":"nft_mint","data":[{"owner_id":..,"token_ids":[..]}]}`. Wallets and explorers read this form. `token_ids` takes `&str` values, so pass `token_id.as_str()`.

**Guard.** Add this private helper inside `impl Collection`, without `pub`:

```rust
fn guard(&self, token_id: &TokenId, receiver: &AccountId) {
    let v = self.vaults.get(token_id).unwrap_or_else(|| env::panic_str("no such vault"));
    require!(v.pending == 0, "busy: a withdrawal is still settling");
    let me = env::current_account_id();
    require!(*receiver != me && !receiver.as_str().ends_with(&format!(".{me}")), "invalid receiver");
    let owner = self.nft.owner_by_id.get(token_id).unwrap();
    if env::block_timestamp() < v.unlock_ns {
        require!(env::predecessor_account_id() == owner, "only the owner");
        require!(*receiver == v.guardian || *receiver == v.child, "locked: only guardian and child");
    }
}
```

**`#[near] impl NonFungibleTokenCore for Collection`.** Write it by hand. Delete the plain `pub fn nft_token` from `impl Collection`, because this impl now provides it.

| Method | Body |
|---|---|
| `#[payable] fn nft_transfer(&mut self, receiver_id: AccountId, token_id: TokenId, approval_id: Option<u64>, memo: Option<String>)` | `self.guard(&token_id, &receiver_id);` then `self.nft.nft_transfer(receiver_id, token_id, approval_id, memo)`. The library checks the 1 yoctoNEAR, the owner, and emits `nft_transfer`. |
| `#[payable] fn nft_transfer_call(&mut self, receiver_id: AccountId, token_id: TokenId, approval_id: Option<u64>, memo: Option<String>, msg: String) -> PromiseOrValue<bool>` | `env::panic_str("disabled: use nft_transfer")`. Keep the argument names exactly, because near-sdk reads them as JSON field names. Put `#[allow(unused_variables)]` on the method instead of renaming them. |
| `fn nft_token(&self, token_id: TokenId) -> Option<Token>` | `self.nft.nft_token(token_id)` |

Do not add `nft_approve`, `nft_revoke`, `nft_revoke_all`, `nft_is_approved` or `nft_resolve_transfer`. Do not implement `NonFungibleTokenResolver`.

**`#[payable] pub fn claim_as_child(&mut self, token_id: TokenId)`**

Check in this order, and panic with the message shown:

1. `near_sdk::assert_one_yocto()`.
2. `let v = self.vaults.get(&token_id)`, else `no such vault`.
3. `env::predecessor_account_id() == v.child`, else `only the child`.
4. `env::block_timestamp() >= v.unlock_ns`, else `locked`.
5. `owner = self.nft.owner_by_id.get(&token_id).unwrap()`. `owner == v.guardian`, else `the guardian no longer holds it`.
6. `v.pending == 0`, else `busy: a withdrawal is still settling`.

Then `self.nft.internal_transfer_unguarded(&token_id, &owner, &v.child);` and emit `NftTransfer { old_owner_id: &owner, new_owner_id: &v.child, token_ids: &[token_id.as_str()], authorized_id: None, memo: Some("child_claim") }.emit();`.

`vault_info` reads `owner_id` from `self.nft`, so it follows every transfer and claim.
