//! The 11 checks for contracts/collection, step S7: transfer guard, child claim and standard events.
use near_workspaces::network::Sandbox;
use near_workspaces::result::ExecutionFinalResult;
use near_workspaces::types::{Gas, NearToken};
use near_workspaces::{Account, AccountId, Contract, Worker};
use serde_json::{json, Value};

const MINT_COST: u128 = 25_000_000_000_000_000_000_000;
const BUSY: &str = "busy: a withdrawal is still settling";
const LOCKED_PAIR: &str = "locked: only guardian and child";

struct Setup {
    worker: Worker<Sandbox>,
    guardian: Account,
    child: Account,
    friend: Account,
    coll: Contract,
    usdc: Contract,
    vault: AccountId,
    unlock: u64,
    mint_logs: Vec<String>,
}

async fn account(root: &Account, name: &str, near: u128) -> anyhow::Result<Account> {
    Ok(root.create_subaccount(name).initial_balance(NearToken::from_near(near)).transact().await?.into_result()?)
}

async fn now(worker: &Worker<Sandbox>) -> anyhow::Result<u64> {
    Ok(worker.view_block().await?.timestamp())
}

/// A Demo collection on tUSDC with one vault, bv-1, minted by the guardian for the child.
/// It holds 5,000,000 tUSDC and unlocks `lock_secs` seconds after the mint.
async fn setup(lock_secs: u64) -> anyhow::Result<Setup> {
    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account()?;
    let guardian = account(&root, "guardian", 20).await?;
    let child = account(&root, "child", 5).await?;
    let friend = account(&root, "friend", 5).await?;
    let usdc = account(&root, "usdc", 10).await?.deploy(&std::fs::read("../target/near/tusdc/tusdc.wasm")?).await?.into_result()?;
    assert!(usdc.call("new").args_json(json!({ "hub_id": root.id() })).transact().await?.is_success());
    let coll = account(&root, "bv", 30).await?.deploy(&std::fs::read("../target/near/collection/collection.wasm")?).await?.into_result()?;
    let r = coll.call("new").args_json(json!({ "usdc": usdc.id(), "mode": "Demo", "min_lock_ns": "1000000000", "max_lock_ns": "3600000000000" })).transact().await?;
    assert!(r.is_success(), "init failed: {:?}", r.failures());
    let r = coll.call("set_vault_code").args_borsh(std::fs::read("../target/near/vault/vault.wasm")?).max_gas().transact().await?;
    assert!(r.is_success(), "set_vault_code failed: {:?}", r.failures());
    for who in [guardian.id(), child.id(), friend.id()] {
        assert!(root.call(usdc.id(), "register").args_json(json!({ "account_id": who })).transact().await?.is_success());
    }
    let unlock = now(&worker).await? + lock_secs * 1_000_000_000;
    let r = guardian.call(coll.id(), "mint_vault")
        .args_json(json!({ "child_id": child.id(), "unlock_ns": unlock.to_string(), "title": null }))
        .deposit(NearToken::from_yoctonear(MINT_COST)).gas(Gas::from_tgas(100)).transact().await?;
    let mint_logs: Vec<String> = r.logs().iter().map(|l| l.to_string()).collect();
    let minted: bool = r.json()?;
    assert!(minted, "the mint must succeed");
    let vault: AccountId = format!("bv-1.{}", coll.id()).parse()?;
    assert!(root.call(usdc.id(), "faucet").args_json(json!({ "account_id": vault })).transact().await?.is_success());
    Ok(Setup { worker, guardian, child, friend, coll, usdc, vault, unlock, mint_logs })
}

