use super::{Like, PostStatus};
use crate::str_serializers::*;
use crate::{AttestationId, CommentId, SponsorshipId, SubmissionId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, Timestamp};
use std::collections::HashSet;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct SubmissionV0 {
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

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct SubmissionV1 {
    pub name: String,
    pub description: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct SubmissionV2 {
    pub name: String,
    pub description: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "submission_version")]
pub enum VersionedSubmission {
    V0(SubmissionV0),
    V1(SubmissionV1),
    V2(SubmissionV2),
}

impl VersionedSubmission {
    pub fn latest_version(self) -> SubmissionV2 {
        self.into()
    }
}

impl From<VersionedSubmission> for SubmissionV0 {
    fn from(vs: VersionedSubmission) -> Self {
        match vs {
            VersionedSubmission::V0(v0) => v0,
            _ => unimplemented!(),
        }
    }
}

impl From<VersionedSubmission> for SubmissionV1 {
    fn from(vs: VersionedSubmission) -> Self {
        match vs {
            VersionedSubmission::V1(v1) => v1,
            _ => unimplemented!(),
        }
    }
}

impl From<VersionedSubmission> for SubmissionV2 {
    fn from(vs: VersionedSubmission) -> Self {
        match vs {
            VersionedSubmission::V2(v2) => v2,
            _ => unimplemented!(),
        }
    }
}

impl From<SubmissionV0> for VersionedSubmission {
    fn from(s: SubmissionV0) -> Self {
        VersionedSubmission::V0(s)
    }
}
