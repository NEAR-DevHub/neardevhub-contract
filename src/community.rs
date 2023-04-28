use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, AccountId};

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Github {
    pub repo: String,
    pub labels: Vec<String>,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Community {
    pub name: String,
    pub description: String,
    pub image: String,
    pub thumbnail: String,
    pub overview: String,
    pub events: String,
    pub admins: Vec<AccountId>,
    pub labels: Vec<String>,
    pub telegram: Vec<String>,
    pub github: Vec<Github>,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CommunityCard {
    pub slug: String,
    pub name: String,
    pub description: String,
    pub image: String,
}

impl Community {
    pub fn validate(&self) {
        if self.name.len() > 30 {
            panic!("Community name is limit to 30 characters");
        }
        if self.description.len() > 60 {
            panic!("Community description is limit to 60 characters");
        }
        if self.labels.len() == 0 {
            panic!("At least one primary label is required");
        }
    }

    pub fn set_default_admin(&mut self) {
        if self.admins.is_empty() {
            self.admins = vec![env::predecessor_account_id()];
        }
    }
}
