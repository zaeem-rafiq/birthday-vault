//! The 13 checks for contracts/collection, step S5: minting.
use near_workspaces::network::Sandbox;
use near_workspaces::result::ExecutionFinalResult;
use near_workspaces::types::{Gas, NearToken};
use near_workspaces::{Account, Contract, Worker};
use serde_json::{json, Value};

const MINT_COST: u128 = 25_000_000_000_000_000_000_000;
const FUNDING: u128 = 2_500_000_000_000_000_000_000;
const REGISTRATION: u128 = 1_250_000_000_000_000_000_000;
const BYTE_COST: u128 = 10_000_000_000_000_000_000;
const TOLERANCE: u128 = 1_000_000_000_000_000_000_000;

struct Setup {
    worker: Worker<Sandbox>,
    root: Account,
    guardian: Account,
    coll: Contract,
    usdc: Contract,
}

async fn account(root: &Account, name: &str, near: u128) -> anyhow::Result<Account> {
    Ok(root
        .create_subaccount(name)
        .initial_balance(NearToken::from_near(near))
        .transact()
        .await?
        .into_result()?)
}

/// A Demo collection whose USDC is tUSDC, or the hostile token when `hostile_usdc` is true.
async fn setup(hostile_usdc: bool, set_code: bool) -> anyhow::Result<Setup> {
    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account()?;
    let guardian = account(&root, "guardian", 20).await?;
    let usdc = if hostile_usdc {
        let c = account(&root, "usdc", 10)
            .await?
            .deploy(&std::fs::read(
                "../target/near/hostile_token/hostile_token.wasm",
            )?)
            .await?
            .into_result()?;
        let r = c
            .call("set_behavior")
            .args_json(json!({ "mode": "", "fail_registration": true }))
            .transact()
            .await?;
        assert!(r.is_success());
        c
    } else {
        let c = account(&root, "usdc", 10)
            .await?
            .deploy(&std::fs::read("../target/near/tusdc/tusdc.wasm")?)
            .await?
            .into_result()?;
        let r = c
            .call("new")
            .args_json(json!({ "hub_id": root.id() }))
            .transact()
            .await?;
        assert!(r.is_success());
        c
    };
    let coll = account(&root, "bv", 30)
        .await?
        .deploy(&std::fs::read("../target/near/collection/collection.wasm")?)
        .await?
        .into_result()?;
    let r = coll.call("new").args_json(json!({ "usdc": usdc.id(), "mode": "Demo", "min_lock_ns": "1000000000", "max_lock_ns": "3600000000000" })).transact().await?;
    assert!(r.is_success(), "init failed: {:?}", r.failures());
    if set_code {
        let r = coll
            .call("set_vault_code")
            .args_borsh(std::fs::read("../target/near/vault/vault.wasm")?)
            .max_gas()
            .transact()
            .await?;
        assert!(r.is_success(), "set_vault_code failed: {:?}", r.failures());
    }
    Ok(Setup {
        worker,
        root,
        guardian,
        coll,
        usdc,
    })
}

async fn now(s: &Setup) -> anyhow::Result<u64> {
    Ok(s.worker.view_block().await?.timestamp())
}

async fn mint(
    s: &Setup,
    who: &Account,
    child: &str,
    unlock: u64,
    title: Option<&str>,
    deposit: u128,
) -> anyhow::Result<ExecutionFinalResult> {
    Ok(who
        .call(s.coll.id(), "mint_vault")
        .args_json(json!({ "child_id": child, "unlock_ns": unlock.to_string(), "title": title }))
        .deposit(NearToken::from_yoctonear(deposit))
        .gas(Gas::from_tgas(100))
        .transact()
        .await?)
}

fn burnt(r: &ExecutionFinalResult) -> u128 {
    r.outcomes()
        .iter()
        .map(|o| o.tokens_burnt.as_yoctonear())
        .sum()
}

async fn balance(s: &Setup, id: &near_workspaces::AccountId) -> anyhow::Result<u128> {
    Ok(s.worker.view_account(id).await?.balance.as_yoctonear())
}

