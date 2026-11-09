# Hostile token: exact specification

A test double that is never deployed for real. Write only `contracts/hostile-token/src/lib.rs`. `tests/tests/hostile_spec.rs` holds the 7 checks it must pass. Use near-sdk macros: `#[near(contract_state)]`, `#[derive(Default)]`, `#[near]`.

**State:** `struct HostileToken { mode: String, fail_registration: bool, transfers: u32 }`. The default `mode` is the empty string, which means normal.

| Method | Behaviour |
|---|---|
| `pub fn set_behavior(&mut self, mode: String, fail_registration: bool)` | Stores both values. Anyone may call it. |
| `#[payable] pub fn storage_deposit(&mut self, account_id: Option<AccountId>, registration_only: Option<bool>) -> near_sdk::serde_json::Value` | Panics with `registration refused` if `fail_registration` is true. Otherwise returns `{"total":"1250000000000000000000","available":"0"}`. |
| `#[payable] pub fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>)` | See the modes below. |
| `pub fn ft_balance_of(&self, account_id: AccountId) -> U128` | Always returns `U128(1_000_000)`. |
| `pub fn transfers(&self) -> u32` | Returns `transfers`. |

**`ft_transfer` modes**

| `mode` | Behaviour |
|---|---|
| empty string | Adds 1 to `transfers`. |
| `fail` | Panics with `transfer refused`. |
| `huge` | Adds 1 to `transfers`, then returns 100,000 bytes of `x` with `near_sdk::env::value_return`. |
| `slow` | Adds 1 to `transfers`, then returns a promise that calls this contract's own `slow_done` method with 5 Tgas. Use `Promise::new(env::current_account_id()).function_call("slow_done".to_string(), vec![], NearToken::from_yoctonear(0), Gas::from_tgas(5))`. |

Add `pub fn slow_done(&self)`, which does nothing. Unused arguments need `#[allow(unused_variables)]` on the method. `ft_transfer` returns nothing in Rust except where the table says so, so use `near_sdk::env::promise_return` for `slow`.
