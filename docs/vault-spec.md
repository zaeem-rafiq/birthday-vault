# Vault contract: exact specification

Write only `contracts/vault/src/lib.rs`. `tests/tests/vault_spec.rs` holds the 7 checks it must pass.

**State:** a struct named `Vault` with no fields, marked `#[near(contract_state)]` and `#[derive(Default)]`. Do not write an `#[init]` method.

**Parent rule:** the parent is everything after the first dot of `env::current_account_id()`. For `vault.parent.test.near` the parent is `parent.test.near`. A method marked "Parent only" must panic unless `env::predecessor_account_id()` equals the parent.

**Methods.** Use exactly these names, argument names and types. `U128` is `near_sdk::json_types::U128`.

| Signature | Who may call | What it does |
|---|---|---|
| `pub fn ft_send(&mut self, token: AccountId, receiver: AccountId, amount: U128) -> Promise` | Parent only | Calls `ft_transfer` on `token` with the JSON arguments `{"receiver_id": receiver, "amount": amount}`, an attached deposit of exactly 1 yoctoNEAR, and 30 Tgas. |
| `pub fn near_send(&mut self, receiver: AccountId, amount: U128) -> Promise` | Parent only | Transfers `amount` yoctoNEAR to `receiver`. |
| `pub fn ft_on_transfer(&mut self, sender_id: AccountId, amount: U128, msg: String) -> U128` | Anyone | Returns `U128(0)`. |
| `pub fn nft_on_transfer(&mut self, sender_id: AccountId, previous_owner_id: AccountId, token_id: String, msg: String) -> bool` | Anyone | Returns `true`. |

**Argument names** become JSON keys, so do not add a leading underscore to any of them. For unused arguments, put `#[allow(unused_variables)]` on the method.

**Macros:** use `#[near(contract_state)]` on the struct and `#[near]` on the `impl` block. Do not use `near_bindgen`.

**JSON:** `near_sdk::serde_json::json!` builds the arguments. Do not add dependencies.
