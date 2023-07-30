use std::collections::HashSet;

use crate::community::CommunityHandle;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};

pub type ProjectId = usize;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ProjectInputs {
    pub tag: String,
    pub name: String,
    pub description: String,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ProjectMetadata {
    pub id: ProjectId,
    pub tag: String,
    pub name: String,
    pub description: String,
    pub owner_community_handles: HashSet<CommunityHandle>,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Project {
    pub metadata: ProjectMetadata,
    /// Configs for project views indexed by their ids and serialized as JSON string
    pub view_configs: String,
}

impl Project {
    pub fn validate(&self) {
        if self.metadata.name.len() < 3 || self.metadata.name.len() > 30 {
            panic!("Project name must contain from 3 to 30 characters");
        }
        if self.metadata.description.len() < 6 || self.metadata.description.len() > 60 {
            panic!("Project description must contain from 6 to 60 characters");
        }
        if self.metadata.tag.len() < 3 || self.metadata.tag.len() > 20 {
            panic!("Project tag must contain from 3 to 20 characters");
        }
        if self.metadata.owner_community_handles.len() < 1 {
            panic!("Project must have at least one owner community");
        }
    }
}
