use super::{Like, PostStatus};
use crate::{str_serializers::*, Balance, CommentId, SolutionId, SponsorshipId};
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, NearSchema, Timestamp};
use std::collections::HashSet;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub enum SponsorshipToken {
    NEAR,
    NEP141 { address: AccountId },
    USD,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct Sponsorship {
    // Common fields
    pub id: SponsorshipId,
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
    pub submission_id: SolutionId,
    pub sponsorship_token: SponsorshipToken,
    #[serde(
        serialize_with = "u128_dec_format::serialize",
        deserialize_with = "u128_dec_format::deserialize"
    )]
    pub amount: Balance,
    pub supervisor: AccountId,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct SponsorshipV1 {
    pub name: String,
    pub description: String,
    pub sponsorship_token: SponsorshipToken,
    #[serde(
        serialize_with = "u128_dec_format::serialize",
        deserialize_with = "u128_dec_format::deserialize"
    )]
    pub amount: Balance,
    pub supervisor: AccountId,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "sponsorship_version")]
#[borsh(crate = "near_sdk::borsh")]
pub enum VersionedSponsorship {
    V0(Sponsorship),
    V1(SponsorshipV1),
}

impl VersionedSponsorship {
    pub fn latest_version(self) -> SponsorshipV1 {
        self.into()
    }
}

impl From<VersionedSponsorship> for Sponsorship {
    fn from(vs: VersionedSponsorship) -> Self {
        match vs {
            VersionedSponsorship::V0(v0) => v0,
            VersionedSponsorship::V1(_) => unimplemented!(),
        }
    }
}

impl From<VersionedSponsorship> for SponsorshipV1 {
    fn from(vs: VersionedSponsorship) -> Self {
        match vs {
            VersionedSponsorship::V1(v1) => v1,
            _ => unimplemented!(),
        }
    }
}

impl From<Sponsorship> for VersionedSponsorship {
    fn from(s: Sponsorship) -> Self {
        VersionedSponsorship::V0(s)
    }
}
