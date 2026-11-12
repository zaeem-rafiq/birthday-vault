//! The 7 checks for contracts/hostile-token.
use near_workspaces::network::Sandbox;
use near_workspaces::types::NearToken;
use near_workspaces::{Account, Contract, Worker};
use serde_json::{json, Value};

async fn setup() -> anyhow::Result<(Worker<Sandbox>, Account, Contract)> {
    let wasm = std::fs::read("../target/near/hostile_token/hostile_token.wasm")?;
    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account()?;
    let user = root
        .create_subaccount("user")
        .initial_balance(NearToken::from_near(10))
        .transact()
        .await?
        .into_result()?;
    let acct = root
        .create_subaccount("hostile")
        .initial_balance(NearToken::from_near(10))
        .transact()
        .await?
        .into_result()?;
    let token = acct.deploy(&wasm).await?.into_result()?;
    Ok((worker, user, token))
}

async fn behave(user: &Account, t: &Contract, mode: &str, fail_reg: bool) -> anyhow::Result<()> {
    let r = user
        .call(t.id(), "set_behavior")
        .args_json(json!({ "mode": mode, "fail_registration": fail_reg }))
        .transact()
        .await?;
    assert!(r.is_success(), "set_behavior failed: {:?}", r.failures());
    Ok(())
}

async fn transfer(
    user: &Account,
    t: &Contract,
) -> anyhow::Result<near_workspaces::result::ExecutionFinalResult> {
    Ok(user
        .call(t.id(), "ft_transfer")
        .args_json(json!({ "receiver_id": user.id(), "amount": "1" }))
        .deposit(NearToken::from_yoctonear(1))
        .max_gas()
        .transact()
        .await?)
}

async fn count(t: &Contract) -> anyhow::Result<u32> {
    Ok(t.view("transfers").await?.json()?)
}

#[tokio::test]
async fn normal_transfer_counts() -> anyhow::Result<()> {
    let (_w, user, t) = setup().await?;
    assert!(transfer(&user, &t).await?.is_success());
    assert_eq!(count(&t).await?, 1);
    Ok(())
}

#[tokio::test]
async fn fail_mode_refuses() -> anyhow::Result<()> {
    let (_w, user, t) = setup().await?;
    behave(&user, &t, "fail", false).await?;
    assert!(transfer(&user, &t).await?.is_failure());
    assert_eq!(count(&t).await?, 0);
    Ok(())
}

#[tokio::test]
async fn huge_mode_returns_a_huge_value() -> anyhow::Result<()> {
    let (_w, user, t) = setup().await?;
    behave(&user, &t, "huge", false).await?;
    let r = transfer(&user, &t).await?;
    assert!(r.is_success(), "huge failed: {:?}", r.failures());
    assert_eq!(r.raw_bytes()?.len(), 100_000);
    Ok(())
}

#[tokio::test]
async fn slow_mode_uses_a_later_receipt() -> anyhow::Result<()> {
    let (_w, user, t) = setup().await?;
    behave(&user, &t, "slow", false).await?;
    let r = transfer(&user, &t).await?;
    assert!(r.is_success(), "slow failed: {:?}", r.failures());
    assert_eq!(count(&t).await?, 1);
    assert!(r.outcomes().len() >= 3, "slow must add a later receipt");
    Ok(())
}

#[tokio::test]
async fn registration_can_succeed() -> anyhow::Result<()> {
    let (_w, user, t) = setup().await?;
    let r = user
        .call(t.id(), "storage_deposit")
        .args_json(json!({ "account_id": user.id(), "registration_only": true }))
        .deposit(NearToken::from_millinear(2))
        .transact()
        .await?;
    assert!(r.is_success(), "storage_deposit failed: {:?}", r.failures());
    let v: Value = r.json()?;
    assert_eq!(v["total"], "1250000000000000000000");
    Ok(())
}

#[tokio::test]
async fn registration_can_be_refused() -> anyhow::Result<()> {
    let (_w, user, t) = setup().await?;
    behave(&user, &t, "", true).await?;
    let r = user
        .call(t.id(), "storage_deposit")
        .args_json(json!({ "account_id": user.id() }))
        .deposit(NearToken::from_millinear(2))
        .transact()
        .await?;
    assert!(r.is_failure());
    Ok(())
}

#[tokio::test]
async fn balance_is_fixed() -> anyhow::Result<()> {
    let (_w, user, t) = setup().await?;
    let b: String = t
        .view("ft_balance_of")
        .args_json(json!({ "account_id": user.id() }))
        .await?
        .json()?;
    assert_eq!(b, "1000000");
    Ok(())
}
