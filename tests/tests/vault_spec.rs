//! The 10 checks for contracts/vault. Build the contract first, then run:
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
    token: Contract,
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
    let token_wasm = std::fs::read("../target/near/test_token/test_token.wasm")?;
    let token_account = root.create_subaccount("token").initial_balance(NearToken::from_near(10)).transact().await?.into_result()?;
    let token = token_account.deploy(&token_wasm).await?.into_result()?;
    let stranger = root.create_subaccount("stranger").initial_balance(NearToken::from_near(10)).transact().await?.into_result()?;
    let receiver = root.create_subaccount("receiver").initial_balance(NearToken::from_near(1)).transact().await?.into_result()?;
    Ok(Setup { _worker: worker, parent, vault_account, vault, token, stranger, receiver })
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
    let args = json!({ "token": s.token.id(), "receiver": s.stranger.id(), "amount": "1" });
    let r = s.stranger.call(s.vault.id(), "ft_send").args_json(args).max_gas().transact().await?;
    assert!(r.is_failure(), "a stranger must not be able to call ft_send");
    let (_, _, _, _, calls): (String, String, String, String, u32) = s.token.view("last_call").await?.json()?;
    assert_eq!(calls, 0, "a refused call must not reach the token");
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

#[tokio::test]
async fn parent_ft_send_calls_ft_transfer_with_one_yocto() -> anyhow::Result<()> {
    let s = setup().await?;
    let args = json!({ "token": s.token.id(), "receiver": s.receiver.id(), "amount": "12345" });
    let r = s.parent.call(s.vault.id(), "ft_send").args_json(args).max_gas().transact().await?;
    assert!(r.is_success(), "ft_send failed: {:?}", r.failures());
    let (predecessor, receiver, amount, deposit, calls): (String, String, String, String, u32) =
        s.token.view("last_call").await?.json()?;
    assert_eq!(calls, 1, "ft_transfer must run exactly once");
    assert_eq!(predecessor, s.vault.id().to_string(), "the vault must be the caller");
    assert_eq!(receiver, s.receiver.id().to_string());
    assert_eq!(amount, "12345");
    assert_eq!(deposit, "1", "ft_transfer needs exactly 1 yoctoNEAR");
    Ok(())
}

fn wasm_imports(wasm: &[u8]) -> Vec<String> {
    fn leb(d: &[u8], i: &mut usize) -> usize {
        let (mut v, mut s) = (0usize, 0);
        loop {
            let b = d[*i];
            *i += 1;
            v |= ((b & 0x7f) as usize) << s;
            s += 7;
            if b & 0x80 == 0 {
                return v;
            }
        }
    }
    let mut names = Vec::new();
    let mut i = 8;
    while i < wasm.len() {
        let id = wasm[i];
        i += 1;
        let size = leb(wasm, &mut i);
        let end = i + size;
        if id == 2 {
            let mut j = i;
            for _ in 0..leb(wasm, &mut j) {
                let m = leb(wasm, &mut j);
                j += m;
                let n = leb(wasm, &mut j);
                names.push(String::from_utf8_lossy(&wasm[j..j + n]).to_string());
                j += n;
                let kind = wasm[j];
                j += 1;
                assert_eq!(kind, 0, "the vault must import only functions");
                leb(wasm, &mut j);
            }
        }
        i = end;
    }
    names
}

#[tokio::test]
async fn the_vault_wasm_cannot_add_keys_or_deploy() -> anyhow::Result<()> {
    let wasm = std::fs::read("../target/near/vault/vault.wasm")?;
    let imports = wasm_imports(&wasm);
    assert!(imports.iter().any(|n| n == "promise_batch_action_transfer"), "unexpected import list: {imports:?}");
    for bad in ["add_key", "delete_key", "deploy_contract", "deploy_global", "use_global", "delete_account", "stake"] {
        assert!(!imports.iter().any(|n| n.contains(bad)), "the vault imports a forbidden action: {bad}");
    }
    Ok(())
}

#[tokio::test]
async fn ft_send_rejects_a_bad_amount() -> anyhow::Result<()> {
    let s = setup().await?;
    for bad in ["abc", "", "340282366920938463463374607431768211456"] {
        let args = json!({ "token": s.token.id(), "receiver": s.receiver.id(), "amount": bad });
        let r = s.parent.call(s.vault.id(), "ft_send").args_json(args).max_gas().transact().await?;
        assert!(r.is_failure(), "the vault must refuse the amount {bad:?}");
    }
    let (_, _, _, _, calls): (String, String, String, String, u32) = s.token.view("last_call").await?.json()?;
    assert_eq!(calls, 0, "a bad amount must not reach the token");
    Ok(())
}
