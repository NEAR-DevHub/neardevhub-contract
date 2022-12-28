use super::{Like, PostStatus};
use crate::str_serializers::*;
use crate::{AttestationId, CommentId, SponsorshipId, SubmissionId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, Timestamp};
use std::collections::HashSet;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Submission {
    // Common fields
    pub id: SubmissionId,
    pub name: String,
    pub description: String,
    pub author_id: AccountId,
    #[serde(with = "u64_dec_format")]
    pub timestamp: Timestamp,
    pub status: PostStatus,
    pub likes: HashSet<Like>,
    pub comments: Vec<CommentId>,

    // Specific fields
    #[serde(with = "u64_dec_format")]
    pub idea_id: u64,
    pub attestations: Vec<AttestationId>,
    pub sponsorships: Vec<SponsorshipId>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SubmissionV1 {
    pub name: String,
    pub description: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum VersionedSubmission {
    V0(Submission),
    V1(SubmissionV1),
}

impl From<VersionedSubmission> for Submission {
    fn from(vs: VersionedSubmission) -> Self {
        match vs {
            VersionedSubmission::V0(v0) => v0,
            VersionedSubmission::V1(_) => unimplemented!(),
        }
    }
}

impl From<Submission> for VersionedSubmission {
    fn from(s: Submission) -> Self {
        VersionedSubmission::V0(s)
    }
}
