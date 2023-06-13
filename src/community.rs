use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, AccountId};

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct WikiPage {
    name: String,
    content_markdown: String,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Community {
    pub handle: String,
    pub admins: Vec<AccountId>,
    pub name: String,
    pub description: String,
    pub bio_markdown: Option<String>,
    pub logo_url: String,
    pub banner_url: String,
    pub tag: String,
    pub github_handle: Option<String>,
    pub telegram_handle: Option<String>,
    pub twitter_handle: Option<String>,
    pub website_url: Option<String>,
    /// JSON string of github board configuration
    pub github: Option<String>,
    pub sponsorship: Option<bool>,
    pub wiki1: Option<WikiPage>,
    pub wiki2: Option<WikiPage>,
    pub overview_id: Option<u8>,
    pub events_id: Option<u8>,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CommunityCard {
    pub handle: String,
    pub name: String,
    pub description: String,
    pub logo_url: String,
    pub banner_url: String,
}

impl Community {
    pub fn validate(&self) {
        if self.name.len() > 30 {
            panic!("Community name is limit to 30 characters");
        }
        if self.description.len() > 60 {
            panic!("Community description is limit to 60 characters");
        }
        if self.handle.len() > 40 {
            panic!("Community handle is limit to 40 characters");
        }
        if self.tag.len() > 20 {
            panic!("Community tag is limit to 20 characters");
        }
    }

    pub fn set_default_admin(&mut self) {
        if self.admins.is_empty() {
            self.admins = vec![env::predecessor_account_id()];
        }
    }
}
