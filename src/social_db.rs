use near_sdk::schemars::JsonSchema;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde_json::Value;
use near_sdk::{env, ext_contract, AccountId};

#[derive(Copy, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
#[schemars(crate = "near_sdk::schemars")]
pub struct SetReturnType {
    pub block_height: near_sdk::json_types::U64,
}

#[ext_contract(ext_social_db)]
pub trait SocialDB {
    fn set(&mut self, data: Value) -> SetReturnType;
}

pub fn social_db_contract() -> ext_social_db::SocialDBExt {
    let social_db: AccountId = if env::current_account_id().to_string().ends_with("testnet") {
        "v1.social08.testnet"
    } else {
        "social.near"
    }
    .parse()
    .unwrap();
    ext_social_db::ext(social_db)
}
