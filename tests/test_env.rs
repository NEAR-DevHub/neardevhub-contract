use near_units::parse_near;
use near_workspaces::network::Sandbox;
use near_workspaces::types::{AccessKey, KeyType, SecretKey};
use near_workspaces::{Account, AccountId, Worker};

use serde_json::json;

const DEVHUB_CONTRACT_PREFIX: &str = "devhub";
const DEVHUB_CONTRACT: &str = "devgovgigs.near"; // current production contract
const NEW_DEVHUB_CONTRACT_PREFIX: &str = "devhub";
const COMMUNITY_FACTORY_PREFIX: &str = "community";
const NEAR_SOCIAL: &str = "social.near";
const TEST_SEED: &str = "testificate";
const DEVHUB_CONTRACT_PATH: &str = "./res/devgovgigs.wasm";
const COMMUNITY_FACTORY_CONTRACT_PATH: &str = "./res/devhub_community_factory.wasm";

pub async fn init_contracts_from_mainnet() -> anyhow::Result<near_workspaces::Contract> {
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

pub async fn init_contracts_from_res(
) -> anyhow::Result<(near_workspaces::Contract, Worker<Sandbox>)> {
    let worker: Worker<Sandbox> = near_workspaces::sandbox().await?;
    let mainnet = near_workspaces::mainnet_archival().await?;

    // NEAR social deployment
    let near_social_id: AccountId = NEAR_SOCIAL.parse()?;
    let near_social = worker
        .import_contract(&near_social_id, &mainnet)
        .initial_balance(parse_near!("10000 N"))
        .transact()
        .await?;
    near_social.call("new").transact().await?.into_result()?;
    near_social
        .call("set_status")
        .args_json(json!({
            "status": "Live"
        }))
        .transact()
        .await?
        .into_result()?;

    let contract_wasm = std::fs::read(DEVHUB_CONTRACT_PATH)?;
    let sk = SecretKey::from_seed(KeyType::ED25519, TEST_SEED);

    let test_near = worker.root_account()?;
    let tla_near = Account::from_secret_key("near".parse()?, sk.clone(), &worker);
    worker
        .patch(tla_near.id())
        .access_key(sk.public_key(), AccessKey::full_access())
        .transact()
        .await?;
    let contract_account = tla_near
        .create_subaccount(DEVHUB_CONTRACT_PREFIX)
        .initial_balance(parse_near!("100 N"))
        .transact()
        .await?
        .into_result()?;
    let contract = contract_account.deploy(&contract_wasm).await?.into_result()?;
    let outcome = contract.call("new").args_json(json!({})).transact().await?;

    let community_factory_account = contract_account
        .create_subaccount(COMMUNITY_FACTORY_PREFIX)
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .into_result()?;
    let community_factory_wasm = std::fs::read(COMMUNITY_FACTORY_CONTRACT_PATH)?;
    let community_factory =
        community_factory_account.deploy(&community_factory_wasm).await?.into_result()?;
    Ok((contract, worker))
}
