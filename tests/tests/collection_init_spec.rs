//! The 9 checks for contracts/collection, step S4.
use near_workspaces::network::Sandbox;
use near_workspaces::types::NearToken;
use near_workspaces::{Account, Contract, Worker};
use serde_json::{json, Value};

const MIN: &str = "1000000000";
const MAX: &str = "3600000000000";

struct Setup {
    _worker: Worker<Sandbox>,
    root: Account,
    stranger: Account,
    coll: Contract,
}

async fn setup() -> anyhow::Result<Setup> {
    let wasm = std::fs::read("../target/near/collection/collection.wasm")?;
    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account()?;
    let stranger = root
        .create_subaccount("stranger")
        .initial_balance(NearToken::from_near(10))
        .transact()
        .await?
        .into_result()?;
    let acct = root
        .create_subaccount("bv")
        .initial_balance(NearToken::from_near(30))
        .transact()
        .await?
        .into_result()?;
    let coll = acct.deploy(&wasm).await?.into_result()?;
    Ok(Setup {
        _worker: worker,
        root,
        stranger,
        coll,
    })
}

fn init_args(mode: &str, min: &str, max: &str) -> Value {
    json!({ "usdc": "usdc.test.near", "mode": mode, "min_lock_ns": min, "max_lock_ns": max })
}

async fn init_by_self(s: &Setup) -> anyhow::Result<()> {
    let r = s
        .coll
        .call("new")
        .args_json(init_args("Demo", MIN, MAX))
        .transact()
        .await?;
    assert!(
        r.is_success(),
        "Demo init on a sandbox account must work: {:?}",
        r.failures()
    );
    Ok(())
}

async fn info(s: &Setup) -> anyhow::Result<Value> {
    Ok(s.coll.view("contract_info").await?.json()?)
}

#[tokio::test]
async fn mode_must_match_network() -> anyhow::Result<()> {
    let s = setup().await?;
    let r = s
        .coll
        .call("new")
        .args_json(init_args("Production", "86400000000000", MAX))
        .transact()
        .await?;
    assert!(
        r.is_failure(),
        "Production must not start on a .test.near account"
    );
    init_by_self(&s).await?;
    let i = info(&s).await?;
    assert_eq!(i["mode"], "Demo");
    assert_eq!(i["next_serial"], 1);
    assert_eq!(i["min_lock_ns"], MIN);
    assert!(i["vault_code_hash"].is_null());
    Ok(())
}

#[tokio::test]
async fn bad_lock_limits_fail() -> anyhow::Result<()> {
    let s = setup().await?;
    for (min, max) in [("0", MAX), ("5000", "4000")] {
        let r = s
            .coll
            .call("new")
            .args_json(init_args("Demo", min, max))
            .transact()
            .await?;
        assert!(r.is_failure(), "lock limits {min}..{max} must fail");
    }
    Ok(())
}

#[tokio::test]
async fn init_only_once_and_only_by_self_or_parent() -> anyhow::Result<()> {
    let s = setup().await?;
    let r = s
        .stranger
        .call(s.coll.id(), "new")
        .args_json(init_args("Demo", MIN, MAX))
        .transact()
        .await?;
    assert!(r.is_failure(), "a stranger must not initialize");
    let r = s
        .root
        .call(s.coll.id(), "new")
        .args_json(init_args("Demo", MIN, MAX))
        .transact()
        .await?;
    assert!(
        r.is_success(),
        "the parent may initialize: {:?}",
        r.failures()
    );
    let r = s
        .coll
        .call("new")
        .args_json(init_args("Demo", MIN, MAX))
        .transact()
        .await?;
    assert!(r.is_failure(), "initialization must happen only once");
    Ok(())
}

