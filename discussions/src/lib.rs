use devhub_common::social_db_contract;
use near_sdk::{env, require, near, AccountId, NearToken, Promise};

#[near(contract_state)]
#[derive(Default)]
pub struct Contract {}

#[near]
impl Contract {
    #[init]
    pub fn new() -> Self {
        social_db_contract()
            .with_unused_gas_weight(1)
            .with_attached_deposit(NearToken::from_near(1))
            .grant_write_permission(
                Some(Contract::get_devhub_account()),
                None,
                vec![env::current_account_id().to_string()],
            )
            .as_return();
        Contract {}
    }

    pub fn destroy(&mut self) -> Promise {
        let devhub_account = Contract::get_devhub_account();
        require!(
            env::predecessor_account_id() == devhub_account,
            "Can only destroy community account from DevHub contract"
        );
        Promise::new(env::current_account_id()).delete_account(devhub_account)
    }

    /**
     * current_account_id = discussions.{{community}}.community.devhub.near
     * returns devhub.near
     */
    fn get_devhub_account() -> AccountId {
        env::current_account_id()
            .get_parent_account_id()
            .expect("Discussions contract should be deployed on a child account")
            .get_parent_account_id()
            .expect("Community contract should be deployed on a child account")
            .get_parent_account_id()
            .expect("Community factory should be deployed on a child account")
            .into()
    }
}
