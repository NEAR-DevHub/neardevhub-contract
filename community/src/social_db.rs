use near_sdk::serde_json::Value;
use near_sdk::{ext_contract, AccountId, PublicKey, env};

#[ext_contract(ext_social_db)]
pub trait SocialDB {
    fn set(&mut self, data: Value);
    fn grant_write_permission(
        &mut self,
        predecessor_id: Option<AccountId>,
        public_key: Option<PublicKey>,
        keys: Vec<String>,
    );
}

pub fn social_db_contract() -> ext_social_db::SocialDBExt {
    let social_db: AccountId = if env::current_account_id().to_string().ends_with("testnet") {
        "v1.social08.testnet"
    } else {
        "social.near"
    }.parse().unwrap();
    ext_social_db::ext(social_db)
}