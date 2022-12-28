use super::{Like, PostStatus};
use crate::{CommentId, IdeaId, SubmissionId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, Timestamp};
use std::collections::HashSet;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
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
    pub submissions: Vec<SubmissionId>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct IdeaV1 {
    pub name: String,
    pub description: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum VersionedIdea {
    V0(Idea),
    V1(IdeaV1),
}

impl From<VersionedIdea> for Idea {
    fn from(vi: VersionedIdea) -> Self {
        match vi {
            VersionedIdea::V0(v0) => v0,
            VersionedIdea::V1(_) => unimplemented!(),
        }
    }
}

impl From<Idea> for VersionedIdea {
    fn from(idea: Idea) -> Self {
        VersionedIdea::V0(idea)
    }
}
