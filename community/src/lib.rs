mod social_db;

use crate::social_db::{ext_social_db, SOCIAL_DB};
use near_sdk::borsh;
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, require, AccountId, NearToken, Promise};

const DEVHUB: &near_sdk::AccountIdRef = near_sdk::AccountIdRef::new_or_panic("devhub.near");

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct Contract {}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        ext_social_db::ext(SOCIAL_DB.into())
            .with_unused_gas_weight(1)
            .with_attached_deposit(NearToken::from_near(1))
            .grant_write_permission(
                Some(DEVHUB.into()),
                None,
                vec![env::current_account_id().to_string()],
            );
        Contract {}
    }

    pub fn destroy(&mut self) {
        let devhub_account: AccountId = DEVHUB.into();
        require!(
            env::predecessor_account_id() == devhub_account,
            "Can only destroy community account from DevHub contract"
        );
        Promise::new(env::current_account_id()).delete_account(devhub_account);
    }
}
