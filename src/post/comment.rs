use super::Like;
use crate::str_serializers::*;
use crate::CommentId;
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, NearSchema, Timestamp};
use std::collections::HashSet;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct CommentV0 {
    pub author_id: AccountId,
    #[serde(
        serialize_with = "u64_dec_format::serialize",
        deserialize_with = "u64_dec_format::deserialize"
    )]
    pub timestamp: Timestamp,
    pub description: String,
    pub likes: HashSet<Like>,
    pub comments: Vec<CommentId>,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct Comment {
    pub id: CommentId,
    pub author_id: AccountId,
    #[serde(
        serialize_with = "u64_dec_format::serialize",
        deserialize_with = "u64_dec_format::deserialize"
    )]
    pub timestamp: Timestamp,
    pub description: String,
    pub likes: HashSet<Like>,
    pub comments: Vec<CommentId>,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct CommentV2 {
    pub description: String,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "comment_version")]
#[borsh(crate = "near_sdk::borsh")]
pub enum VersionedComment {
    V0(CommentV0),
    V1(Comment),
    V2(CommentV2),
}

impl VersionedComment {
    pub fn latest_version(self) -> CommentV2 {
        self.into()
    }
}

impl From<VersionedComment> for Comment {
    fn from(vc: VersionedComment) -> Self {
        match vc {
            VersionedComment::V0(v0) => Comment {
                id: 0,
                author_id: v0.author_id,
                timestamp: v0.timestamp,
                description: v0.description,
                likes: v0.likes,
                comments: v0.comments,
            },
            VersionedComment::V1(v1) => v1,
            VersionedComment::V2(_) => unimplemented!(),
        }
    }
}

impl From<VersionedComment> for CommentV2 {
    fn from(vc: VersionedComment) -> Self {
        match vc {
            VersionedComment::V2(v2) => v2,
            _ => unimplemented!(),
        }
    }
}

impl From<Comment> for VersionedComment {
    fn from(c: Comment) -> Self {
        VersionedComment::V1(c)
    }
}