async fn wait_unlock(s: &Setup) -> anyhow::Result<()> {
    while now(&s.worker).await? <= s.unlock {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    Ok(())
}

async fn transfer(s: &Setup, who: &Account, to: &AccountId, token: &str) -> anyhow::Result<ExecutionFinalResult> {
    Ok(who.call(s.coll.id(), "nft_transfer")
        .args_json(json!({ "receiver_id": to, "token_id": token }))
        .deposit(NearToken::from_yoctonear(1)).gas(Gas::from_tgas(30)).transact().await?)
}

async fn claim(s: &Setup, who: &Account, token: &str, yocto: u128) -> anyhow::Result<ExecutionFinalResult> {
    Ok(who.call(s.coll.id(), "claim_as_child")
        .args_json(json!({ "token_id": token }))
        .deposit(NearToken::from_yoctonear(yocto)).gas(Gas::from_tgas(30)).transact().await?)
}

async fn withdraw(s: &Setup, who: &Account, amount: u128) -> anyhow::Result<ExecutionFinalResult> {
    Ok(who.call(s.coll.id(), "withdraw")
        .args_json(json!({ "token_id": "bv-1", "ft_contract": s.usdc.id(), "amount": amount.to_string(), "receiver_id": null }))
        .deposit(NearToken::from_yoctonear(1)).gas(Gas::from_tgas(100)).transact().await?)
}

async fn owner(s: &Setup) -> anyhow::Result<String> {
    let token: Value = s.coll.view("nft_token").args_json(json!({ "token_id": "bv-1" })).await?.json()?;
    Ok(token["owner_id"].as_str().expect("owner_id").to_string())
}

async fn usdc_balance(s: &Setup, who: &AccountId) -> anyhow::Result<u128> {
    let b: String = s.usdc.view("ft_balance_of").args_json(json!({ "account_id": who })).await?.json()?;
    Ok(b.parse()?)
}

fn fails_with(r: &ExecutionFinalResult, msg: &str) -> bool {
    r.is_failure() && format!("{:?}", r.failures()).contains(msg)
}

fn nep171_events<S: AsRef<str>>(logs: &[S]) -> Vec<Value> {
    logs.iter()
        .filter_map(|l| l.as_ref().strip_prefix("EVENT_JSON:"))
        .filter_map(|j| serde_json::from_str::<Value>(j).ok())
        .filter(|e| e["standard"] == "nep171")
        .collect()
}

/// Sets the stored `pending` counter of bv-1. VaultState is the Borsh value under key
/// `[4] ++ borsh("bv-1")` (StorageKey::Vaults), and `pending` is its last 4 bytes.
async fn set_pending(s: &Setup, pending: u32) -> anyhow::Result<()> {
    let mut key = vec![4u8, 4, 0, 0, 0];
    key.extend_from_slice(b"bv-1");
    let state = s.worker.view_state(s.coll.id()).await?;
    let mut value = state.get(&key).expect("VaultState for bv-1 under the Vaults key").clone();
    let n = value.len();
    value[n - 4..].copy_from_slice(&pending.to_le_bytes());
    s.worker.patch_state(s.coll.id(), &key, &value).await?;
    let info: Value = s.coll.view("vault_info").args_json(json!({ "token_id": "bv-1" })).await?.json()?;
    assert_eq!(info["pending"], pending, "the patch must change pending");
    Ok(())
}

#[tokio::test]
async fn transfer_before_unlock_only_between_guardian_and_child() -> anyhow::Result<()> {
    let s = setup(60).await?;
    let r = transfer(&s, &s.guardian, s.friend.id(), "bv-1").await?;
    assert!(fails_with(&r, LOCKED_PAIR), "{:?}", r.failures());
    let r = transfer(&s, &s.friend, s.child.id(), "bv-1").await?;
    assert!(fails_with(&r, "only the owner"), "{:?}", r.failures());
    let r = transfer(&s, &s.guardian, s.child.id(), "bv-9").await?;
    assert!(fails_with(&r, "no such vault"), "{:?}", r.failures());
    assert!(transfer(&s, &s.guardian, s.child.id(), "bv-1").await?.is_success());
    assert_eq!(owner(&s).await?, s.child.id().to_string());
    let r = transfer(&s, &s.child, s.friend.id(), "bv-1").await?;
    assert!(fails_with(&r, LOCKED_PAIR), "{:?}", r.failures());
    assert!(transfer(&s, &s.child, s.guardian.id(), "bv-1").await?.is_success());
    assert_eq!(owner(&s).await?, s.guardian.id().to_string());
    Ok(())
}

#[tokio::test]
async fn transfer_after_unlock_is_open() -> anyhow::Result<()> {
    let s = setup(5).await?;
    wait_unlock(&s).await?;
    let r = transfer(&s, &s.guardian, s.friend.id(), "bv-1").await?;
    assert!(r.is_success(), "{:?}", r.failures());
    assert_eq!(owner(&s).await?, s.friend.id().to_string());
    let r = transfer(&s, &s.guardian, s.child.id(), "bv-1").await?;
    assert!(r.is_failure(), "the old owner must not move the NFT");
    let r = s.friend.call(s.coll.id(), "nft_transfer").args_json(json!({ "receiver_id": s.child.id(), "token_id": "bv-1" })).gas(Gas::from_tgas(30)).transact().await?;
    assert!(r.is_failure(), "a transfer needs 1 yoctoNEAR");
    let info: Value = s.coll.view("vault_info").args_json(json!({ "token_id": "bv-1" })).await?.json()?;
    assert_eq!(info["owner_id"], s.friend.id().to_string(), "vault_info follows the NFT owner");
    Ok(())
}

#[tokio::test]
async fn transfer_call_disabled() -> anyhow::Result<()> {
    let s = setup(60).await?;
    let r = s.guardian.call(s.coll.id(), "nft_transfer_call")
        .args_json(json!({ "receiver_id": s.child.id(), "token_id": "bv-1", "msg": "" }))
        .deposit(NearToken::from_yoctonear(1)).gas(Gas::from_tgas(100)).transact().await?;
    assert!(fails_with(&r, "disabled: use nft_transfer"), "{:?}", r.failures());
    assert_eq!(owner(&s).await?, s.guardian.id().to_string());
    Ok(())
}

#[tokio::test]
async fn no_approval_or_resolver_methods() -> anyhow::Result<()> {
    let s = setup(60).await?;
    for method in ["nft_approve", "nft_revoke", "nft_revoke_all", "nft_is_approved", "nft_resolve_transfer"] {
        let r = s.guardian.call(s.coll.id(), method)
            .args_json(json!({ "token_id": "bv-1", "account_id": s.friend.id(), "approved_account_id": s.friend.id(), "owner_id": s.guardian.id(), "receiver_id": s.friend.id(), "msg": "" }))
            .deposit(NearToken::from_yoctonear(1)).gas(Gas::from_tgas(30)).transact().await?;
        assert!(fails_with(&r, "MethodNotFound"), "{method} must not exist: {:?}", r.failures());
    }
    Ok(())
}

#[tokio::test]
async fn cannot_send_nft_to_a_vault_or_collection() -> anyhow::Result<()> {
    let s = setup(5).await?;
    wait_unlock(&s).await?;
    let other_vault: AccountId = format!("bv-2.{}", s.coll.id()).parse()?;
    for bad in [s.coll.id(), &s.vault, &other_vault] {
        let r = transfer(&s, &s.guardian, bad, "bv-1").await?;
        assert!(fails_with(&r, "invalid receiver"), "receiver {bad} must be refused: {:?}", r.failures());
    }
    assert_eq!(owner(&s).await?, s.guardian.id().to_string());
    Ok(())
}

#[tokio::test]
async fn old_owner_cannot_withdraw_after_transfer() -> anyhow::Result<()> {
    let s = setup(5).await?;
    wait_unlock(&s).await?;
    assert!(transfer(&s, &s.guardian, s.friend.id(), "bv-1").await?.is_success());
    let r = withdraw(&s, &s.guardian, 1_000).await?;
    assert!(fails_with(&r, "only the owner"), "{:?}", r.failures());
    assert_eq!(usdc_balance(&s, &s.vault).await?, 5_000_000, "funds untouched");
    let ok: bool = withdraw(&s, &s.friend, 1_000).await?.json()?;
    assert!(ok, "the new owner can withdraw");
    assert_eq!(usdc_balance(&s, s.friend.id()).await?, 1_000);
    Ok(())
}

#[tokio::test]
async fn pending_blocks_transfer_claim_and_second_withdraw() -> anyhow::Result<()> {
    let s = setup(5).await?;
    wait_unlock(&s).await?;
    set_pending(&s, 1).await?;
    let r = transfer(&s, &s.guardian, s.child.id(), "bv-1").await?;
    assert!(fails_with(&r, BUSY), "transfer: {:?}", r.failures());
    let r = claim(&s, &s.child, "bv-1", 1).await?;
    assert!(fails_with(&r, BUSY), "claim: {:?}", r.failures());
    let r = withdraw(&s, &s.guardian, 1_000).await?;
    assert!(fails_with(&r, BUSY), "withdraw: {:?}", r.failures());
    assert_eq!(owner(&s).await?, s.guardian.id().to_string());
    set_pending(&s, 0).await?;
    assert!(claim(&s, &s.child, "bv-1", 1).await?.is_success(), "the claim works once settled");
    assert_eq!(owner(&s).await?, s.child.id().to_string());
    Ok(())
}

#[tokio::test]
async fn child_claim_rules() -> anyhow::Result<()> {
    let s = setup(10).await?;
    let r = claim(&s, &s.child, "bv-1", 1).await?;
    assert!(fails_with(&r, "locked"), "a claim before unlock must fail: {:?}", r.failures());
    wait_unlock(&s).await?;
    let cases: [(&Account, &str, u128, &str); 4] = [
        (&s.child, "bv-1", 0, "Requires attached deposit of exactly 1 yoctoNEAR"),
        (&s.child, "bv-9", 1, "no such vault"),
        (&s.friend, "bv-1", 1, "only the child"),
        (&s.guardian, "bv-1", 1, "only the child"),
    ];
    for (who, token, yocto, msg) in cases {
        let r = claim(&s, who, token, yocto).await?;
        assert!(fails_with(&r, msg), "expected '{msg}', got {:?}", r.failures());
    }
    assert!(claim(&s, &s.child, "bv-1", 1).await?.is_success());
    assert_eq!(owner(&s).await?, s.child.id().to_string());
    let r = claim(&s, &s.child, "bv-1", 1).await?;
    assert!(fails_with(&r, "the guardian no longer holds it"), "{:?}", r.failures());
    Ok(())
}

#[tokio::test]
async fn child_claim_updates_enumeration() -> anyhow::Result<()> {
    let s = setup(5).await?;
    wait_unlock(&s).await?;
    assert!(claim(&s, &s.child, "bv-1", 1).await?.is_success());
    let child_tokens: Vec<Value> = s.coll.view("nft_tokens_for_owner").args_json(json!({ "account_id": s.child.id() })).await?.json()?;
    assert_eq!(child_tokens.len(), 1);
    assert_eq!(child_tokens[0]["token_id"], "bv-1");
    let guardian_tokens: Vec<Value> = s.coll.view("nft_tokens_for_owner").args_json(json!({ "account_id": s.guardian.id() })).await?.json()?;
    assert!(guardian_tokens.is_empty());
    let supply: String = s.coll.view("nft_supply_for_owner").args_json(json!({ "account_id": s.child.id() })).await?.json()?;
    assert_eq!(supply, "1");
    let info: Value = s.coll.view("vault_info").args_json(json!({ "token_id": "bv-1" })).await?.json()?;
    assert_eq!(info["owner_id"], s.child.id().to_string());
    Ok(())
}

#[tokio::test]
async fn claim_vs_withdraw_ordering() -> anyhow::Result<()> {
    let s = setup(5).await?;
    wait_unlock(&s).await?;
    let ok: bool = withdraw(&s, &s.guardian, 1_000).await?.json()?;
    assert!(ok, "the guardian withdraws first");
    assert!(claim(&s, &s.child, "bv-1", 1).await?.is_success(), "the claim works after the withdrawal settles");
    let r = withdraw(&s, &s.guardian, 1_000).await?;
    assert!(fails_with(&r, "only the owner"), "{:?}", r.failures());
    let ok: bool = withdraw(&s, &s.child, 2_000).await?.json()?;
    assert!(ok, "the child withdraws after the claim");
    assert_eq!(usdc_balance(&s, s.guardian.id()).await?, 1_000);
    assert_eq!(usdc_balance(&s, s.child.id()).await?, 2_000);
    assert_eq!(usdc_balance(&s, &s.vault).await?, 5_000_000 - 3_000);
    Ok(())
}

#[tokio::test]
async fn events_conform_to_nep171() -> anyhow::Result<()> {
    let s = setup(5).await?;
    let mint = nep171_events(&s.mint_logs);
    assert_eq!(mint, vec![json!({ "standard": "nep171", "version": "1.0.0", "event": "nft_mint",
        "data": [{ "owner_id": s.guardian.id(), "token_ids": ["bv-1"] }] })]);
    let r = transfer(&s, &s.guardian, s.child.id(), "bv-1").await?;
    assert_eq!(nep171_events(&r.logs()), vec![json!({ "standard": "nep171", "version": "1.0.0", "event": "nft_transfer",
        "data": [{ "old_owner_id": s.guardian.id(), "new_owner_id": s.child.id(), "token_ids": ["bv-1"] }] })]);
    assert!(transfer(&s, &s.child, s.guardian.id(), "bv-1").await?.is_success());
    wait_unlock(&s).await?;
    let r = claim(&s, &s.child, "bv-1", 1).await?;
    assert_eq!(nep171_events(&r.logs()), vec![json!({ "standard": "nep171", "version": "1.0.0", "event": "nft_transfer",
        "data": [{ "old_owner_id": s.guardian.id(), "new_owner_id": s.child.id(), "token_ids": ["bv-1"], "memo": "child_claim" }] })]);
    Ok(())
}
