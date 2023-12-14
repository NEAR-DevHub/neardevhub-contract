mod social_db;

use crate::social_db::{ext_social_db, SOCIAL_DB};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, AccountId, NearToken};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(devhub_account: AccountId) -> Self {
        ext_social_db::ext(SOCIAL_DB.parse().unwrap())
            .with_unused_gas_weight(1)
            .with_attached_deposit(NearToken::from_near(1))
            .grant_write_permission(
                Some(devhub_account),
                None,
                vec![env::current_account_id().to_string()],
            );
        Contract {}
    }
}
