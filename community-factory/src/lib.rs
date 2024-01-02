use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, require, AccountId, Gas, NearToken, Promise};

const CODE: &[u8] = include_bytes!("../../res/devhub_community.wasm");
const INITIAL_BALANCE: NearToken = NearToken::from_near(2);
const PUBKEY_STR: &str = "ed25519:4deBAvg1S4MF7qe9GBDJwDCGLyyXtJa73JnMXwyG9vsB";
const DEVHUB: &near_sdk::AccountIdRef = near_sdk::AccountIdRef::new_or_panic("devhub.near");

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Default)]
#[borsh(crate = "near_sdk::borsh")]
pub struct Contract {}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn create_community_account(&mut self, community: String) -> Promise {
        let devhub_account: AccountId = DEVHUB.into();
        require!(
            env::predecessor_account_id() == devhub_account,
            "Can only be called from DevHub contract"
        );
        require!(
            env::attached_deposit() == INITIAL_BALANCE,
            "Require 2 NEAR to create community account"
        );

        let community_account_id: AccountId =
            format!("{}.{}", community, env::current_account_id()).parse().unwrap();

        let pubkey = PUBKEY_STR.parse().unwrap();
        Promise::new(community_account_id)
            .create_account()
            .add_full_access_key(pubkey)
            .transfer(INITIAL_BALANCE)
            .deploy_contract(CODE.to_vec())
            .function_call(
                "new".to_string(),
                b"{}".to_vec(),
                NearToken::from_near(0),
                Gas::from_tgas(20),
            )
    }
}
