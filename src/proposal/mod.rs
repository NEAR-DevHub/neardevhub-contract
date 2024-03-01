pub mod repost;
pub mod timeline;

use std::collections::HashSet;

use self::timeline::TimelineStatus;

use crate::notify::get_text_mentions;
use crate::str_serializers::*;

use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, BlockHeight, NearSchema, Timestamp};

pub type ProposalId = u32;

type PostTag = String;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "proposal_version")]
#[borsh(crate = "near_sdk::borsh")]
pub enum VersionedProposal {
    V0(Proposal),
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct Proposal {
    pub id: ProposalId,
    pub author_id: AccountId,
    #[serde(
        serialize_with = "u64_dec_format::serialize",
        deserialize_with = "u64_dec_format::deserialize"
    )]
    pub social_db_post_block_height: BlockHeight,
    pub snapshot: ProposalSnapshot,
    // // Excludes the current snapshot itself.
    pub snapshot_history: Vec<ProposalSnapshot>,
}

impl From<VersionedProposal> for Proposal {
    fn from(vp: VersionedProposal) -> Self {
        match vp {
            VersionedProposal::V0(v0) => v0,
        }
    }
}

impl From<Proposal> for VersionedProposal {
    fn from(p: Proposal) -> Self {
        VersionedProposal::V0(p)
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct ProposalSnapshot {
    pub editor_id: AccountId,
    #[serde(
        serialize_with = "u64_dec_format::serialize",
        deserialize_with = "u64_dec_format::deserialize"
    )]
    pub timestamp: Timestamp,
    pub labels: HashSet<PostTag>,
    #[serde(flatten)]
    pub body: VersionedProposalBody,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct ProposalBodyV0 {
    pub name: String,
    pub category: String,
    pub summary: String,
    pub description: String,
    pub linked_proposals: Vec<ProposalId>,
    #[serde(
        serialize_with = "u32_dec_format::serialize",
        deserialize_with = "u32_dec_format::deserialize"
    )]
    pub requested_sponsorship_usd_amount: u32,
    pub requested_sponsorship_paid_in_currency: ProposalFundingCurrency,
    pub receiver_account: AccountId,
    pub requested_sponsor: AccountId,
    pub supervisor: Option<AccountId>,
    pub timeline: TimelineStatus,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "proposal_body_version")]
#[borsh(crate = "near_sdk::borsh")]
pub enum VersionedProposalBody {
    V0(ProposalBodyV0),
}

impl From<VersionedProposalBody> for ProposalBodyV0 {
    fn from(solution: VersionedProposalBody) -> Self {
        match solution {
            VersionedProposalBody::V0(v0) => v0,
        }
    }
}

impl From<ProposalBodyV0> for VersionedProposalBody {
    fn from(p: ProposalBodyV0) -> Self {
        VersionedProposalBody::V0(p)
    }
}

impl VersionedProposalBody {
    pub fn latest_version(self) -> ProposalBodyV0 {
        self.into()
    }
}

pub fn get_subscribers(proposal_body: &ProposalBodyV0) -> Vec<String> {
    let mut result = [
        get_text_mentions(proposal_body.description.as_str()),
        get_text_mentions(proposal_body.summary.as_str()),
    ]
    .concat();
    if let Some(supervisor) = proposal_body.supervisor.clone() {
        result.push(supervisor.to_string());
    }
    result.push(proposal_body.requested_sponsor.to_string());
    result
}

pub fn default_categories() -> Vec<String> {
    vec![
        String::from("DevDAO Operations"),
        String::from("Decentralized DevRel"),
        String::from("NEAR Campus"),
        String::from("Marketing"),
        String::from("Events"),
        String::from("Tooling & Infrastructures"),
        String::from("Other"),
    ]
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub enum ProposalFundingCurrency {
    NEAR,
    USDT,
    USDC,
    OTHER,
}