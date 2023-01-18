use near_sdk::ext_contract;
use near_sdk::serde_json::Value;

pub const SOCIAL_DB: &str = "social.near";

#[ext_contract(ext_social_db)]
pub trait SocialDB {
    fn set(&mut self, mut data: Value);
}
