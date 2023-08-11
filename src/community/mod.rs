use std::collections::HashSet;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, AccountId};

use crate::workspace::WorkspaceId;

pub type CommunityHandle = String;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct WikiPage {
    name: String,
    content_markdown: String,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CommunityFeatureFlags {
    pub github_integration: bool,
    pub workspaces: bool,
    pub wiki: bool,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CommunityInputs {
    pub handle: CommunityHandle,
    pub admins: Vec<AccountId>,
    pub name: String,
    pub description: String,
    pub bio_markdown: Option<String>,
    pub logo_url: String,
    pub banner_url: String,
    pub tag: String,
    pub github_handle: Option<String>,
    pub telegram_handle: Vec<String>,
    pub twitter_handle: Option<String>,
    pub website_url: Option<String>,
    /// JSON string of github board configuration
    pub github: Option<String>,
    pub wiki1: Option<WikiPage>,
    pub wiki2: Option<WikiPage>,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Community {
    pub handle: CommunityHandle,
    pub admins: Vec<AccountId>,
    pub name: String,
    pub description: String,
    pub bio_markdown: Option<String>,
    pub logo_url: String,
    pub banner_url: String,
    pub tag: String,
    pub github_handle: Option<String>,
    pub telegram_handle: Vec<String>,
    pub twitter_handle: Option<String>,
    pub website_url: Option<String>,
    /// JSON string of github board configuration
    pub github: Option<String>,
    pub wiki1: Option<WikiPage>,
    pub wiki2: Option<WikiPage>,
    pub workspace_ids: HashSet<WorkspaceId>,
    pub feature_flags: CommunityFeatureFlags,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct FeaturedCommunity {
    pub handle: CommunityHandle,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CommunityMetadata {
    pub handle: CommunityHandle,
    pub name: String,
    pub description: String,
    pub logo_url: String,
    pub banner_url: String,
}

impl Community {
    pub fn validate(&self) {
        if self.name.len() < 3 || self.name.len() > 30 {
            panic!("Community name must contain from 3 to 30 characters");
        }
        if self.description.len() < 6 || self.description.len() > 60 {
            panic!("Community description must contain from 6 to 60 characters");
        }
        if self.handle.len() < 3 || self.handle.len() > 40 {
            panic!("Community handle must contain from 3 to 40 characters");
        }
        if self.tag.len() < 3 || self.tag.len() > 20 {
            panic!("Community tag must contain from 3 to 20 characters");
        }
    }

    pub fn set_default_admin(&mut self) {
        if self.admins.is_empty() {
            self.admins = vec![env::predecessor_account_id()];
        }
    }
}
