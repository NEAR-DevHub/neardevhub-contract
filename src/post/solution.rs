use super::{Like, PostStatus, SponsorshipToken};
use crate::str_serializers::*;
use crate::{AttestationId, Balance, CommentId, SolutionId, SponsorshipId};
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, NearSchema, Timestamp};
use std::collections::HashSet;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct SolutionV0 {
    // Common fields
    pub id: SolutionId,
    pub name: String,
    pub description: String,
    pub author_id: AccountId,
    #[serde(
        serialize_with = "u64_dec_format::serialize",
        deserialize_with = "u64_dec_format::deserialize"
    )]
    pub timestamp: Timestamp,
    pub status: PostStatus,
    pub likes: HashSet<Like>,
    pub comments: Vec<CommentId>,

    // Specific fields
    #[serde(
        serialize_with = "u64_dec_format::serialize",
        deserialize_with = "u64_dec_format::deserialize"
    )]
    pub idea_id: u64,
    pub attestations: Vec<AttestationId>,
    pub sponsorships: Vec<SponsorshipId>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct SolutionV1 {
    pub name: String,
    pub description: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct SolutionV2 {
    pub name: String,
    pub description: String,
    pub requested_sponsor: Option<AccountId>,
    #[serde(
        serialize_with = "u128_dec_format::serialize",
        deserialize_with = "u128_dec_format::deserialize"
    )]
    pub requested_sponsorship_amount: Balance,
    pub requested_sponsorship_token: Option<SponsorshipToken>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "solution_version")]
#[borsh(crate = "near_sdk::borsh")]
pub enum VersionedSolution {
    V0(SolutionV0),
    V1(SolutionV1),
    V2(SolutionV2),
}

impl VersionedSolution {
    pub fn latest_version(self) -> SolutionV2 {
        self.into()
    }

    pub fn validate(&self) {
        match self {
            VersionedSolution::V2(solution) => {
                if solution.requested_sponsorship_amount > 0
                    && (solution.requested_sponsorship_token.is_none()
                        || solution.requested_sponsor.is_none())
                {
                    panic!(
                        "Solution that requires funding must specify sponsorship token and sponsor"
                    )
                }
            }

            _ => unimplemented!(),
        }
    }
}

impl From<VersionedSolution> for SolutionV0 {
    fn from(solution: VersionedSolution) -> Self {
        match solution {
            VersionedSolution::V0(v0) => v0,
            _ => unimplemented!(),
        }
    }
}

impl From<VersionedSolution> for SolutionV1 {
    fn from(solution: VersionedSolution) -> Self {
        match solution {
            VersionedSolution::V1(v1) => v1,
            _ => unimplemented!(),
        }
    }
}

impl From<SolutionV0> for VersionedSolution {
    fn from(solution: SolutionV0) -> Self {
        VersionedSolution::V0(solution)
    }
}

impl From<VersionedSolution> for SolutionV2 {
    fn from(solution: VersionedSolution) -> Self {
        match solution {
            VersionedSolution::V2(v2) => v2,

            VersionedSolution::V1(v1) => SolutionV2 {
                name: v1.name,
                description: v1.description,
                requested_sponsor: None,
                requested_sponsorship_amount: 0,
                requested_sponsorship_token: None,
            },

            _ => unimplemented!(),
        }
    }
}
