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
pub struct OldCommunityV1 {
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
    pub telegram_handle: Vec<String>,
    pub twitter_handle: Option<String>,
    pub website_url: Option<String>,
    /// JSON string of github board configuration
    pub github: Option<String>,
    pub sponsorship: Option<bool>,
    pub wiki1: Option<WikiPage>,
    pub wiki2: Option<WikiPage>,
}

impl From<OldCommunityV1> for Community {
    fn from(old_community: OldCommunityV1) -> Self {
        let OldCommunityV1 {
            handle,
            admins,
            name,
            description,
            bio_markdown,
            logo_url,
            banner_url,
            tag,
            github_handle,
            telegram_handle,
            twitter_handle,
            website_url,
            github,
            sponsorship,
            wiki1,
            wiki2,
        } = old_community;
        Community {
            handle,
            admins,
            name,
            description,
            bio_markdown,
            logo_url,
            banner_url,
            tag,
            github_handle,
            twitter_handle,
            website_url,
            github,
            sponsorship,
            wiki1,
            wiki2,
            telegram_handle: telegram_handle.iter().cloned().collect(),
        }
    }
}
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct FeaturedCommunity {
    pub handle: String,
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
