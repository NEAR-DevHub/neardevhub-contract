use near_units::parse_near;
use near_workspaces::AccountId;
use serde_json::json;

const DEVHUB_CONTRACT: &str = "devgovgigs.near";
const NEAR_SOCIAL: &str = "social.near";

pub async fn init_contracts() -> anyhow::Result<near_workspaces::Contract> {
    let worker = near_workspaces::sandbox().await?;
    let mainnet = near_workspaces::mainnet_archival().await?;

    // NEAR social deployment
    let near_social_id: AccountId = NEAR_SOCIAL.parse()?;
    let near_social = worker
        .import_contract(&near_social_id, &mainnet)
        .initial_balance(parse_near!("10000 N"))
        .transact()
        .await?;
    near_social.call("new").transact().await?.into_result()?;

    // Devhub contract deployment
    let contract_id: AccountId = DEVHUB_CONTRACT.parse()?;
    let contract = worker
        .import_contract(&contract_id, &mainnet)
        .initial_balance(parse_near!("1000 N"))
        .transact()
        .await?;
    let outcome = contract.call("new").args_json(json!({})).transact().await?;
    assert!(outcome.is_success());
    assert!(format!("{:?}", outcome).contains("Migrated to version:"));

    Ok(contract)
}
