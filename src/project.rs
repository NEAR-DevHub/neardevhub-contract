use crate::community::CommunityHandle;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ProjectMetadata {
    pub id: String,
    pub tag: String,
    pub name: String,
    pub description: String,
    pub owners: Vec<CommunityHandle>,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Project {
    pub id: String,
    pub tag: String,
    pub name: String,
    pub description: String,
    pub owners: Vec<CommunityHandle>,
    /// Project views indexed by their ids and serialized as JSON string
    pub views: String,
}

impl Project {
    pub fn validate(&self) {
        if self.name.len() < 3 || self.name.len() > 30 {
            panic!("Project name must contain from 3 to 30 characters");
        }
        if self.description.len() < 6 || self.description.len() > 60 {
            panic!("Project description must contain from 6 to 60 characters");
        }
        if self.tag.len() < 3 || self.tag.len() > 20 {
            panic!("Project tag must contain from 3 to 20 characters");
        }
    }
}
