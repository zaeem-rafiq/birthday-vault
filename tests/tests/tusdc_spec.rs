//! The 9 checks for contracts/tusdc.
use near_workspaces::network::Sandbox;
use near_workspaces::types::NearToken;
use near_workspaces::{Account, Contract, Worker};
use serde_json::{json, Value};

struct Setup {
    _worker: Worker<Sandbox>,
    hub: Account,
    stranger: Account,
    alice: Account,
    bob: Account,
    token: Contract,
}

async fn account(root: &Account, name: &str) -> anyhow::Result<Account> {
    Ok(root.create_subaccount(name).initial_balance(NearToken::from_near(10)).transact().await?.into_result()?)
}

async fn setup() -> anyhow::Result<Setup> {
    let wasm = std::fs::read("../target/near/tusdc/tusdc.wasm")?;
    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account()?;
    let hub = account(&root, "hub").await?;
    let stranger = account(&root, "stranger").await?;
    let alice = account(&root, "alice").await?;
    let bob = account(&root, "bob").await?;
    let token_account = account(&root, "tusdc").await?;
    let token = token_account.deploy(&wasm).await?.into_result()?;
    let r = token.call("new").args_json(json!({ "hub_id": hub.id() })).transact().await?;
    assert!(r.is_success(), "new failed: {:?}", r.failures());
    Ok(Setup { _worker: worker, hub, stranger, alice, bob, token })
}

async fn balance(s: &Setup, who: &Account) -> anyhow::Result<String> {
    Ok(s.token.view("ft_balance_of").args_json(json!({ "account_id": who.id() })).await?.json()?)
}

async fn faucet(s: &Setup, caller: &Account, to: &Account) -> anyhow::Result<bool> {
    let r = caller.call(s.token.id(), "faucet").args_json(json!({ "account_id": to.id() })).transact().await?;
    Ok(r.is_success())
}

#[tokio::test]
async fn metadata_is_play_money() -> anyhow::Result<()> {
    let s = setup().await?;
    let m: Value = s.token.view("ft_metadata").await?.json()?;
    assert_eq!(m["name"], "Test USDC (play money)");
    assert_eq!(m["symbol"], "tUSDC");
    assert_eq!(m["decimals"], 6);
    Ok(())
}

#[tokio::test]
async fn registration_costs_the_standard_amount() -> anyhow::Result<()> {
    let s = setup().await?;
    let b: Value = s.token.view("storage_balance_bounds").await?.json()?;
    assert_eq!(b["min"], "1250000000000000000000");
    Ok(())
}

#[tokio::test]
async fn new_mints_nothing() -> anyhow::Result<()> {
    let s = setup().await?;
    let supply: String = s.token.view("ft_total_supply").await?.json()?;
    assert_eq!(supply, "0");
    Ok(())
}

#[tokio::test]
async fn hub_can_faucet_a_new_account() -> anyhow::Result<()> {
    let s = setup().await?;
    assert!(faucet(&s, &s.hub, &s.alice).await?, "the hub must be able to use the faucet");
    assert_eq!(balance(&s, &s.alice).await?, "5000000");
    let reg: Value = s.token.view("storage_balance_of").args_json(json!({ "account_id": s.alice.id() })).await?.json()?;
    assert!(!reg.is_null(), "faucet must register the account");
    Ok(())
}

#[tokio::test]
async fn faucet_twice_adds_up() -> anyhow::Result<()> {
    let s = setup().await?;
    assert!(faucet(&s, &s.hub, &s.alice).await?);
    assert!(faucet(&s, &s.hub, &s.alice).await?, "a second faucet call must work");
    assert_eq!(balance(&s, &s.alice).await?, "10000000");
    let supply: String = s.token.view("ft_total_supply").await?.json()?;
    assert_eq!(supply, "10000000");
    Ok(())
}

#[tokio::test]
async fn stranger_cannot_use_the_faucet() -> anyhow::Result<()> {
    let s = setup().await?;
    assert!(!faucet(&s, &s.stranger, &s.stranger).await?, "a stranger must not mint");
    assert_eq!(balance(&s, &s.stranger).await?, "0");
    Ok(())
}

#[tokio::test]
async fn only_the_hub_can_register() -> anyhow::Result<()> {
    let s = setup().await?;
    let r = s.stranger.call(s.token.id(), "register").args_json(json!({ "account_id": s.bob.id() })).transact().await?;
    assert!(r.is_failure(), "a stranger must not register accounts");
    for _ in 0..2 {
        let r = s.hub.call(s.token.id(), "register").args_json(json!({ "account_id": s.bob.id() })).transact().await?;
        assert!(r.is_success(), "register must work, also when repeated: {:?}", r.failures());
    }
    let reg: Value = s.token.view("storage_balance_of").args_json(json!({ "account_id": s.bob.id() })).await?.json()?;
    assert!(!reg.is_null());
    Ok(())
}

#[tokio::test]
async fn holders_can_transfer() -> anyhow::Result<()> {
    let s = setup().await?;
    assert!(faucet(&s, &s.hub, &s.alice).await?);
    let r = s.hub.call(s.token.id(), "register").args_json(json!({ "account_id": s.bob.id() })).transact().await?;
    assert!(r.is_success());
    let r = s.alice.call(s.token.id(), "ft_transfer").args_json(json!({ "receiver_id": s.bob.id(), "amount": "1000" }))
        .deposit(NearToken::from_yoctonear(1)).transact().await?;
    assert!(r.is_success(), "ft_transfer failed: {:?}", r.failures());
    assert_eq!(balance(&s, &s.bob).await?, "1000");
    assert_eq!(balance(&s, &s.alice).await?, "4999000");
    Ok(())
}

#[tokio::test]
async fn storage_unregister_cannot_drain_the_contract() -> anyhow::Result<()> {
    let s = setup().await?;
    assert!(faucet(&s, &s.hub, &s.alice).await?);
    let r = s.alice.call(s.token.id(), "storage_unregister").args_json(json!({ "force": true }))
        .deposit(NearToken::from_yoctonear(1)).transact().await?;
    assert!(r.is_failure(), "unregistering must be disabled, or holders could drain the contract");
    assert_eq!(balance(&s, &s.alice).await?, "5000000");
    Ok(())
}
