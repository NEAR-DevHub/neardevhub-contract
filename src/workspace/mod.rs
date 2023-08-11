use std::collections::HashSet;

use crate::community::CommunityHandle;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};

pub type WorkspaceId = usize;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct WorkspaceInputs {
    pub name: String,
    pub description: String,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct WorkspaceMetadata {
    pub id: WorkspaceId,
    pub name: String,
    pub description: String,
    pub owner_community_handles: HashSet<CommunityHandle>,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Workspace {
    pub metadata: WorkspaceMetadata,
    /// Configs for workspace views indexed by their ids and serialized as JSON string
    pub view_ids: HashSet<String>,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct WorkspaceViewInputsMetadata {
    pub workspace_id: WorkspaceId,
    pub kind: String,
    pub title: String,
    pub description: String,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct WorkspaceViewInputs {
    pub metadata: WorkspaceViewInputsMetadata,
    pub config: WorkspaceViewConfig,
}

pub type WorkspaceViewId = String;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct WorkspaceViewMetadata {
    pub id: WorkspaceViewId,
    pub workspace_id: WorkspaceId,
    pub kind: String,
    pub title: String,
    pub description: String,
}

/// Workspace view configuration serialized as JSON string
pub type WorkspaceViewConfig = String;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct WorkspaceView {
    pub metadata: WorkspaceViewMetadata,
    pub config: WorkspaceViewConfig,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct WorkspacePermissions {
    pub can_configure: bool,
}

impl Workspace {
    pub fn validate(&self) {
        if self.metadata.name.len() < 3 || self.metadata.name.len() > 30 {
            panic!("Workspace name must contain from 3 to 30 characters");
        }
        if self.metadata.description.len() < 6 || self.metadata.description.len() > 60 {
            panic!("Workspace description must contain from 6 to 60 characters");
        }
        if self.metadata.owner_community_handles.len() < 1 {
            panic!("Workspace must have at least one owner community");
        }
    }
}
