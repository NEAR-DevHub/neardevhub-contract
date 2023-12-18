use super::Like;
use crate::str_serializers::*;
use crate::BlogId;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, Timestamp};
use std::collections::HashSet;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Blog {
    pub id: BlogId,
    pub author_id: AccountId,
    #[serde(with = "u64_dec_format")]
    pub timestamp: Timestamp,
    pub description: String,
    pub likes: HashSet<Like>,
    pub comments: Vec<BlogId>,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct BlogV1 {
    pub description: String,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "comment_version")]
pub enum VersionedBlog {
    V0(Blog),
    V1(BlogV1),
}

impl VersionedBlog {
    pub fn latest_version(self) -> BlogV1 {
        self.into()
    }
}

impl From<VersionedBlog> for Blog {
    fn from(vc: VersionedBlog) -> Self {
        match vc {
            VersionedBlog::V0(v0) => Blog {
                id: 0,
                author_id: v0.author_id,
                timestamp: v0.timestamp,
                description: v0.description,
                likes: v0.likes,
                comments: v0.comments,
            },
            VersionedBlog::V1(_) => unimplemented!(),
        }
    }
}

impl From<VersionedBlog> for BlogV1 {
    fn from(vc: VersionedBlog) -> Self {
        match vc {
            VersionedBlog::V1(v1) => v1,
            _ => unimplemented!(),
        }
    }
}

impl From<Blog> for VersionedBlog {
    fn from(c: Blog) -> Self {
        VersionedBlog::V0(c)
    }
}
