use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, AccountId};

pub type CommunityHandle = String;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CommunityInputs {
    pub handle: CommunityHandle,
    pub name: String,
    pub tag: String,
    pub description: String,
    pub logo_url: String,
    pub banner_url: String,
    pub bio_markdown: Option<String>,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CommunityMetadata {
    pub admins: Vec<AccountId>,
    pub handle: CommunityHandle,
    pub name: String,
    pub tag: String,
    pub description: String,
    pub logo_url: String,
    pub banner_url: String,
    pub bio_markdown: Option<String>,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CommunityFeatureFlags {
    pub telegram: bool,
    pub github: bool,
    pub board: bool,
    pub wiki: bool,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct WikiPage {
    name: String,
    content_markdown: String,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Community {
    pub admins: Vec<AccountId>,
    pub handle: CommunityHandle,
    pub name: String,
    pub tag: String,
    pub description: String,
    pub logo_url: String,
    pub banner_url: String,
    pub bio_markdown: Option<String>,
    pub github_handle: Option<String>,
    pub telegram_handle: Vec<String>,
    pub twitter_handle: Option<String>,
    pub website_url: Option<String>,
    /// JSON string of github board configuration
    pub github: Option<String>,
    /// JSON string of kanban board configuration
    pub board: Option<String>,
    pub wiki1: Option<WikiPage>,
    pub wiki2: Option<WikiPage>,
    pub features: CommunityFeatureFlags,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct FeaturedCommunity {
    pub handle: CommunityHandle,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CommunityPermissions {
    pub can_configure: bool,
    pub can_delete: bool,
}

impl Community {
    pub fn validate(&self) {
        if self.handle.len() < 3 || self.handle.len() > 40 {
            panic!("Community handle must contain from 3 to 40 characters");
        }

        if self.name.len() < 3 || self.name.len() > 30 {
            panic!("Community name must contain from 3 to 30 characters");
        }

        if self.tag.len() < 3 || self.tag.len() > 30 {
            panic!("Community tag must contain from 3 to 30 characters");
        }

        if self.description.len() < 6 || self.description.len() > 60 {
            panic!("Community description must contain from 6 to 60 characters");
        }

        if self.bio_markdown.is_some()
            && (self.bio_markdown.as_ref().unwrap().len() < 3
                || self.bio_markdown.as_ref().unwrap().len() > 200)
        {
            panic!("Community bio must contain from 3 to 200 characters");
        }
    }

    pub fn set_default_admin(&mut self) {
        if self.admins.is_empty() {
            self.admins = vec![env::predecessor_account_id()];
        }
    }
}
