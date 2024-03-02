use near_sdk::{AccountIdRef, NearToken};
use near_workspaces::network::Sandbox;
use near_workspaces::types::{AccessKey, KeyType, SecretKey};
use near_workspaces::{Account, Worker};

use serde_json::json;

const DEVHUB_CONTRACT_PREFIX: &str = "devhub";
const DEVHUB_CONTRACT: &AccountIdRef = AccountIdRef::new_or_panic("devhub.near");
const DEVHUB_COMMUNITY_CONTRACT: &AccountIdRef =
    AccountIdRef::new_or_panic("community.devhub.near");
const COMMUNITY_FACTORY_PREFIX: &str = "community";
const NEAR_SOCIAL: &AccountIdRef = AccountIdRef::new_or_panic("social.near");
const _TEST_NEAR_SOCIAL: &AccountIdRef = AccountIdRef::new_or_panic("v1.social08.testnet");
const TEST_SEED: &str = "testificate";
const DEVHUB_CONTRACT_PATH: &str = "./target/near/devhub.wasm";
const COMMUNITY_FACTORY_CONTRACT_PATH: &str =
    "./community-factory/target/near/devhub_community_factory.wasm";

#[allow(dead_code)]
pub async fn init_contracts_from_mainnet() -> anyhow::Result<near_workspaces::Contract> {
    let worker = near_workspaces::sandbox().await?;
    let mainnet = near_workspaces::mainnet_archival().await?;

    // NEAR social deployment
    let near_social = worker
        .import_contract(&NEAR_SOCIAL.to_owned(), &mainnet)
        .initial_balance(NearToken::from_near(10000))
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

    // Devhub contract deployment
    let devhub_contract = worker
        .import_contract(&DEVHUB_CONTRACT.to_owned(), &mainnet)
        .initial_balance(NearToken::from_near(1000))
        .transact()
        .await?;
    let outcome = devhub_contract.call("new").args_json(json!({})).transact().await?;
    assert!(outcome.is_success());
    assert!(format!("{:?}", outcome).contains("Migrated to version:"));

    // Devhub Community contract deployment
    worker
        .import_contract(&DEVHUB_COMMUNITY_CONTRACT.to_owned(), &mainnet)
        .initial_balance(NearToken::from_near(10))
        .transact()
        .await?;

    Ok(devhub_contract)
}

#[allow(dead_code)]
pub async fn init_contracts_from_res(
) -> anyhow::Result<(near_workspaces::Contract, Worker<Sandbox>, near_workspaces::Contract)> {
    let worker: Worker<Sandbox> = near_workspaces::sandbox().await?;
    let mainnet = near_workspaces::mainnet_archival().await?;

    // NEAR social deployment
    let near_social = worker
        .import_contract(&NEAR_SOCIAL.to_owned(), &mainnet)
        .initial_balance(NearToken::from_near(10000))
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

    let _test_near = worker.root_account()?;
    let tla_near = Account::from_secret_key("near".parse()?, sk.clone(), &worker);
    worker
        .patch(tla_near.id())
        .access_key(sk.public_key(), AccessKey::full_access())
        .transact()
        .await?;
    let contract_account = tla_near
        .create_subaccount(DEVHUB_CONTRACT_PREFIX)
        .initial_balance(NearToken::from_near(100))
        .transact()
        .await?
        .into_result()?;
    let contract = contract_account.deploy(&contract_wasm).await?.into_result()?;
    let _outcome = contract.call("new").args_json(json!({})).transact().await?;

    let community_factory_account = contract_account
        .create_subaccount(COMMUNITY_FACTORY_PREFIX)
        .initial_balance(NearToken::from_near(10))
        .transact()
        .await?
        .into_result()?;
    let community_factory_wasm = std::fs::read(COMMUNITY_FACTORY_CONTRACT_PATH)?;
    let _community_factory =
        community_factory_account.deploy(&community_factory_wasm).await?.into_result()?;
    Ok((contract, worker, near_social))
}
