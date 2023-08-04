// macro allowing us to convert args into JSON bytes to be read by the contract.
use serde_json::json;
use workspaces::{Account, AccountId, BlockHeight, Contract, Worker};
use near_units::{parse_gas, parse_near};
use workspaces::network::Sandbox;


#[tokio::test]
async fn test2() -> anyhow::Result<()> {
    const CONTRACT_ACCOUNT: &str = "devgovgigs.near";
    let worker = workspaces::sandbox().await?;

    let mainnet = workspaces::mainnet_archival().await?;
    let contract_id: AccountId = CONTRACT_ACCOUNT.parse()?;

    let contract = worker
        .import_contract(&contract_id, &mainnet)
        .initial_balance(parse_near!("1000 N"))
        .transact()
        .await?;
    
    println!("{}",format!("{:?}", contract));

    let outcome = contract
        .call("new")
        .args_json(json!({}))
        .transact() // note: we use the contract's keys here to sign the transaction
        .await?;

    assert!(outcome.is_success());
    assert!(format!("{:?}", outcome)
    .contains("Migrated to version:"));

    println!("Init outcome: {:#?}", outcome);

    let current_wasm = workspaces::compile_project("./").await?;
    let self_upgrade = contract.call("unsafe_self_upgrade")
                       .args(current_wasm)
                       .max_gas()
                       .transact()
                       .await?;
    
    println!("Unsafe self upgrade: {:#?}", self_upgrade);
    assert!(self_upgrade.is_success());

    //assert!(contract1.is_success());

    Ok(())

    //do unsafe migrate
    // do check some empty returning function // check the migrations video. 
}