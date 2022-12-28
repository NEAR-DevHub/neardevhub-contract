mod attestation;
mod comment;
mod idea;
mod like;
mod sponsorship;
mod submission;

pub use attestation::*;
pub use comment::*;
pub use idea::*;
pub use like::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::BorshStorageKey;
pub use sponsorship::*;
pub use submission::*;

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
