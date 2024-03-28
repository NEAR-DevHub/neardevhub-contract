use devhub_common::social_db_contract;
use near_sdk;
use near_sdk::Gas;
use near_sdk::{env, near, require, AccountId, NearToken, Promise};

const CODE: &[u8] = include_bytes!("../../discussions/target/near/devhub_discussions.wasm");
const PUBKEY_STR: &str = "ed25519:4deBAvg1S4MF7qe9GBDJwDCGLyyXtJa73JnMXwyG9vsB";

#[near(contract_state)]
#[derive(Default)]
pub struct Contract {}

#[near]
impl Contract {
    #[payable]
    pub fn new(&mut self) -> Promise {
        social_db_contract()
            .with_unused_gas_weight(1)
            .with_attached_deposit(NearToken::from_near(1))
            .grant_write_permission(
                Some(Contract::get_devhub_account()),
                None,
                vec![env::current_account_id().to_string()],
            )
            .then(self.create_discussions_account())
    }

    pub fn destroy(&mut self) -> Promise {
        let devhub_account = Contract::get_devhub_account();
        require!(
            env::predecessor_account_id() == devhub_account,
            "Can only destroy community account from DevHub contract"
        );
        Promise::new(env::current_account_id()).delete_account(devhub_account)
    }

    fn get_devhub_account() -> AccountId {
        env::current_account_id()
            .get_parent_account_id()
            .expect("Community contract should be deployed on a child account")
            .get_parent_account_id()
            .expect("Community factory should be deployed on a child account")
            .into()
    }

    pub fn create_discussions_account(&mut self) -> Promise {
        let account_id: AccountId =
            format!("discussions.{}", env::current_account_id()).parse().unwrap();

        let pubkey = PUBKEY_STR.parse().unwrap();
        Promise::new(account_id)
            .create_account()
            .add_full_access_key(pubkey)
            .transfer(NearToken::from_near(2))
            .deploy_contract(CODE.to_vec())
            .function_call(
                "new".to_string(),
                b"{}".to_vec(),
                NearToken::from_near(0),
                Gas::from_tgas(20),
            )
    }
}