async fn supply(s: &Setup) -> anyhow::Result<String> {
    Ok(s.coll.view("nft_total_supply").await?.json()?)
}

#[tokio::test]
async fn mint_sets_owner_to_guardian() -> anyhow::Result<()> {
    let s = setup(false, true).await?;
    let unlock = now(&s).await? + 60_000_000_000;
    let r = mint(
        &s,
        &s.guardian,
        "child.test.near",
        unlock,
        Some("For Ada"),
        MINT_COST,
    )
    .await?;
    assert!(r.is_success(), "mint failed: {:?}", r.failures());
    let token: Value = s
        .coll
        .view("nft_token")
        .args_json(json!({ "token_id": "bv-1" }))
        .await?
        .json()?;
    assert_eq!(token["owner_id"], s.guardian.id().to_string());
    assert_eq!(token["metadata"]["title"], "For Ada");
    let info: Value = s
        .coll
        .view("vault_info")
        .args_json(json!({ "token_id": "bv-1" }))
        .await?
        .json()?;
    assert_eq!(info["vault_id"], format!("bv-1.{}", s.coll.id()));
    assert_eq!(info["guardian"], s.guardian.id().to_string());
    assert_eq!(info["child"], "child.test.near");
    assert_eq!(info["locked"], true);
    assert_eq!(info["pending"], 0);
    assert_eq!(supply(&s).await?, "1");
    Ok(())
}

#[tokio::test]
async fn mint_registers_vault_with_usdc() -> anyhow::Result<()> {
    let s = setup(false, true).await?;
    let unlock = now(&s).await? + 60_000_000_000;
    assert!(
        mint(&s, &s.guardian, "child.test.near", unlock, None, MINT_COST)
            .await?
            .is_success()
    );
    let reg: Value = s
        .usdc
        .view("storage_balance_of")
        .args_json(json!({ "account_id": format!("bv-1.{}", s.coll.id()) }))
        .await?
        .json()?;
    assert!(!reg.is_null(), "the vault must be registered with USDC");
    Ok(())
}

#[tokio::test]
async fn the_vault_has_code_and_no_keys() -> anyhow::Result<()> {
    let s = setup(false, true).await?;
    let unlock = now(&s).await? + 60_000_000_000;
    assert!(
        mint(&s, &s.guardian, "child.test.near", unlock, None, MINT_COST)
            .await?
            .is_success()
    );
    let vault: near_workspaces::AccountId = format!("bv-1.{}", s.coll.id()).parse()?;
    assert_eq!(
        balance(&s, &vault).await?,
        FUNDING,
        "the vault must hold exactly VAULT_FUNDING"
    );
    let keys = s.worker.view_access_keys(&vault).await?;
    assert!(keys.is_empty(), "the vault must have no access keys");
    let r = s
        .guardian
        .call(&vault, "ft_on_transfer")
        .args_json(json!({ "sender_id": s.guardian.id(), "amount": "1", "msg": "" }))
        .transact()
        .await?;
    assert!(
        r.is_success(),
        "the vault must run the global vault code: {:?}",
        r.failures()
    );
    assert_eq!(r.json::<String>()?, "0");
    Ok(())
}

#[tokio::test]
async fn mint_refund_exact() -> anyhow::Result<()> {
    let s = setup(false, true).await?;
    let unlock = now(&s).await? + 60_000_000_000;
    let payer_before = balance(&s, s.guardian.id()).await?;
    let usage_before = s.worker.view_account(s.coll.id()).await?.storage_usage as u128;
    let r = mint(
        &s,
        &s.guardian,
        "child.test.near",
        unlock,
        Some("A title"),
        2 * MINT_COST,
    )
    .await?;
    assert!(r.is_success(), "mint failed: {:?}", r.failures());
    let usage_after = s.worker.view_account(s.coll.id()).await?.storage_usage as u128;
    let storage = (usage_after - usage_before) * BYTE_COST;
    let spent = payer_before - balance(&s, s.guardian.id()).await?;
    let expected = FUNDING + REGISTRATION + storage + burnt(&r);
    assert!(
        spent.abs_diff(expected) <= TOLERANCE,
        "payer spent {spent}, expected about {expected}"
    );
    Ok(())
}

