use near_sdk::ext_contract;
use near_sdk::serde_json::Value;

pub const SOCIAL_DB: &str = "social.near";

#[ext_contract(ext_social_db)]
pub trait SocialDB {
    // interface from: https://github.com/NearSocial/social-db/blob/39016e654739b0a3e8cb7ffaea4b03157c4aea6e/contract/src/api.rs#L135
    fn set(&mut self, #[allow(unused_mut)] mut data: Value);
}
