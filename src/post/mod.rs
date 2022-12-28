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
use std::cmp::Ordering;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
pub use submission::*;

pub type PostId = u64;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum PostType {
    Idea,
    Submission,
    Attestation,
    Sponsorship,
    Comment,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
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
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum VersionedPost {
    V0(Post),
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Post {
    pub author_id: AccountId,
    pub editor_id: AccountId,
    #[serde(with = "u64_dec_format")]
    pub timestamp: Timestamp,
    pub likes: HashSet<Like>,
    pub labels: HashSet<Label>,

    pub body: PostBody,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Label {
    pub name: String,
}

impl Hash for Label {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}

impl PartialEq for Label {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}

impl PartialOrd for Label {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl Eq for Label {}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum PostBody {
    Comment(VersionedComment),
    Idea(VersionedIdea),
    Submission(VersionedSubmission),
    Attestation(VersionedAttestation),
    Sponsorship(VersionedSponsorship),
}
