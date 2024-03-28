use near_sdk::{env, near, require, AccountId, Gas, NearToken, Promise};
use near_sdk::serde_json::json;

use devhub_common::social_db_contract;

const CODE: &[u8] = include_bytes!("../../community/target/near/devhub_community.wasm");
const INITIAL_BALANCE: NearToken = NearToken::from_near(4);
const PUBKEY_STR: &str = "ed25519:4deBAvg1S4MF7qe9GBDJwDCGLyyXtJa73JnMXwyG9vsB";

#[near(contract_state)]
#[derive(Default)]
pub struct Contract {}

#[near]
impl Contract {
    #[payable]
    pub fn create_community_account(&mut self, community: String) -> Promise {
        let parent_account: AccountId = env::current_account_id()
            .get_parent_account_id()
            .expect("Community factory should be deployed on a child account")
            .into();
        require!(
            env::predecessor_account_id() == parent_account,
            "Can only be called from parent contract"
        );
        require!(
            env::attached_deposit() >= INITIAL_BALANCE,
            "Require 4 NEAR to create community account"
        );

        let community_account_id: AccountId =
            format!("{}.{}", community, env::current_account_id()).parse().unwrap();

        let pubkey = PUBKEY_STR.parse().unwrap();
        Promise::new(community_account_id.clone())
            .create_account()
            .add_full_access_key(pubkey)
            .transfer(INITIAL_BALANCE)
            .deploy_contract(CODE.to_vec())
            .function_call(
                "new".to_string(),
                b"{}".to_vec(),
                NearToken::from_near(0),
                Gas::from_tgas(50),
            )
            .then(
                self.subscribe_to_community_accounts(community_account_id)
            )
    }

    pub fn subscribe_to_community_accounts(&mut self, community_account_id: AccountId) -> Promise {
        let community_factory_account = env::current_account_id();
        let discussions_account_id: AccountId =
            format!("discussions.{}", community_account_id.clone()).parse().unwrap();

        social_db_contract()
            .with_static_gas(env::prepaid_gas().saturating_div(3))
            .with_attached_deposit(env::attached_deposit())
            .set(json!({
                community_factory_account: {
                    "graph": {
                        "follow": {
                            community_account_id.clone(): "",
                            discussions_account_id.clone(): "",
                        }
                    },
                    "index": {
                        "graph": json!([
                            {
                                "key": "follow",
                                "value": {
                                    "type": "follow",
                                    "accountId": community_account_id
                                }
                            },
                            {
                                "key": "follow",
                                "value": {
                                    "type": "follow",
                                    "accountId": discussions_account_id
                                }
                            }
                        ]).to_string()
                    }                  
                },
            }))
    }
}
