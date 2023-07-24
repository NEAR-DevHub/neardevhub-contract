// macro allowing us to convert args into JSON bytes to be read by the contract.
use serde_json::json;

// Additional convenient imports that allows workspaces to function readily.

//Pointing to my devhub contract
//Compile an old verison of the contract.
const DEVHUB_WASM_FILEPATH: &str = "../res/old-devgovgigs.wasm";
//Compile your current version
const DEVHUB_WASM_FILEPATH_NEW: &str = "../res/devgovgigs.wasm";

//This includes launching the sandbox, loading your wasm file and deploying it to the sandbox environment.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let wasm = std::fs::read(DEVHUB_WASM_FILEPATH)?;
    let wasmnew = std::fs::read(DEVHUB_WASM_FILEPATH_NEW)?;
    let contract = worker.dev_deploy(&wasm).await?;

    let outcome = contract
        .call("new")
        .args_json(json!({}))
        .transact() // note: we use the contract's keys here to sign the transaction
        .await?;

    // outcome contains data like logs, receipts and transaction outcomes.
    println!("Init outcome: {:#?}", outcome);

    let self_upgrade =
        contract.call("unsafe_self_upgrade").args(wasmnew).max_gas().transact().await?;

    println!("Unsafe self upgrade: {:#?}", self_upgrade);

    let migrate = contract.call("unsafe_migrate").args_json(json!({})).max_gas().transact().await?;

    println!("Unsafe migrate: {:#?}", migrate);

    Ok(())
}