#[tokio::test]
async fn lock_date_validation() -> anyhow::Result<()> {
    let s = setup(false, true).await?;
    let t = now(&s).await?;
    for unlock in [t, t + 7_200_000_000_000] {
        let before = balance(&s, s.guardian.id()).await?;
        let r = mint(&s, &s.guardian, "child.test.near", unlock, None, MINT_COST).await?;
        assert!(r.is_failure(), "unlock {unlock} must be refused");
        assert!(
            before - balance(&s, s.guardian.id()).await? <= burnt(&r) + TOLERANCE,
            "a refused mint must refund the deposit"
        );
    }
    assert_eq!(supply(&s).await?, "0");
    Ok(())
}

#[tokio::test]
async fn mint_input_validation() -> anyhow::Result<()> {
    let s = setup(false, true).await?;
    let unlock = now(&s).await? + 60_000_000_000;
    let me = s.guardian.id().to_string();
    let coll = s.coll.id().to_string();
    let inner = format!("x.{coll}");
    let long = "t".repeat(65);
    let cases: Vec<(&str, Option<&str>, u128)> = vec![
        (me.as_str(), None, MINT_COST),
        (coll.as_str(), None, MINT_COST),
        (inner.as_str(), None, MINT_COST),
        ("child.test.near", Some(long.as_str()), MINT_COST),
        ("child.test.near", None, MINT_COST - 1),
    ];
    for (child, title, deposit) in cases {
        let r = mint(&s, &s.guardian, child, unlock, title, deposit).await?;
        assert!(
            r.is_failure(),
            "mint must be refused for child {child}, deposit {deposit}"
        );
    }
    assert_eq!(supply(&s).await?, "0");
    Ok(())
}

#[tokio::test]
async fn mint_needs_vault_code() -> anyhow::Result<()> {
    let s = setup(false, false).await?;
    let unlock = now(&s).await? + 60_000_000_000;
    let r = mint(&s, &s.guardian, "child.test.near", unlock, None, MINT_COST).await?;
    assert!(
        r.is_failure() && format!("{:?}", r.failures()).contains("vault code not set"),
        "{:?}",
        r.failures()
    );
    Ok(())
}

#[tokio::test]
async fn mint_fails_cleanly_if_registration_fails() -> anyhow::Result<()> {
    let s = setup(true, true).await?;
    let unlock = now(&s).await? + 60_000_000_000;
    let before = balance(&s, s.guardian.id()).await?;
    let r = mint(&s, &s.guardian, "child.test.near", unlock, None, MINT_COST).await?;
    let spent = before - balance(&s, s.guardian.id()).await?;
    assert!(
        spent.abs_diff(FUNDING + burnt(&r)) <= TOLERANCE,
        "refund must keep only the vault funding: spent {spent}"
    );
    assert_eq!(supply(&s).await?, "0");
    let minted: bool = r.json()?;
    assert!(!minted, "a failed mint must return false");
    Ok(())
}

#[tokio::test]
async fn mint_fails_cleanly_if_account_exists() -> anyhow::Result<()> {
    let s = setup(false, true).await?;
    // Take the next vault id first, so account creation fails and registration succeeds.
    s.coll
        .as_account()
        .create_subaccount("bv-1")
        .initial_balance(NearToken::from_millinear(10))
        .transact()
        .await?
        .into_result()?;
    let unlock = now(&s).await? + 60_000_000_000;
    let before = balance(&s, s.guardian.id()).await?;
    let r = mint(&s, &s.guardian, "child.test.near", unlock, None, MINT_COST).await?;
    let spent = before - balance(&s, s.guardian.id()).await?;
    assert!(
        spent.abs_diff(REGISTRATION + burnt(&r)) <= TOLERANCE,
        "refund must keep only the registration: spent {spent}"
    );
    assert_eq!(
        supply(&s).await?,
        "0",
        "no NFT for an account the collection did not create"
    );
    let info: Value = s
        .coll
        .view("vault_info")
        .args_json(json!({ "token_id": "bv-1" }))
        .await?
        .json()?;
    assert!(info.is_null());
    let minted: bool = r.json()?;
    assert!(!minted, "a failed mint must return false");
    Ok(())
}

