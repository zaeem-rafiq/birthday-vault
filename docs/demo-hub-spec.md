# Demo hub: exact specification (step S8)

A testnet-only contract that gives each visitor three fresh play accounts, so the public page works with no wallet and no login. Write only `contracts/demo-hub/src/lib.rs`. `tests/tests/demo_hub_spec.rs` holds the 7 checks it must pass. Run `bash scripts/hub-check.sh`. PLAN.md section 5.8 is the design.

`PublicKey` and `CurveType` come from near-sdk, which re-exports them from `docs/sources/near-sdk-core-4.4.2/src/types/public_key.rs`. Use `near_sdk::PublicKey` and `near_sdk::CurveType`.

**Imports you will need:** `near_sdk::store::LookupSet` (not `near_sdk::collections`, which this crate does not enable), `near_sdk::json_types::U64`, `near_sdk::{env, near, require, AccountId, BorshStorageKey, CurveType, Gas, NearToken, PanicOnDefault, Promise, PublicKey}`.

**Constants**

```rust
const GUARDIAN_NEAR: NearToken = NearToken::from_millinear(150);
const CHILD_NEAR: NearToken = NearToken::from_millinear(120);
const BUYER_NEAR: NearToken = NearToken::from_millinear(120);
const USDC_CALL_GAS: Gas = Gas::from_tgas(10);
```

**State**

```rust
#[near] #[derive(BorshStorageKey)] enum StorageKey { Used }

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct DemoHub {
    usdc: AccountId,
    hour_ns: u64,
    day_ns: u64,
    hourly_limit: u32,
    daily_limit: u32,
    starts: Vec<u64>,
    used: LookupSet<AccountId>,
}
```

`starts` holds the block timestamps of recent starts, oldest first. `used` holds every guardian account that has started a demo.

**Implicit account id:** the lowercase hex of the key bytes after the curve byte, which is 64 characters for an ED25519 key. `pk.as_bytes()[0]` is the curve byte, and `pk.as_bytes()[1..]` are the key bytes. Write `fn implicit_id(pk: &PublicKey) -> AccountId` with `format!("{b:02x}")` over each byte, then `.parse().unwrap()`.

**Methods**

| Method | Rule |
|---|---|
| `#[init] pub fn new(usdc: AccountId, hour_ns: U64, day_ns: U64, hourly_limit: u32, daily_limit: u32) -> Self` | Panic with `only the hub` unless `env::predecessor_account_id() == env::current_account_id()`. Panic with `bad limits` unless `hour_ns > 0`, `day_ns >= hour_ns` and `0 < hourly_limit <= daily_limit`. `starts` is empty. `used` is `LookupSet::new(StorageKey::Used)`. Testnet uses one hour, one day, 20 and 100. Tests use seconds. |
| `pub fn start_demo(&mut self, guardian_pk: PublicKey, child_pk: PublicKey, buyer_pk: PublicKey) -> near_sdk::serde_json::Value` | See below. |
| `pub fn hub_info(&self) -> near_sdk::serde_json::Value` | `{"usdc", "hourly_limit", "daily_limit", "starts_last_hour", "starts_last_day"}`. The two counts are the entries of `starts` with `now - t < hour_ns` and `now - t < day_ns`, where `now = env::block_timestamp()`. |
| `pub fn has_started(&self, guardian_id: AccountId) -> bool` | `self.used.contains(&guardian_id)` |

**`start_demo`.** Check in this order, and panic with the message shown:

1. `env::predecessor_account_id() == env::current_account_id()`, else `only the hub`. The page signs with one of the hub's own function-call keys, so the caller is the hub itself.
2. Each key's `curve_type()` is `CurveType::ED25519`, else `only ed25519 keys`.
3. The three keys are all different, else `keys must differ`.
4. `guardian = implicit_id(&guardian_pk)`, and the same for `child` and `buyer`. `!self.used.contains(&guardian)`, else `this demo already started`.
5. `now = env::block_timestamp()`. Remove from the front of `starts` every entry with `now - t >= day_ns`. Count the entries with `now - t < hour_ns`. If that count is at least `hourly_limit`, panic with `hourly limit reached, try again later`. If `starts.len()` is at least `daily_limit`, panic with `daily limit reached, try again tomorrow`.

Then push `now` onto `starts`, insert it with `self.used.insert(guardian.clone())`, and send:

```rust
Promise::new(guardian.clone()).transfer(GUARDIAN_NEAR).detach();
Promise::new(child.clone()).transfer(CHILD_NEAR).detach();
Promise::new(buyer.clone()).transfer(BUYER_NEAR).detach();
Promise::new(self.usdc.clone())
    .function_call("faucet".to_string(), near_sdk::serde_json::to_vec(&near_sdk::serde_json::json!({"account_id": guardian})).unwrap(), NearToken::from_yoctonear(0), USDC_CALL_GAS)
    .function_call("register".to_string(), near_sdk::serde_json::to_vec(&near_sdk::serde_json::json!({"account_id": child})).unwrap(), NearToken::from_yoctonear(0), USDC_CALL_GAS)
    .function_call("register".to_string(), near_sdk::serde_json::to_vec(&near_sdk::serde_json::json!({"account_id": buyer})).unwrap(), NearToken::from_yoctonear(0), USDC_CALL_GAS)
    .detach();
```

Each account must hold about 0.001 NEAR per Tgas it attaches, because NEAR charges the prepaid gas up front at a worst-case price and refunds the rest. A mint attaches 100 Tgas plus a 0.025 NEAR deposit, and a withdrawal 90 to 100 Tgas, so the guardian gets 0.15 NEAR and the child and buyer 0.12 NEAR each. Testnet showed this on 10 Nov: a 0.1 NEAR guardian could not mint. A transfer to a 64-character hex id creates that account, and the account's only key is the key the id came from. Return `{"guardian": guardian, "child": child, "buyer": buyer}`.

The hub is deployed only to testnet. Its balance is the hard ceiling on what visitors can take.
