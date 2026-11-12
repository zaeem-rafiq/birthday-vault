//! Starts a local sandbox with the real contracts for the web end-to-end tests.
//!
//! Run from the `tests` folder: `cargo run -q -p tests --example e2e_sandbox`.
//! It deploys tUSDC, the demo hub and a Demo collection, adds a pool of hub keys that may call
//! only `start_demo`, then prints one line `E2E_READY {json}` with the RPC address, the ids and the
//! pool keys. The keys exist only in this throwaway sandbox. It stops the sandbox when its
//! standard input closes, which is how the Playwright teardown ends it.
use near_workspaces::types::{AccessKey, KeyType, NearToken, SecretKey};
use near_workspaces::Account;
use serde_json::json;
use tokio::io::AsyncReadExt;

const POOL_SIZE: usize = 3;

async fn sub(root: &Account, name: &str, near: u128) -> anyhow::Result<Account> {
    Ok(root
        .create_subaccount(name)
        .initial_balance(NearToken::from_near(near))
        .transact()
        .await?
        .into_result()?)
}

fn wasm(name: &str) -> anyhow::Result<Vec<u8>> {
    let path = format!("../target/near/{name}/{name}.wasm");
    std::fs::read(&path).map_err(|e| anyhow::anyhow!("{path}: {e}. Build the contracts first."))
}

fn ok(r: near_workspaces::result::ExecutionFinalResult, what: &str) -> anyhow::Result<()> {
    anyhow::ensure!(r.is_success(), "{what} failed: {:?}", r.failures());
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account()?;

    let hub = sub(&root, "hub", 200)
        .await?
        .deploy(&wasm("demo_hub")?)
        .await?
        .into_result()?;
    let usdc = sub(&root, "tusdc", 10)
        .await?
        .deploy(&wasm("tusdc")?)
        .await?
        .into_result()?;
    let coll = sub(&root, "bv", 30)
        .await?
        .deploy(&wasm("collection")?)
        .await?
        .into_result()?;

    ok(
        usdc.call("new")
            .args_json(json!({ "hub_id": hub.id() }))
            .transact()
            .await?,
        "tusdc new",
    )?;
    ok(
        hub.call("new")
            .args_json(json!({ "usdc": usdc.id(), "hour_ns": "3600000000000", "day_ns": "86400000000000", "hourly_limit": 1000, "daily_limit": 1000 }))
            .transact()
            .await?,
        "hub new",
    )?;
    ok(
        coll.call("new")
            .args_json(json!({ "usdc": usdc.id(), "mode": "Demo", "min_lock_ns": "1000000000", "max_lock_ns": "3600000000000" }))
            .transact()
            .await?,
        "collection new",
    )?;
    ok(
        coll.call("set_vault_code")
            .args_borsh(wasm("vault")?)
            .max_gas()
            .transact()
            .await?,
        "set_vault_code",
    )?;

    let mut pool = Vec::new();
    for _ in 0..POOL_SIZE {
        let sk = SecretKey::from_random(KeyType::ED25519);
        ok(
            hub.as_account()
                .batch(hub.id())
                .add_key(
                    sk.public_key(),
                    AccessKey::function_call_access(
                        hub.id(),
                        &["start_demo"],
                        Some(NearToken::from_near(1)),
                    ),
                )
                .transact()
                .await?,
            "add pool key",
        )?;
        pool.push(sk.to_string());
    }

    println!(
        "E2E_READY {}",
        json!({
            "rpc": worker.rpc_addr(),
            "collection": coll.id(),
            "hub": hub.id(),
            "usdc": usdc.id(),
            "hubKeys": pool.join(","),
        })
    );

    // Keep the sandbox alive until the test run closes standard input.
    let mut buf = [0u8; 64];
    let mut stdin = tokio::io::stdin();
    while stdin.read(&mut buf).await? > 0 {}
    drop(worker);
    Ok(())
}
