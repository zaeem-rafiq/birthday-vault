# Vault contract: exact specification

Write only `contracts/vault/src/lib.rs`. `tests/tests/vault_spec.rs` holds the 9 checks it must pass.

**Size:** the wasm must be at most 30 KB. `near-sdk` is too large, so do not use it. Use only `near-sys` 0.4.1 (source in `docs/sources/near-sys-0.4.1/`), with `#![no_std]` and no allocator. Use fixed-size stack buffers. Do not add dependencies.

**Build:** `source scripts/env.sh; cargo build --locked -p vault --target wasm32-unknown-unknown --release`. `cargo near` refuses a crate without near-sdk, so `scripts/vault-check.sh` uses this command.

**File shape**

```rust
#![no_std]
use near_sys as sys;

#[panic_handler]
fn on_panic(_: &core::panic::PanicInfo) -> ! {
    unsafe { sys::panic() }
}
```

Each method is `#[no_mangle] pub extern "C" fn NAME()` with no Rust arguments. Reject bad input with `sys::panic_utf8` and a short message.

**Reading registers:** call a host function with register 0, read `sys::register_len(0)`, reject anything longer than the buffer, then `sys::read_register(0, ptr)`.

**Parent rule:** read `sys::current_account_id(0)` and `sys::predecessor_account_id(0)`. The parent is every byte after the first `.` of the current account id. Panic if there is no dot. A method marked "Parent only" panics unless the predecessor bytes equal the parent bytes.

**Reading JSON arguments:** read `sys::input(0)` into a 512-byte buffer and panic if it is longer. To get a key's value, find `"key"` in the bytes, skip spaces, require `:`, skip spaces, require `"`, then take the bytes up to the next `"`. Panic if the key is missing or the value holds a backslash. Amounts are decimal digit strings. Parse them into `u128` with checked arithmetic, and panic on an empty value, a non-digit or overflow.

**Methods**

| Method | JSON arguments | Who may call | What it does |
|---|---|---|---|
| `ft_send` | `token`, `receiver`, `amount` | Parent only | `sys::promise_batch_create` on `token`, then `sys::promise_batch_action_function_call` with method `ft_transfer`, arguments exactly `{"receiver_id":"<receiver>","amount":"<amount>"}`, deposit 1 yoctoNEAR, gas `30_000_000_000_000`. Then `sys::promise_return`. |
| `near_send` | `receiver`, `amount` | Parent only | `sys::promise_batch_create` on `receiver`, then `sys::promise_batch_action_transfer` with `amount`. Then `sys::promise_return`. |
| `ft_on_transfer` | ignored | Anyone | `sys::value_return` of the 3 bytes `"0"`, quotes included. |
| `nft_on_transfer` | ignored | Anyone | `sys::value_return` of the 4 bytes `true`. |

**Amounts passed to host functions** are pointers to a 16-byte little-endian `u128`.
