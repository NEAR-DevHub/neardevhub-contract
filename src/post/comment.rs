use super::Like;
use crate::str_serializers::*;
use crate::CommentId;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, Timestamp};
use std::collections::HashSet;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CommentV0 {
    pub author_id: AccountId,
    #[serde(with = "u64_dec_format")]
    pub timestamp: Timestamp,
    pub description: String,
    pub likes: HashSet<Like>,
    pub comments: Vec<CommentId>,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Comment {
    pub id: CommentId,
    pub author_id: AccountId,
    #[serde(with = "u64_dec_format")]
    pub timestamp: Timestamp,
    pub description: String,
    pub likes: HashSet<Like>,
    pub comments: Vec<CommentId>,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CommentV2 {
    pub description: String,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum VersionedComment {
    V0(CommentV0),
    V1(Comment),
    V2(CommentV2),
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

impl From<Comment> for VersionedComment {
    fn from(c: Comment) -> Self {
        VersionedComment::V1(c)
    }
}
