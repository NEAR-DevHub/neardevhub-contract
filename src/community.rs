use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, AccountId};

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Wiki {
    name: String,
    content_markdown: String,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Community {
    pub name: String,
    pub description: String,
    pub image_url: String,
    pub thumbnail_url: String,
    pub wiki1: Option<Wiki>,
    pub wiki2: Option<Wiki>,
    pub admins: Vec<AccountId>,
    pub tag: String,
    pub telegram_handle: Option<String>,
    /// JSON string of github board configuration
    pub github: Option<String>,
    pub sponsorship: bool,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CommunityCard {
    pub handle: String,
    pub name: String,
    pub description: String,
    pub image_url: String,
}

impl Community {
    pub fn validate(&self) {
        if self.name.len() > 30 {
            panic!("Community name is limit to 30 characters");
        }
        if self.description.len() > 60 {
            panic!("Community description is limit to 60 characters");
        }
    }

    pub fn set_default_admin(&mut self) {
        if self.admins.is_empty() {
            self.admins = vec![env::predecessor_account_id()];
        }
    }
}