#[tokio::test]
async fn set_vault_code_once_by_self_or_parent() -> anyhow::Result<()> {
    let s = setup().await?;
    init_by_self(&s).await?;
    let vault = std::fs::read("../target/near/vault/vault.wasm")?;
    let r = s
        .stranger
        .call(s.coll.id(), "set_vault_code")
        .args_borsh(vault.clone())
        .max_gas()
        .transact()
        .await?;
    assert!(r.is_failure(), "a stranger must not set the vault code");
    let r = s
        .coll
        .call("set_vault_code")
        .args_borsh(vault.clone())
        .max_gas()
        .transact()
        .await?;
    assert!(
        r.is_success(),
        "self may set the vault code: {:?}",
        r.failures()
    );
    let expect: String = {
        use sha2::{Digest, Sha256};
        use std::fmt::Write;
        let digest = Sha256::digest(&vault);
        digest.iter().fold(String::new(), |mut out, b| {
            let _ = write!(out, "{b:02x}");
            out
        })
    };
    assert_eq!(info(&s).await?["vault_code_hash"], expect);
    let r = s
        .coll
        .call("set_vault_code")
        .args_borsh(vault)
        .max_gas()
        .transact()
        .await?;
    assert!(r.is_failure(), "the vault code must be set only once");
    Ok(())
}

#[tokio::test]
async fn metadata_and_supply() -> anyhow::Result<()> {
    let s = setup().await?;
    init_by_self(&s).await?;
    let m: Value = s.coll.view("nft_metadata").await?.json()?;
    assert_eq!(m["name"], "Birthday Vault (DEMO, play money)");
    assert_eq!(m["symbol"], "VAULT");
    assert_eq!(m["spec"], "nft-1.0.0");
    assert!(m["icon"]
        .as_str()
        .unwrap_or("")
        .starts_with("data:image/svg+xml"));
    let supply: String = s.coll.view("nft_total_supply").await?.json()?;
    assert_eq!(supply, "0");
    Ok(())
}

#[tokio::test]
async fn mainnet_ids_are_classified() -> anyhow::Result<()> {
    let s = setup().await?;
    init_by_self(&s).await?;
    for (id, expect) in [
        ("alice.near", true),
        ("bv.test.near", false),
        ("x.sandbox", false),
        ("a.testnet", false),
        ("near", false),
    ] {
        let got: bool = s
            .coll
            .view("is_mainnet_id")
            .args_json(json!({ "account_id": id }))
            .await?
            .json()?;
        assert_eq!(got, expect, "{id}");
    }
    Ok(())
}

#[tokio::test]
async fn the_collection_wasm_cannot_add_keys_or_upgrade() -> anyhow::Result<()> {
    let mut source = String::new();
    for entry in std::fs::read_dir("../contracts/collection/src")? {
        source.push_str(&std::fs::read_to_string(entry?.path())?);
    }
    for bad in [
        "add_full_access_key",
        "add_access_key",
        "deploy_contract",
        "delete_account",
        "stake",
        "delete_key",
    ] {
        assert!(
            !source.contains(bad),
            "the collection source must not use {bad}"
        );
    }
    Ok(())
}

#[tokio::test]
async fn views_work_before_any_mint() -> anyhow::Result<()> {
    let s = setup().await?;
    init_by_self(&s).await?;
    let i = info(&s).await?;
    assert_eq!(i["usdc"], "usdc.test.near");
    assert_eq!(i["max_lock_ns"], MAX);
    Ok(())
}

#[tokio::test]
async fn parent_can_set_vault_code() -> anyhow::Result<()> {
    let s = setup().await?;
    init_by_self(&s).await?;
    let vault = std::fs::read("../target/near/vault/vault.wasm")?;
    let r = s
        .root
        .call(s.coll.id(), "set_vault_code")
        .args_borsh(vault)
        .max_gas()
        .transact()
        .await?;
    assert!(
        r.is_success(),
        "the parent may set the vault code: {:?}",
        r.failures()
    );
    assert!(info(&s).await?["vault_code_hash"].is_string());
    Ok(())
}
