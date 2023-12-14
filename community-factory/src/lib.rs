use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde_json::json;
use near_sdk::{env, near_bindgen, AccountId, Gas, NearToken, Promise};

const INITIAL_BALANCE: NearToken = NearToken::from_near(2);
const PUBKEY_STR: &str = "ed25519:4deBAvg1S4MF7qe9GBDJwDCGLyyXtJa73JnMXwyG9vsB";
const CODE: &[u8] = include_bytes!("../../res/devhub_community.wasm");

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct Contract {}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn create_community_account(&mut self, community: String) -> Promise {
        let current_account_id = env::current_account_id().to_string();
        let first_dot = current_account_id.find('.').unwrap();
        let parent_account_id = &current_account_id[first_dot + 1..];
        if env::predecessor_account_id() != parent_account_id {
            panic!("Can only be called from DevHub contract");
        }
        if env::attached_deposit() != INITIAL_BALANCE {
            panic!("Require 2 NEAR to create community account");
        }

        let community_account_id: AccountId =
            format!("{}.{}", community, current_account_id).parse().unwrap();

        let pubkey = PUBKEY_STR.parse().unwrap();
        Promise::new(community_account_id)
            .create_account()
            .add_full_access_key(pubkey)
            .transfer(INITIAL_BALANCE)
            .deploy_contract(CODE.to_vec())
            .function_call(
                "new".to_string(),
                near_sdk::serde_json::to_vec(&json!({
                    "devhub_account": parent_account_id,
                }))
                .unwrap(),
                NearToken::from_near(0),
                Gas::from_tgas(25),
            )
    }
}
