//! Phase 0 proof: the local sandbox starts, and this prints its root account name.
//! source scripts/env.sh; cargo test --locked -p tests --test sandbox_root -- --nocapture

#[tokio::test]
async fn sandbox_root_account_name() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account()?;
    println!("Sandbox root account: {}", root.id());
    assert!(root.id().as_str().ends_with(".near"));
    Ok(())
}
