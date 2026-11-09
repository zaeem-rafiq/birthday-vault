//! The 7 checks for contracts/vault. Build the contract first, then run:
//! source scripts/env.sh; cargo test --locked -p tests --test vault_spec -- --test-threads=3
use near_workspaces::network::Sandbox;
use near_workspaces::types::NearToken;
use near_workspaces::{Account, Contract, Worker};
use serde_json::{json, Value};

const ONE_NEAR: &str = "1000000000000000000000000";

struct Setup {
    _worker: Worker<Sandbox>,
    parent: Account,
    vault_account: Account,
    vault: Contract,
    stranger: Account,
    receiver: Account,
}

async fn setup() -> anyhow::Result<Setup> {
    let wasm = std::fs::read("../target/near/vault/vault.wasm")?;
    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account()?;
    let parent = root.create_subaccount("parent").initial_balance(NearToken::from_near(40)).transact().await?.into_result()?;
    let vault_account = parent.create_subaccount("vault").initial_balance(NearToken::from_near(20)).transact().await?.into_result()?;
    let vault = vault_account.deploy(&wasm).await?.into_result()?;
    let stranger = root.create_subaccount("stranger").initial_balance(NearToken::from_near(10)).transact().await?.into_result()?;
    let receiver = root.create_subaccount("receiver").initial_balance(NearToken::from_near(1)).transact().await?.into_result()?;
    Ok(Setup { _worker: worker, parent, vault_account, vault, stranger, receiver })
}

fn send_args(s: &Setup) -> Value {
    json!({ "receiver": s.receiver.id(), "amount": ONE_NEAR })
}

#[tokio::test]
async fn vault_rejects_every_caller_but_parent() -> anyhow::Result<()> {
    let s = setup().await?;
    let r = s.stranger.call(s.vault.id(), "near_send").args_json(send_args(&s)).transact().await?;
    assert!(r.is_failure(), "a stranger must not be able to call near_send");
    Ok(())
}

#[tokio::test]
async fn stranger_cannot_call_ft_send() -> anyhow::Result<()> {
    let s = setup().await?;
    let args = json!({ "token": s.receiver.id(), "receiver": s.stranger.id(), "amount": "1" });
    let r = s.stranger.call(s.vault.id(), "ft_send").args_json(args).deposit(NearToken::from_yoctonear(1)).transact().await?;
    assert!(r.is_failure(), "a stranger must not be able to call ft_send");
    Ok(())
}

#[tokio::test]
async fn vault_cannot_call_its_own_near_send() -> anyhow::Result<()> {
    let s = setup().await?;
    let r = s.vault_account.call(s.vault.id(), "near_send").args_json(send_args(&s)).transact().await?;
    assert!(r.is_failure(), "the vault itself is not its parent");
    Ok(())
}

#[tokio::test]
async fn refused_calls_move_no_money() -> anyhow::Result<()> {
    let s = setup().await?;
    let before = s.receiver.view_account().await?.balance;
    let _ = s.stranger.call(s.vault.id(), "near_send").args_json(send_args(&s)).transact().await?;
    assert_eq!(s.receiver.view_account().await?.balance, before, "a refused call moved money");
    Ok(())
}

#[tokio::test]
async fn parent_can_call_near_send_and_money_arrives() -> anyhow::Result<()> {
    let s = setup().await?;
    let before = s.receiver.view_account().await?.balance;
    let r = s.parent.call(s.vault.id(), "near_send").args_json(send_args(&s)).transact().await?;
    assert!(r.is_success(), "the parent call failed: {:?}", r.failures());
    let after = s.receiver.view_account().await?.balance;
    assert_eq!(after.as_yoctonear() - before.as_yoctonear(), NearToken::from_near(1).as_yoctonear());
    Ok(())
}

#[tokio::test]
async fn ft_on_transfer_keeps_the_tokens() -> anyhow::Result<()> {
    let s = setup().await?;
    let args = json!({ "sender_id": s.stranger.id(), "amount": "5", "msg": "" });
    let r = s.stranger.call(s.vault.id(), "ft_on_transfer").args_json(args).transact().await?;
    assert!(r.is_success(), "ft_on_transfer failed: {:?}", r.failures());
    assert_eq!(r.json::<String>()?, "0");
    Ok(())
}

#[tokio::test]
async fn nft_on_transfer_sends_stray_nfts_back() -> anyhow::Result<()> {
    let s = setup().await?;
    let args = json!({ "sender_id": s.stranger.id(), "previous_owner_id": s.stranger.id(), "token_id": "1", "msg": "" });
    let r = s.stranger.call(s.vault.id(), "nft_on_transfer").args_json(args).transact().await?;
    assert!(r.is_success(), "nft_on_transfer failed: {:?}", r.failures());
    assert!(r.json::<bool>()?);
    Ok(())
}
