use super::{Like, PostStatus, SponsorshipToken};
use crate::str_serializers::*;
use crate::{AttestationId, CommentId, SolutionId, SponsorshipId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, Balance, Timestamp};
use std::collections::HashSet;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct SolutionV0 {
    // Common fields
    pub id: SolutionId,
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
pub struct SolutionV1 {
    pub name: String,
    pub description: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct SolutionV2 {
    pub name: String,
    pub description: String,
    pub sponsorship_token: SponsorshipToken,
    #[serde(with = "u128_dec_format")]
    pub amount: Balance,
    pub requested_sponsor: AccountId,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "solution_version")]
pub enum VersionedSolution {
    V0(SolutionV0),
    V1(SolutionV1),
    V2(SolutionV2),
}

impl VersionedSolution {
    pub fn latest_version(self) -> SolutionV2 {
        self.into()
    }
}

impl From<VersionedSolution> for SolutionV0 {
    fn from(vs: VersionedSolution) -> Self {
        match vs {
            VersionedSolution::V0(v0) => v0,
            _ => unimplemented!(),
        }
    }
}

impl From<VersionedSolution> for SolutionV1 {
    fn from(vs: VersionedSolution) -> Self {
        match vs {
            VersionedSolution::V1(v1) => v1,
            _ => unimplemented!(),
        }
    }
}

impl From<VersionedSolution> for SolutionV2 {
    fn from(vs: VersionedSolution) -> Self {
        match vs {
            VersionedSolution::V2(v2) => v2,

            VersionedSolution::V1(v1) => SolutionV2 {
                name: v1.name,
                description: v1.description,
                sponsorship_token: SponsorshipToken::USD,
                amount: 0,
                requested_sponsor: "".to_string(),
            },

            _ => unimplemented!(),
        }
    }
}

impl From<SolutionV0> for VersionedSolution {
    fn from(s: SolutionV0) -> Self {
        VersionedSolution::V0(s)
    }
}
