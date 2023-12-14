use near_sdk::serde_json::Value;
use near_sdk::{ext_contract, AccountId, PublicKey};

pub const SOCIAL_DB: &str = "social.near";

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
