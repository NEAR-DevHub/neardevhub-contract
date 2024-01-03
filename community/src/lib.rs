mod social_db;

use crate::social_db::{ext_social_db, SOCIAL_DB};
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, require, AccountId, NearToken, Promise};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Default)]
#[borsh(crate = "near_sdk::borsh")]
pub struct Contract {}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        ext_social_db::ext(SOCIAL_DB.into())
            .with_unused_gas_weight(1)
            .with_attached_deposit(NearToken::from_near(1))
            .grant_write_permission(
                Some(Contract::get_devhub_account()),
                None,
                vec![env::current_account_id().to_string()],
            );
        Contract {}
    }

    pub fn destroy(&mut self) {
        let devhub_account = Contract::get_devhub_account();
        require!(
            env::predecessor_account_id() == devhub_account,
            "Can only destroy community account from DevHub contract"
        );
        Promise::new(env::current_account_id()).delete_account(devhub_account);
    }

    fn get_devhub_account() -> AccountId {
        env::current_account_id()
            .get_parent_account_id()
            .expect("Community contract should be deployed on a child account")
            .get_parent_account_id()
            .expect("Community factory should be deployed on a child account")
            .into()
    }
}
