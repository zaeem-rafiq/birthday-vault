//! The 13 checks for contracts/collection, step S6: withdrawals.
use near_workspaces::network::Sandbox;
use near_workspaces::operations::Function;
use near_workspaces::result::ExecutionFinalResult;
use near_workspaces::types::{Gas, NearToken};
use near_workspaces::{Account, AccountId, Contract, Worker};
use serde_json::{json, Value};

const MINT_COST: u128 = 25_000_000_000_000_000_000_000;
const FAUCET: u128 = 5_000_000;
const BUSY: &str = "busy: a withdrawal is still settling";
const ONE_YOCTO: &str = "Requires attached deposit of exactly 1 yoctoNEAR";

struct Setup {
    worker: Worker<Sandbox>,
    root: Account,
    guardian: Account,
    child: Account,
    friend: Account,
    coll: Contract,
    usdc: Contract,
    hostile: Contract,
    vault: AccountId,
    unlock: u64,
}

async fn account(root: &Account, name: &str, near: u128) -> anyhow::Result<Account> {
    Ok(root
        .create_subaccount(name)
        .initial_balance(NearToken::from_near(near))
        .transact()
        .await?
        .into_result()?)
}

async fn now(worker: &Worker<Sandbox>) -> anyhow::Result<u64> {
    Ok(worker.view_block().await?.timestamp())
}

/// A Demo collection on tUSDC with one vault, bv-1. The guardian owns it, it holds
/// 5,000,000 tUSDC, and it unlocks about 5 seconds after the mint.
async fn setup() -> anyhow::Result<Setup> {
    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account()?;
    let guardian = account(&root, "guardian", 20).await?;
    let child = account(&root, "child", 5).await?;
    let friend = account(&root, "friend", 5).await?;
    let usdc = account(&root, "usdc", 10)
        .await?
        .deploy(&std::fs::read("../target/near/tusdc/tusdc.wasm")?)
        .await?
        .into_result()?;
    assert!(usdc
        .call("new")
        .args_json(json!({ "hub_id": root.id() }))
        .transact()
        .await?
        .is_success());
    let hostile = account(&root, "hostile", 10)
        .await?
        .deploy(&std::fs::read(
            "../target/near/hostile_token/hostile_token.wasm",
        )?)
        .await?
        .into_result()?;
    let coll = account(&root, "bv", 30)
        .await?
        .deploy(&std::fs::read("../target/near/collection/collection.wasm")?)
        .await?
        .into_result()?;
    let r = coll.call("new").args_json(json!({ "usdc": usdc.id(), "mode": "Demo", "min_lock_ns": "1000000000", "max_lock_ns": "3600000000000" })).transact().await?;
    assert!(r.is_success(), "init failed: {:?}", r.failures());
    let r = coll
        .call("set_vault_code")
        .args_borsh(std::fs::read("../target/near/vault/vault.wasm")?)
        .max_gas()
        .transact()
        .await?;
    assert!(r.is_success(), "set_vault_code failed: {:?}", r.failures());
    for who in [guardian.id(), child.id(), friend.id()] {
        assert!(root
            .call(usdc.id(), "register")
            .args_json(json!({ "account_id": who }))
            .transact()
            .await?
            .is_success());
    }
    let unlock = now(&worker).await? + 5_000_000_000;
    let minted: bool = guardian
        .call(coll.id(), "mint_vault")
        .args_json(
            json!({ "child_id": child.id(), "unlock_ns": unlock.to_string(), "title": null }),
        )
        .deposit(NearToken::from_yoctonear(MINT_COST))
        .gas(Gas::from_tgas(100))
        .transact()
        .await?
        .json()?;
    assert!(minted, "the mint must succeed");
    let vault: AccountId = format!("bv-1.{}", coll.id()).parse()?;
    assert!(root
        .call(usdc.id(), "faucet")
        .args_json(json!({ "account_id": vault }))
        .transact()
        .await?
        .is_success());
    Ok(Setup {
        worker,
        root,
        guardian,
        child,
        friend,
        coll,
        usdc,
        hostile,
        vault,
        unlock,
    })
}

