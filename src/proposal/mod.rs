pub mod repost;
pub mod timeline;

use std::collections::HashSet;

use self::timeline::TimelineStatus;

use crate::notify::get_text_mentions;
use crate::str_serializers::*;

use near_sdk::{near, AccountId, BlockHeight, Timestamp};

pub type ProposalId = u32;

type PostTag = String;

#[near(serializers=[borsh, json])]
#[derive(Clone)]
#[serde(tag = "proposal_version")]
pub enum VersionedProposal {
    V0(Proposal),
}

#[near(serializers=[borsh, json])]
#[derive(Clone)]
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

#[near(serializers=[borsh, json])]
#[derive(Clone)]
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

#[near(serializers=[borsh, json])]
#[derive(Clone)]
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

#[near(serializers=[borsh, json])]
#[derive(Clone)]
#[serde(tag = "proposal_body_version")]
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

#[near(serializers=[borsh, json])]
#[derive(Clone)]
pub enum ProposalFundingCurrency {
    NEAR,
    USDT,
    USDC,
    OTHER,
}
