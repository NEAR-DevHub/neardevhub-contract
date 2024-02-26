use super::{Like, PostStatus};
use crate::{CommentId, IdeaId, SolutionId};
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, NearSchema, Timestamp};
use std::collections::HashSet;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct Idea {
    // Common Fields
    pub id: IdeaId,
    pub name: String,
    pub description: String,
    pub author_id: AccountId,
    pub timestamp: Timestamp,
    pub status: PostStatus,
    pub likes: HashSet<Like>,
    pub comments: Vec<CommentId>,

    // Specific fields
    pub solutions: Vec<SolutionId>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct IdeaV1 {
    pub name: String,
    pub description: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "idea_version")]
#[borsh(crate = "near_sdk::borsh")]
pub enum VersionedIdea {
    V0(Idea),
    V1(IdeaV1),
}

impl VersionedIdea {
    pub fn latest_version(self) -> IdeaV1 {
        self.into()
    }
}

impl From<VersionedIdea> for Idea {
    fn from(vi: VersionedIdea) -> Self {
        match vi {
            VersionedIdea::V0(v0) => v0,
            VersionedIdea::V1(_) => unimplemented!(),
        }
    }
}

impl From<VersionedIdea> for IdeaV1 {
    fn from(vi: VersionedIdea) -> Self {
        match vi {
            VersionedIdea::V1(v1) => v1,
            _ => unimplemented!(),
        }
    }
}

impl From<Idea> for VersionedIdea {
    fn from(idea: Idea) -> Self {
        VersionedIdea::V0(idea)
    }
}
