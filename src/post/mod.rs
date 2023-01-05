mod attestation;
mod comment;
mod idea;
mod like;
mod sponsorship;
mod submission;

use crate::str_serializers::*;
pub use attestation::*;
pub use comment::*;
pub use idea::*;
pub use like::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, BorshStorageKey, Timestamp};
pub use sponsorship::*;
use std::collections::HashSet;
pub use submission::*;

pub type PostId = u64;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum PostType {
    Comment,
    Idea,
    Submission,
    Attestation,
    Sponsorship,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum PostStatus {
    Open,
    Closed { reason: String },
}

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    Ideas,
    Submissions,
    Attestations,
    Sponsorships,
    Comments,
    Posts,
    PostToParent,
    PostToChildren,
    LabelToPosts,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "post_version")]
pub enum VersionedPost {
    V0(Post),
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Post {
    pub id: PostId,
    pub author_id: AccountId,
    pub likes: HashSet<Like>,
    pub snapshot: PostSnapshot,
    // Excludes the current snapshot itself.
    pub snapshot_history: Vec<PostSnapshot>,
}

impl From<VersionedPost> for Post {
    fn from(vp: VersionedPost) -> Self {
        match vp {
            VersionedPost::V0(v0) => v0,
        }
    }
}

impl From<Post> for VersionedPost {
    fn from(p: Post) -> Self {
        VersionedPost::V0(p)
    }
}

type Label = String;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct PostSnapshot {
    pub editor_id: AccountId,
    #[serde(with = "u64_dec_format")]
    pub timestamp: Timestamp,
    pub labels: HashSet<Label>,
    #[serde(flatten)]
    pub body: PostBody,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "post_type")]
pub enum PostBody {
    Comment(VersionedComment),
    Idea(VersionedIdea),
    Submission(VersionedSubmission),
    Attestation(VersionedAttestation),
    Sponsorship(VersionedSponsorship),
}