#[tokio::test]
async fn on_vault_created_is_private() -> anyhow::Result<()> {
    let s = setup(false, true).await?;
    let r = s.guardian.call(s.coll.id(), "on_vault_created")
        .args_json(json!({ "token_id": "bv-7", "guardian": s.guardian.id(), "child": "child.test.near", "unlock_ns": "1", "title": null, "payer": s.guardian.id(), "deposit": "0" }))
        .max_gas().transact().await?;
    assert!(
        r.is_failure()
            && format!("{:?}", r.failures()).contains("Method on_vault_created is private"),
        "{:?}",
        r.failures()
    );
    assert_eq!(supply(&s).await?, "0");
    Ok(())
}

#[tokio::test]
async fn failed_mint_is_not_a_vault() -> anyhow::Result<()> {
    let s = setup(true, true).await?;
    let unlock = now(&s).await? + 60_000_000_000;
    let _ = mint(&s, &s.guardian, "child.test.near", unlock, None, MINT_COST).await?;
    let info: Value = s
        .coll
        .view("vault_info_by_account")
        .args_json(json!({ "vault_id": format!("bv-1.{}", s.coll.id()) }))
        .await?
        .json()?;
    assert!(info.is_null(), "a failed mint must not look like a vault");
    let info: Value = s.coll.view("contract_info").await?.json()?;
    assert_eq!(info["next_serial"], 2, "a serial is never reused");
    Ok(())
}

#[tokio::test]
async fn vault_info_by_account_finds_real_vaults() -> anyhow::Result<()> {
    let s = setup(false, true).await?;
    let unlock = now(&s).await? + 60_000_000_000;
    assert!(
        mint(&s, &s.guardian, "child.test.near", unlock, None, MINT_COST)
            .await?
            .is_success()
    );
    let info: Value = s
        .coll
        .view("vault_info_by_account")
        .args_json(json!({ "vault_id": format!("bv-1.{}", s.coll.id()) }))
        .await?
        .json()?;
    assert_eq!(info["owner_id"], s.guardian.id().to_string());
    let other: Value = s
        .coll
        .view("vault_info_by_account")
        .args_json(json!({ "vault_id": "bv-1.other.test.near" }))
        .await?
        .json()?;
    assert!(other.is_null());
    let owned: Vec<Value> = s
        .coll
        .view("nft_tokens_for_owner")
        .args_json(json!({ "account_id": s.guardian.id() }))
        .await?
        .json()?;
    assert_eq!(owned.len(), 1);
    Ok(())
}

#[tokio::test]
async fn set_vault_code_needs_balance() -> anyhow::Result<()> {
    let s = setup(false, false).await?;
    let spare = s
        .worker
        .view_account(s.coll.id())
        .await?
        .balance
        .as_yoctonear();
    let usage = s.worker.view_account(s.coll.id()).await?.storage_usage as u128;
    let keep = usage * BYTE_COST + 200_000_000_000_000_000_000_000;
    let r = s
        .coll
        .as_account()
        .transfer_near(s.root.id(), NearToken::from_yoctonear(spare - keep))
        .await?;
    assert!(r.is_success());
    // The parent calls, so the collection pays no gas and the contract itself must refuse.
    let r = s
        .root
        .call(s.coll.id(), "set_vault_code")
        .args_borsh(std::fs::read("../target/near/vault/vault.wasm")?)
        .max_gas()
        .transact()
        .await?;
    assert!(
        r.is_failure(),
        "set_vault_code must refuse when the balance cannot cover the code"
    );
    assert!(
        format!("{:?}", r.failures()).contains("not enough balance"),
        "wrong refusal: {:?}",
        r.failures()
    );
    Ok(())
}
