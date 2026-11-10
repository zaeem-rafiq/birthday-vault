//! The 7 checks for contracts/demo-hub, step S8.
use near_workspaces::network::Sandbox;
use near_workspaces::result::ExecutionFinalResult;
use near_workspaces::types::{AccessKey, Gas, KeyType, NearToken, PublicKey, SecretKey};
use near_workspaces::{Account, AccountId, Contract, Worker};
use serde_json::{json, Value};

const HOUR: u64 = 15_000_000_000;
const DAY: u64 = 40_000_000_000;

struct Setup {
    worker: Worker<Sandbox>,
    root: Account,
    hub: Contract,
    usdc: Contract,
    pool: Account,
    stranger: Account,
}

async fn account(root: &Account, name: &str, near: u128) -> anyhow::Result<Account> {
    Ok(root.create_subaccount(name).initial_balance(NearToken::from_near(near)).transact().await?.into_result()?)
}

async fn now(worker: &Worker<Sandbox>) -> anyhow::Result<u64> {
    Ok(worker.view_block().await?.timestamp())
}

async fn wait_until(worker: &Worker<Sandbox>, t: u64) -> anyhow::Result<()> {
    while now(worker).await? <= t {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    Ok(())
}

fn hub_wasm() -> anyhow::Result<Vec<u8>> {
    Ok(std::fs::read("../target/near/demo_hub/demo_hub.wasm")?)
}

/// A hub with test windows (15 s "hour", 40 s "day", limits 3 and 5), the tUSDC it feeds,
/// and one function-call key on the hub that may call only start_demo.
async fn setup() -> anyhow::Result<Setup> {
    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account()?;
    let hub = account(&root, "hub", 50).await?.deploy(&hub_wasm()?).await?.into_result()?;
    let usdc = account(&root, "usdc", 10).await?.deploy(&std::fs::read("../target/near/tusdc/tusdc.wasm")?).await?.into_result()?;
    assert!(usdc.call("new").args_json(json!({ "hub_id": hub.id() })).transact().await?.is_success());
    let r = hub.call("new")
        .args_json(json!({ "usdc": usdc.id(), "hour_ns": HOUR.to_string(), "day_ns": DAY.to_string(), "hourly_limit": 3, "daily_limit": 5 }))
        .transact().await?;
    assert!(r.is_success(), "init failed: {:?}", r.failures());
    let sk = SecretKey::from_seed(KeyType::ED25519, "pool-key-1");
    let r = hub.as_account().batch(hub.id())
        .add_key(sk.public_key(), AccessKey::function_call_access(hub.id(), &["start_demo"], Some(NearToken::from_near(1))))
        .transact().await?;
    assert!(r.is_success(), "adding the pool key failed: {:?}", r.failures());
    let pool = Account::from_secret_key(hub.id().clone(), sk, &worker);
    let stranger = account(&root, "stranger", 5).await?;
    Ok(Setup { worker, root, hub, usdc, pool, stranger })
}

fn key(seed: &str) -> PublicKey {
    SecretKey::from_seed(KeyType::ED25519, seed).public_key()
}

fn demo(n: u32) -> (PublicKey, PublicKey, PublicKey) {
    (key(&format!("guardian-{n}")), key(&format!("child-{n}")), key(&format!("buyer-{n}")))
}

fn implicit(pk: &PublicKey) -> String {
    pk.key_data().iter().map(|b| format!("{b:02x}")).collect()
}

async fn start(s: &Setup, signer: &Account, keys: &(PublicKey, PublicKey, PublicKey)) -> anyhow::Result<ExecutionFinalResult> {
    Ok(signer.call(s.hub.id(), "start_demo")
        .args_json(json!({ "guardian_pk": keys.0.to_string(), "child_pk": keys.1.to_string(), "buyer_pk": keys.2.to_string() }))
        .gas(Gas::from_tgas(100)).transact().await?)
}

fn fails_with(r: &ExecutionFinalResult, msg: &str) -> bool {
    r.is_failure() && format!("{:?}", r.failures()).contains(msg)
}

async fn info(s: &Setup) -> anyhow::Result<Value> {
    Ok(s.hub.view("hub_info").await?.json()?)
}

#[tokio::test]
async fn start_demo_creates_three_funded_accounts() -> anyhow::Result<()> {
    let s = setup().await?;
    let keys = demo(1);
    let r = start(&s, &s.pool, &keys).await?;
    assert!(r.is_success(), "{:?}", r.failures());
    let out: Value = r.json()?;
    assert_eq!(out, json!({ "guardian": implicit(&keys.0), "child": implicit(&keys.1), "buyer": implicit(&keys.2) }));
    for (pk, millinear) in [(&keys.0, 100), (&keys.1, 20), (&keys.2, 30)] {
        let id: AccountId = implicit(pk).parse()?;
        assert_eq!(s.worker.view_account(&id).await?.balance, NearToken::from_millinear(millinear), "{id}");
        let access = s.worker.view_access_keys(&id).await?;
        assert_eq!(access.len(), 1, "an implicit account has exactly its own key");
        assert_eq!(access[0].public_key.to_string(), pk.to_string());
    }
    let guardian: AccountId = implicit(&keys.0).parse()?;
    let bal: String = s.usdc.view("ft_balance_of").args_json(json!({ "account_id": guardian })).await?.json()?;
    assert_eq!(bal, "5000000", "the guardian gets play USDC");
    for pk in [&keys.1, &keys.2] {
        let reg: Value = s.usdc.view("storage_balance_of").args_json(json!({ "account_id": implicit(pk) })).await?.json()?;
        assert!(!reg.is_null(), "child and buyer are registered with tUSDC");
    }
    let started: bool = s.hub.view("has_started").args_json(json!({ "guardian_id": guardian })).await?.json()?;
    assert!(started);
    Ok(())
}

#[tokio::test]
async fn hub_keys_are_reusable_within_limits() -> anyhow::Result<()> {
    let s = setup().await?;
    for n in 1..=2 {
        let r = start(&s, &s.pool, &demo(n)).await?;
        assert!(r.is_success(), "start {n} with the same key: {:?}", r.failures());
    }
    let i = info(&s).await?;
    assert_eq!(i["starts_last_hour"], 2);
    assert_eq!(i["starts_last_day"], 2);
    assert_eq!(i["hourly_limit"], 3);
    assert_eq!(i["daily_limit"], 5);
    assert_eq!(i["usdc"], s.usdc.id().to_string());
    Ok(())
}

#[tokio::test]
async fn one_start_per_guardian_key() -> anyhow::Result<()> {
    let s = setup().await?;
    assert!(start(&s, &s.pool, &demo(1)).await?.is_success());
    let again = (demo(1).0, key("child-9"), key("buyer-9"));
    let r = start(&s, &s.pool, &again).await?;
    assert!(fails_with(&r, "this demo already started"), "{:?}", r.failures());
    assert_eq!(info(&s).await?["starts_last_day"], 1);
    Ok(())
}

#[tokio::test]
async fn hub_limits_hold() -> anyhow::Result<()> {
    let s = setup().await?;
    for n in 1..=3 {
        assert!(start(&s, &s.pool, &demo(n)).await?.is_success(), "start {n}");
    }
    let after_three = now(&s.worker).await?;
    let r = start(&s, &s.pool, &demo(4)).await?;
    assert!(fails_with(&r, "hourly limit reached, try again later"), "{:?}", r.failures());
    wait_until(&s.worker, after_three + HOUR + 1_000_000_000).await?;
    for n in 4..=5 {
        assert!(start(&s, &s.pool, &demo(n)).await?.is_success(), "start {n} after the hour");
    }
    let r = start(&s, &s.pool, &demo(6)).await?;
    assert!(fails_with(&r, "daily limit reached, try again tomorrow"), "{:?}", r.failures());
    wait_until(&s.worker, after_three + DAY + 1_000_000_000).await?;
    assert!(start(&s, &s.pool, &demo(6)).await?.is_success(), "the first three starts have left the day window");
    assert_eq!(info(&s).await?["starts_last_day"], 3);
    Ok(())
}

#[tokio::test]
async fn hub_auth() -> anyhow::Result<()> {
    let s = setup().await?;
    let r = start(&s, &s.stranger, &demo(1)).await?;
    assert!(fails_with(&r, "only the hub"), "{:?}", r.failures());
    let r = s.stranger.call(s.usdc.id(), "faucet").args_json(json!({ "account_id": s.stranger.id() })).transact().await?;
    assert!(fails_with(&r, "only the hub"), "a stranger must not use the faucet: {:?}", r.failures());
    let r = s.hub.call("new")
        .args_json(json!({ "usdc": s.usdc.id(), "hour_ns": "1", "day_ns": "1", "hourly_limit": 1, "daily_limit": 1 }))
        .transact().await?;
    assert!(r.is_failure(), "the hub must not initialize twice");
    let r = s.pool.call(s.usdc.id(), "faucet").args_json(json!({ "account_id": s.stranger.id() })).transact().await;
    assert!(r.map(|x| x.is_failure()).unwrap_or(true), "the pool key may call only start_demo on the hub");
    let r = s.pool.call(s.hub.id(), "has_started").args_json(json!({ "guardian_id": s.stranger.id() })).transact().await;
    assert!(r.map(|x| x.is_failure()).unwrap_or(true), "the pool key may call only start_demo");
    assert_eq!(info(&s).await?["starts_last_day"], 0);
    Ok(())
}

#[tokio::test]
async fn hub_rejects_bad_keys() -> anyhow::Result<()> {
    let s = setup().await?;
    let (g, c, b) = demo(1);
    let secp = SecretKey::from_seed(KeyType::SECP256K1, "secp-1").public_key();
    let cases = [
        ((secp.clone(), c.clone(), b.clone()), "only ed25519 keys"),
        ((g.clone(), c.clone(), secp.clone()), "only ed25519 keys"),
        ((g.clone(), g.clone(), b.clone()), "keys must differ"),
        ((g.clone(), c.clone(), c.clone()), "keys must differ"),
        ((g.clone(), c.clone(), g.clone()), "keys must differ"),
    ];
    for (keys, msg) in cases {
        let r = start(&s, &s.pool, &keys).await?;
        assert!(fails_with(&r, msg), "expected '{msg}', got {:?}", r.failures());
    }
    let started: bool = s.hub.view("has_started").args_json(json!({ "guardian_id": implicit(&g) })).await?.json()?;
    assert!(!started);
    assert_eq!(info(&s).await?["starts_last_day"], 0);
    assert!(start(&s, &s.pool, &(g, c, b)).await?.is_success(), "good keys still work");
    Ok(())
}

#[tokio::test]
async fn new_rejects_bad_limits() -> anyhow::Result<()> {
    let s = setup().await?;
    let hub2 = account(&s.root, "hub2", 10).await?.deploy(&hub_wasm()?).await?.into_result()?;
    let args = |hour: u64, day: u64, hourly: u32, daily: u32| json!({ "usdc": s.usdc.id(), "hour_ns": hour.to_string(), "day_ns": day.to_string(), "hourly_limit": hourly, "daily_limit": daily });
    let r = s.stranger.call(hub2.id(), "new").args_json(args(HOUR, DAY, 3, 5)).transact().await?;
    assert!(fails_with(&r, "only the hub"), "{:?}", r.failures());
    for (hour, day, hourly, daily) in [(0, DAY, 3, 5), (DAY, HOUR, 3, 5), (HOUR, DAY, 0, 5), (HOUR, DAY, 6, 5)] {
        let r = hub2.call("new").args_json(args(hour, day, hourly, daily)).transact().await?;
        assert!(fails_with(&r, "bad limits"), "({hour}, {day}, {hourly}, {daily}): {:?}", r.failures());
    }
    assert!(hub2.call("new").args_json(args(HOUR, DAY, 3, 5)).transact().await?.is_success());
    Ok(())
}