async fn wait_unlock(s: &Setup) -> anyhow::Result<()> {
    while now(&s.worker).await? <= s.unlock {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn call_withdraw(
    s: &Setup,
    who: &Account,
    token: &str,
    ft: &AccountId,
    amount: u128,
    receiver: Option<&AccountId>,
    tgas: u64,
    yocto: u128,
) -> anyhow::Result<ExecutionFinalResult> {
    Ok(who.call(s.coll.id(), "withdraw")
        .args_json(json!({ "token_id": token, "ft_contract": ft, "amount": amount.to_string(), "receiver_id": receiver }))
        .deposit(NearToken::from_yoctonear(yocto)).gas(Gas::from_tgas(tgas)).transact().await?)
}

async fn withdraw(
    s: &Setup,
    ft: &AccountId,
    amount: u128,
    receiver: Option<&AccountId>,
) -> anyhow::Result<ExecutionFinalResult> {
    call_withdraw(s, &s.guardian, "bv-1", ft, amount, receiver, 100, 1).await
}

async fn withdraw_near(
    s: &Setup,
    amount: u128,
    receiver: Option<&AccountId>,
) -> anyhow::Result<ExecutionFinalResult> {
    Ok(s.guardian
        .call(s.coll.id(), "withdraw_near")
        .args_json(
            json!({ "token_id": "bv-1", "amount": amount.to_string(), "receiver_id": receiver }),
        )
        .deposit(NearToken::from_yoctonear(1))
        .gas(Gas::from_tgas(100))
        .transact()
        .await?)
}

async fn set_mode(s: &Setup, mode: &str) -> anyhow::Result<()> {
    let r = s
        .hostile
        .call("set_behavior")
        .args_json(json!({ "mode": mode, "fail_registration": false }))
        .transact()
        .await?;
    assert!(r.is_success());
    Ok(())
}

async fn pending(s: &Setup) -> anyhow::Result<u64> {
    let info: Value = s
        .coll
        .view("vault_info")
        .args_json(json!({ "token_id": "bv-1" }))
        .await?
        .json()?;
    Ok(info["pending"].as_u64().expect("pending is a number"))
}

async fn usdc_balance(s: &Setup, who: &AccountId) -> anyhow::Result<u128> {
    let b: String = s
        .usdc
        .view("ft_balance_of")
        .args_json(json!({ "account_id": who }))
        .await?
        .json()?;
    Ok(b.parse()?)
}

fn fails_with(r: &ExecutionFinalResult, msg: &str) -> bool {
    r.is_failure() && format!("{:?}", r.failures()).contains(&format!("panicked: {msg}"))
}

fn withdraw_events(r: &ExecutionFinalResult) -> Vec<Value> {
    r.logs()
        .iter()
        .filter_map(|l| l.strip_prefix("EVENT_JSON:"))
        .filter_map(|j| serde_json::from_str::<Value>(j).ok())
        .filter(|e| e["standard"] == "birthday_vault" && e["event"] == "withdraw")
        .collect()
}

#[tokio::test]
async fn withdraw_before_unlock_fails() -> anyhow::Result<()> {
    let s = setup().await?;
    let r = withdraw(&s, s.usdc.id(), 1_000, None).await?;
    assert!(
        fails_with(&r, "locked"),
        "a withdrawal before unlock must fail with locked: {:?}",
        r.failures()
    );
    assert_eq!(usdc_balance(&s, s.guardian.id()).await?, 0);
    assert_eq!(pending(&s).await?, 0);
    wait_unlock(&s).await?;
    let ok: bool = withdraw(&s, s.usdc.id(), 1_000, None).await?.json()?;
    assert!(ok, "the withdrawal after unlock must report success");
    assert_eq!(usdc_balance(&s, s.guardian.id()).await?, 1_000);
    assert_eq!(usdc_balance(&s, &s.vault).await?, FAUCET - 1_000);
    assert_eq!(pending(&s).await?, 0);
    Ok(())
}

#[tokio::test]
async fn withdraw_caller_and_input_rules() -> anyhow::Result<()> {
    let s = setup().await?;
    wait_unlock(&s).await?;
    let cases: [(&Account, &str, u128, u128, &str); 6] = [
        (&s.guardian, "bv-1", 1_000, 0, ONE_YOCTO),
        (&s.guardian, "bv-1", 1_000, 2, ONE_YOCTO),
        (&s.guardian, "bv-9", 1_000, 1, "no such vault"),
        (&s.friend, "bv-1", 1_000, 1, "only the owner"),
        (&s.child, "bv-1", 1_000, 1, "only the owner"),
        (&s.guardian, "bv-1", 0, 1, "amount must be positive"),
    ];
    for (who, token, amount, yocto, msg) in cases {
        let r = call_withdraw(&s, who, token, s.usdc.id(), amount, None, 100, yocto).await?;
        assert!(
            fails_with(&r, msg),
            "expected '{msg}', got {:?}",
            r.failures()
        );
    }
    assert_eq!(usdc_balance(&s, &s.vault).await?, FAUCET);
    assert_eq!(pending(&s).await?, 0);
    Ok(())
}

#[tokio::test]
async fn withdraw_receiver_rules() -> anyhow::Result<()> {
    let s = setup().await?;
    wait_unlock(&s).await?;
    for bad in [&s.vault, s.coll.id(), s.usdc.id()] {
        let r = withdraw(&s, s.usdc.id(), 1_000, Some(bad)).await?;
        assert!(
            fails_with(&r, "invalid receiver"),
            "receiver {bad} must be refused: {:?}",
            r.failures()
        );
    }
    assert_eq!(usdc_balance(&s, &s.vault).await?, FAUCET);
    let ok: bool = withdraw(&s, s.usdc.id(), 1_000, Some(s.friend.id()))
        .await?
        .json()?;
    assert!(ok);
    assert_eq!(usdc_balance(&s, s.friend.id()).await?, 1_000);
    let ok: bool = withdraw(&s, s.usdc.id(), 2_000, None).await?.json()?;
    assert!(ok);
    assert_eq!(
        usdc_balance(&s, s.guardian.id()).await?,
        2_000,
        "the receiver defaults to the owner"
    );
    assert_eq!(usdc_balance(&s, &s.vault).await?, FAUCET - 3_000);
    Ok(())
}

#[tokio::test]
async fn insufficient_gas_leaves_pending_zero() -> anyhow::Result<()> {
    let s = setup().await?;
    wait_unlock(&s).await?;
    let r = call_withdraw(&s, &s.guardian, "bv-1", s.usdc.id(), 1_000, None, 50, 1).await?;
    assert!(fails_with(&r, "not enough gas"), "{:?}", r.failures());
    assert_eq!(pending(&s).await?, 0);
    assert_eq!(usdc_balance(&s, &s.vault).await?, FAUCET);
    let ok: bool = call_withdraw(&s, &s.guardian, "bv-1", s.usdc.id(), 1_000, None, 90, 1)
        .await?
        .json()?;
    assert!(ok, "90 Tgas must be enough for a whole withdrawal");
    assert_eq!(pending(&s).await?, 0);
    Ok(())
}

#[tokio::test]
async fn failed_ft_transfer_does_not_wedge() -> anyhow::Result<()> {
    let s = setup().await?;
    wait_unlock(&s).await?;
    set_mode(&s, "fail").await?;
    let r = withdraw(&s, s.hostile.id(), 500, None).await?;
    assert!(
        r.is_success(),
        "a refused token transfer must not fail the whole call: {:?}",
        r.failures()
    );
    let events = withdraw_events(&r);
    assert_eq!(events.len(), 1, "exactly one withdraw event");
    assert_eq!(events[0]["data"]["ok"], false);
    let ok: bool = r.json()?;
    assert!(!ok, "on_withdraw must return false");
    assert_eq!(pending(&s).await?, 0);
    set_mode(&s, "").await?;
    let ok: bool = withdraw(&s, s.hostile.id(), 500, None).await?.json()?;
    assert!(ok, "the retry must work");
    let transfers: u32 = s.hostile.view("transfers").await?.json()?;
    assert_eq!(transfers, 1);
    assert_eq!(pending(&s).await?, 0);
    Ok(())
}

#[tokio::test]
async fn hostile_token_cannot_wedge_pending() -> anyhow::Result<()> {
    let s = setup().await?;
    wait_unlock(&s).await?;
    for (mode, expected) in [("huge", true), ("slow", true), ("fail", false)] {
        set_mode(&s, mode).await?;
        let r = withdraw(&s, s.hostile.id(), 500, None).await?;
        assert!(r.is_success(), "mode {mode}: {:?}", r.failures());
        let ok: bool = r.json()?;
        assert_eq!(ok, expected, "mode {mode}");
        assert_eq!(pending(&s).await?, 0, "mode {mode} left pending raised");
    }
    Ok(())
}

#[tokio::test]
async fn second_withdraw_while_pending_is_busy() -> anyhow::Result<()> {
    let s = setup().await?;
    wait_unlock(&s).await?;
    let call = || {
        Function::new("withdraw")
        .args_json(json!({ "token_id": "bv-1", "ft_contract": s.usdc.id(), "amount": "1000", "receiver_id": null }))
        .deposit(NearToken::from_yoctonear(1)).gas(Gas::from_tgas(100))
    };
    // Both calls run in one receipt. The first raises pending, so the second must refuse.
    let r = s
        .guardian
        .batch(s.coll.id())
        .call(call())
        .call(call())
        .transact()
        .await?;
    assert!(fails_with(&r, BUSY), "{:?}", r.failures());
    assert_eq!(pending(&s).await?, 0, "the failed receipt is rolled back");
    assert_eq!(usdc_balance(&s, &s.vault).await?, FAUCET);
    let ok: bool = withdraw(&s, s.usdc.id(), 1_000, None).await?.json()?;
    assert!(ok, "a withdrawal after settling must work");
    Ok(())
}

#[tokio::test]
async fn withdraw_emits_event() -> anyhow::Result<()> {
    let s = setup().await?;
    wait_unlock(&s).await?;
    let r = withdraw(&s, s.usdc.id(), 1_000, Some(s.friend.id())).await?;
    let events = withdraw_events(&r);
    assert_eq!(events.len(), 1, "exactly one withdraw event");
    assert_eq!(
        events[0],
        json!({ "standard": "birthday_vault", "version": "1.0.0", "event": "withdraw",
        "data": { "token_id": "bv-1", "ft_contract": s.usdc.id(), "receiver_id": s.friend.id(), "amount": "1000", "ok": true } })
    );
    Ok(())
}

#[tokio::test]
async fn on_withdraw_is_private() -> anyhow::Result<()> {
    let s = setup().await?;
    let r = s.guardian.call(s.coll.id(), "on_withdraw")
        .args_json(json!({ "token_id": "bv-1", "ft_contract": s.usdc.id(), "receiver_id": s.guardian.id(), "amount": "1" }))
        .transact().await?;
    assert!(
        fails_with(&r, "Method on_withdraw is private"),
        "{:?}",
        r.failures()
    );
    Ok(())
}

#[tokio::test]
async fn withdraw_near_rules() -> anyhow::Result<()> {
    let s = setup().await?;
    assert!(s
        .guardian
        .transfer_near(&s.vault, NearToken::from_near(1))
        .await?
        .is_success());
    let half = NearToken::from_millinear(500).as_yoctonear();
    let r = withdraw_near(&s, half, Some(s.friend.id())).await?;
    assert!(fails_with(&r, "locked"), "{:?}", r.failures());
    wait_unlock(&s).await?;
    for bad in [&s.vault, s.coll.id()] {
        let r = withdraw_near(&s, half, Some(bad)).await?;
        assert!(
            fails_with(&r, "invalid receiver"),
            "receiver {bad} must be refused: {:?}",
            r.failures()
        );
    }
    let before = s
        .worker
        .view_account(s.friend.id())
        .await?
        .balance
        .as_yoctonear();
    let r = withdraw_near(&s, half, Some(s.friend.id())).await?;
    let events = withdraw_events(&r);
    assert_eq!(events.len(), 1, "exactly one withdraw event");
    assert_eq!(events[0]["data"]["ft_contract"], Value::Null);
    let ok: bool = r.json()?;
    assert!(ok);
    let after = s
        .worker
        .view_account(s.friend.id())
        .await?
        .balance
        .as_yoctonear();
    assert_eq!(after - before, half, "the receiver gets exactly the amount");
    assert_eq!(pending(&s).await?, 0);
    Ok(())
}

#[tokio::test]
async fn withdraw_near_caller_and_input_rules() -> anyhow::Result<()> {
    let s = setup().await?;
    assert!(s
        .guardian
        .transfer_near(&s.vault, NearToken::from_near(1))
        .await?
        .is_success());
    wait_unlock(&s).await?;
    let half = NearToken::from_millinear(500).as_yoctonear();
    let cases: [(&Account, &str, u128, u64, u128, &str); 6] = [
        (&s.guardian, "bv-1", half, 100, 0, ONE_YOCTO),
        (&s.guardian, "bv-1", half, 50, 1, "not enough gas"),
        (&s.guardian, "bv-9", half, 100, 1, "no such vault"),
        (&s.friend, "bv-1", half, 100, 1, "only the owner"),
        (&s.child, "bv-1", half, 100, 1, "only the owner"),
        (&s.guardian, "bv-1", 0, 100, 1, "amount must be positive"),
    ];
    for (who, token, amount, tgas, yocto, msg) in cases {
        let r = who
            .call(s.coll.id(), "withdraw_near")
            .args_json(
                json!({ "token_id": token, "amount": amount.to_string(), "receiver_id": null }),
            )
            .deposit(NearToken::from_yoctonear(yocto))
            .gas(Gas::from_tgas(tgas))
            .transact()
            .await?;
        assert!(
            fails_with(&r, msg),
            "expected '{msg}', got {:?}",
            r.failures()
        );
    }
    let call = || {
        Function::new("withdraw_near")
            .args_json(
                json!({ "token_id": "bv-1", "amount": half.to_string(), "receiver_id": null }),
            )
            .deposit(NearToken::from_yoctonear(1))
            .gas(Gas::from_tgas(100))
    };
    // Both calls run in one receipt. The first raises pending, so the second must refuse.
    let r = s
        .guardian
        .batch(s.coll.id())
        .call(call())
        .call(call())
        .transact()
        .await?;
    assert!(fails_with(&r, BUSY), "{:?}", r.failures());
    assert_eq!(pending(&s).await?, 0);
    Ok(())
}

#[tokio::test]
async fn unexpected_assets() -> anyhow::Result<()> {
    let s = setup().await?;
    assert!(s
        .root
        .call(s.usdc.id(), "faucet")
        .args_json(json!({ "account_id": s.friend.id() }))
        .transact()
        .await?
        .is_success());
    let r = s
        .friend
        .call(s.usdc.id(), "ft_transfer_call")
        .args_json(json!({ "receiver_id": s.vault, "amount": "2000", "msg": "" }))
        .deposit(NearToken::from_yoctonear(1))
        .gas(Gas::from_tgas(100))
        .transact()
        .await?;
    assert!(r.is_success(), "{:?}", r.failures());
    assert_eq!(
        usdc_balance(&s, &s.vault).await?,
        FAUCET + 2_000,
        "the vault keeps tokens sent with ft_transfer_call"
    );
    assert_eq!(usdc_balance(&s, s.friend.id()).await?, FAUCET - 2_000);
    assert!(s
        .friend
        .transfer_near(&s.vault, NearToken::from_near(1))
        .await?
        .is_success());
    let r = withdraw(&s, s.usdc.id(), 2_000, None).await?;
    assert!(fails_with(&r, "locked"), "{:?}", r.failures());
    let r = withdraw_near(&s, NearToken::from_near(1).as_yoctonear(), None).await?;
    assert!(fails_with(&r, "locked"), "{:?}", r.failures());
    Ok(())
}

#[tokio::test]
async fn hostile_token_cannot_use_vault() -> anyhow::Result<()> {
    let s = setup().await?;
    wait_unlock(&s).await?;
    let r = s
        .usdc
        .as_account()
        .call(&s.vault, "ft_send")
        .args_json(json!({ "token": s.usdc.id(), "receiver": s.usdc.id(), "amount": "1000" }))
        .max_gas()
        .transact()
        .await?;
    assert!(r.is_failure(), "the token itself must not move vault funds");
    let r = s
        .hostile
        .as_account()
        .call(&s.vault, "near_send")
        .args_json(json!({ "receiver": s.hostile.id(), "amount": "1000" }))
        .max_gas()
        .transact()
        .await?;
    assert!(r.is_failure(), "a hostile token must not move vault NEAR");
    assert_eq!(usdc_balance(&s, &s.vault).await?, FAUCET);
    Ok(())
}
