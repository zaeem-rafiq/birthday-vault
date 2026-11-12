//! Plan section 6, T9 and T24: a Production collection on a mainnet-style id, and the
//! mint cost at maximum field lengths. Written by Claude Code in step S9.
use near_workspaces::types::{AccessKey, AccountDetailsPatch, Gas, KeyType, NearToken, SecretKey};
use near_workspaces::{Account, AccountId, Contract};
use serde_json::{json, Value};

const MINT_COST: u128 = 25_000_000_000_000_000_000_000;
const FUNDING: u128 = 2_500_000_000_000_000_000_000;
const REGISTRATION: u128 = 1_250_000_000_000_000_000_000;
const BYTE_COST: u128 = 10_000_000_000_000_000_000;
const DAY_NS: u64 = 86_400_000_000_000;
const MAX_LOCK_NS: u64 = 1_262_304_000_000_000_000;

async fn deploy_usdc(root: &Account, hub: &AccountId) -> anyhow::Result<Contract> {
    let usdc = root
        .create_subaccount("usdc")
        .initial_balance(NearToken::from_near(10))
        .transact()
        .await?
        .into_result()?
        .deploy(&std::fs::read("../target/near/tusdc/tusdc.wasm")?)
        .await?
        .into_result()?;
    assert!(usdc
        .call("new")
        .args_json(json!({ "hub_id": hub }))
        .transact()
        .await?
        .is_success());
    Ok(usdc)
}

async fn mint(
    coll: &AccountId,
    who: &Account,
    child: &str,
    unlock: u64,
    title: Option<&str>,
) -> anyhow::Result<near_workspaces::result::ExecutionFinalResult> {
    Ok(who
        .call(coll, "mint_vault")
        .args_json(json!({ "child_id": child, "unlock_ns": unlock.to_string(), "title": title }))
        .deposit(NearToken::from_yoctonear(MINT_COST))
        .gas(Gas::from_tgas(100))
        .transact()
        .await?)
}

#[tokio::test]
async fn production_mint_validation() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account()?;
    // The sandbox has no "near" registrar, so the mainnet-style id is written into state directly.
    let id: AccountId = "bv.near".parse()?;
    let sk = SecretKey::from_seed(KeyType::ED25519, "production collection");
    worker
        .patch(&id)
        .account(AccountDetailsPatch::default().balance(NearToken::from_near(30)))
        .access_key(sk.public_key(), AccessKey::full_access())
        .transact()
        .await?;
    let coll = Account::from_secret_key(id, sk, &worker)
        .deploy(&std::fs::read("../target/near/collection/collection.wasm")?)
        .await?
        .into_result()?;
    let usdc = deploy_usdc(&root, root.id()).await?;
    // T9: a mainnet id refuses a Production minimum under 24 h, a maximum over 40 years, and Demo
    // mode. A failed init leaves the account uninitialized, so the good init below still runs.
    for (mode, min, max, msg) in [
        ("Production", DAY_NS - 1, MAX_LOCK_NS, "bad lock limits"),
        ("Production", DAY_NS, MAX_LOCK_NS + 1, "bad lock limits"),
        ("Demo", DAY_NS, MAX_LOCK_NS, "mode does not match network"),
    ] {
        let r = coll
            .call("new")
            .args_json(json!({ "usdc": usdc.id(), "mode": mode, "min_lock_ns": min.to_string(), "max_lock_ns": max.to_string() }))
            .transact()
            .await?;
        assert!(
            r.is_failure() && format!("{:?}", r.failures()).contains(msg),
            "{mode} init with min {min}, max {max} on bv.near must fail with {msg:?}: {:?}",
            r.failures()
        );
    }
    let r = coll.call("new").args_json(json!({ "usdc": usdc.id(), "mode": "Production", "min_lock_ns": DAY_NS.to_string(), "max_lock_ns": MAX_LOCK_NS.to_string() })).transact().await?;
    assert!(
        r.is_success(),
        "Production init on bv.near failed: {:?}",
        r.failures()
    );
    let r = coll
        .call("set_vault_code")
        .args_borsh(std::fs::read("../target/near/vault/vault.wasm")?)
        .max_gas()
        .transact()
        .await?;
    assert!(r.is_success(), "set_vault_code failed: {:?}", r.failures());
    let meta: Value = coll.view("nft_metadata").await?.json()?;
    assert_eq!(meta["name"], "Birthday Vault");
    let guardian = root
        .create_subaccount("guardian")
        .initial_balance(NearToken::from_near(5))
        .transact()
        .await?
        .into_result()?;
    let now = worker.view_block().await?.timestamp();
    for unlock in [now + 3_600_000_000_000, now + MAX_LOCK_NS + DAY_NS] {
        let r = mint(coll.id(), &guardian, "child.test.near", unlock, None).await?;
        assert!(
            r.is_failure() && format!("{:?}", r.failures()).contains("bad unlock time"),
            "unlock {unlock}: {:?}",
            r.failures()
        );
    }
    let minted: bool = mint(
        coll.id(),
        &guardian,
        "child.test.near",
        now + 2 * DAY_NS,
        None,
    )
    .await?
    .json()?;
    assert!(minted, "a two-day lock must mint in Production");
    let info: Value = coll
        .view("vault_info")
        .args_json(json!({ "token_id": "bv-1" }))
        .await?
        .json()?;
    assert_eq!(info["vault_id"], "bv-1.bv.near");
    assert_eq!(info["mode"], "Production");
    Ok(())
}

#[tokio::test]
async fn mint_state_fits_cost() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account()?;
    let usdc = deploy_usdc(&root, root.id()).await?;
    let coll = root
        .create_subaccount("bv")
        .initial_balance(NearToken::from_near(30))
        .transact()
        .await?
        .into_result()?
        .deploy(&std::fs::read("../target/near/collection/collection.wasm")?)
        .await?
        .into_result()?;
    assert!(coll.call("new").args_json(json!({ "usdc": usdc.id(), "mode": "Demo", "min_lock_ns": "1000000000", "max_lock_ns": "3600000000000" })).transact().await?.is_success());
    assert!(coll
        .call("set_vault_code")
        .args_borsh(std::fs::read("../target/near/vault/vault.wasm")?)
        .max_gas()
        .transact()
        .await?
        .is_success());
    // Longest ids (64 characters) and the longest title (64 bytes).
    let long = "g".repeat(64 - ".test.near".len());
    let guardian = root
        .create_subaccount(&long)
        .initial_balance(NearToken::from_near(5))
        .transact()
        .await?
        .into_result()?;
    assert_eq!(guardian.id().as_str().len(), 64);
    let child = format!("{}.test.near", "c".repeat(64 - ".test.near".len()));
    let title = "T".repeat(64);
    let before = worker.view_account(coll.id()).await?.storage_usage as u128;
    let now = worker.view_block().await?.timestamp();
    let minted: bool = mint(
        coll.id(),
        &guardian,
        &child,
        now + 60_000_000_000,
        Some(&title),
    )
    .await?
    .json()?;
    assert!(minted);
    let bytes = worker.view_account(coll.id()).await?.storage_usage as u128 - before;
    let needed = FUNDING + REGISTRATION + bytes * BYTE_COST * 13 / 10;
    println!("mint at maximum lengths stores {bytes} bytes; cost with 30% margin {needed} yocto; MINT_COST {MINT_COST}");
    assert!(
        needed <= MINT_COST,
        "MINT_COST must cover funding, registration and 1.3 x the state: {bytes} bytes"
    );
    Ok(())
}
