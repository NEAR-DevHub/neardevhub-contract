use near_units::parse_near;
use serde_json::json;
use std::env;
use std::process::Command;
use workspaces::AccountId;

#[tokio::test]
async fn test_deploy_contract_self_upgrade() -> anyhow::Result<()> {
    const CONTRACT_ACCOUNT: &str = "devgovgigs.near";
    let worker = workspaces::sandbox().await?;

    let mainnet = workspaces::mainnet_archival().await?;
    let contract_id: AccountId = CONTRACT_ACCOUNT.parse()?;

    //To use block height if needed
    //const BLOCK_HEIGHT: BlockHeight = 97416242;

    let contract = worker
        .import_contract(&contract_id, &mainnet)
        .initial_balance(parse_near!("1000 N"))
        //.block_height(BLOCK_HEIGHT)
        .transact()
        .await?;

    println!("{}", format!("{:?}", contract));

    let outcome = contract
        .call("new")
        .args_json(json!({}))
        .transact() // note: we use the contract's keys here to sign the transaction
        .await?;

    assert!(outcome.is_success());
    assert!(format!("{:?}", outcome).contains("Migrated to version:"));

    println!("Init outcome: {:#?}", outcome);

    //compile the current code

    // non-working function from workspaces, needs more debugging.
    //let current_wasm = workspaces::compile_project("./").await?;

    //tempororary solution using cargo build
    //checkout comment here - https://github.com/near/neardevhub-contract/pull/46#issuecomment-1666830434
    let result = compile_project("./").await;
    assert!(result.is_ok(), "Compilation failed with error: {:?}", result.err().unwrap());

    let wasm = std::fs::read("./target/wasm32-unknown-unknown/release/devgovgigs.wasm")?;

    let self_upgrade = contract.call("unsafe_self_upgrade").args(wasm).max_gas().transact().await?;

    println!("Unsafe self upgrade: {:#?}", self_upgrade);

    //check if upgrade was success
    assert!(self_upgrade.is_success());
    if format!("{:?}", self_upgrade).contains("needs-migration") {
        let migrate =
            contract.call("unsafe_migrate").args_json(json!({})).max_gas().transact().await?;
        assert!(format!("{:?}", migrate).contains("Migration done."));
    } else {
        assert!(format!("{:?}", self_upgrade).contains("Migration done."));
    }

    Ok(())
}

pub async fn compile_project(project_path: &str) -> Result<(), std::io::Error> {
    // Set RUSTFLAGS environment variable
    env::set_var("RUSTFLAGS", "-C link-arg=-s");
    // Change current directory to parent directory
    env::set_current_dir(project_path)?;

    // Run the command
    let output = Command::new("cargo")
        .arg("build")
        .arg("--target")
        .arg("wasm32-unknown-unknown")
        .arg("--release")
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let text = String::from_utf8(output.stdout).unwrap();
        println!("Build successful: {}", text);
        Ok(())
    } else {
        let err = String::from_utf8(output.stderr).unwrap();
        println!("Build failed: {}", err);
        Err(std::io::Error::new(std::io::ErrorKind::Other, "Build failed"))
    }
}
