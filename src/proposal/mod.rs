pub mod repost;
pub mod timeline;

use std::collections::HashSet;

use self::timeline::TimelineStatus;

use crate::Contract;
use crate::str_serializers::*;
use crate::{notify::get_text_mentions, rfp::RFPId};

use near_sdk::{env, near, require, AccountId, BlockHeight, Timestamp};

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
pub struct ProposalBodyV1 {
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
    pub linked_rfp: Option<RFPId>,
}

#[near(serializers=[borsh, json])]
#[derive(Clone)]
#[serde(tag = "proposal_body_version")]
pub enum VersionedProposalBody {
    V0(ProposalBodyV0),
    V1(ProposalBodyV1),
}

impl From<ProposalBodyV0> for ProposalBodyV1 {
    fn from(v0: ProposalBodyV0) -> Self {
        ProposalBodyV1 {
            name: v0.name,
            category: v0.category,
            summary: v0.summary,
            description: v0.description,
            linked_proposals: v0.linked_proposals,
            requested_sponsorship_usd_amount: v0.requested_sponsorship_usd_amount,
            requested_sponsorship_paid_in_currency: v0.requested_sponsorship_paid_in_currency,
            receiver_account: v0.receiver_account,
            requested_sponsor: v0.requested_sponsor,
            supervisor: v0.supervisor,
            timeline: v0.timeline,
            linked_rfp: None,
        }
    }
}

impl From<VersionedProposalBody> for ProposalBodyV0 {
    fn from(solution: VersionedProposalBody) -> Self {
        match solution {
            VersionedProposalBody::V0(v0) => v0,
            _ => unimplemented!(),
        }
    }
}

impl From<VersionedProposalBody> for ProposalBodyV1 {
    fn from(solution: VersionedProposalBody) -> Self {
        match solution {
            VersionedProposalBody::V0(v0) => v0.into(),
            VersionedProposalBody::V1(v1) => v1,
        }
    }
}

impl From<ProposalBodyV0> for VersionedProposalBody {
    fn from(p: ProposalBodyV0) -> Self {
        VersionedProposalBody::V0(p)
    }
}

impl From<ProposalBodyV1> for VersionedProposalBody {
    fn from(p: ProposalBodyV1) -> Self {
        VersionedProposalBody::V1(p)
    }
}

impl VersionedProposalBody {
    pub fn latest_version(self) -> ProposalBodyV1 {
        self.into()
    }
}

pub fn get_subscribers(proposal_body: &ProposalBodyV1) -> Vec<String> {
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

#[near(serializers=[borsh, json])]
#[derive(Clone)]
pub enum ProposalFundingCurrency {
    NEAR,
    USDT,
    USDC,
    OTHER,
}

impl Contract {
    pub(crate) fn update_proposal_labels(&mut self, proposal_id: ProposalId, new_labels: HashSet<String>) -> ProposalId {
        let proposal: Proposal = self
            .proposals
            .get(proposal_id.into())
            .unwrap_or_else(|| panic!("Proposal id {} not found", proposal_id))
            .into();

        self.edit_proposal_internal(proposal_id, proposal.snapshot.body, new_labels)
    }

    pub(crate) fn edit_proposal_internal(
        &mut self,
        id: ProposalId,
        body: VersionedProposalBody,
        labels: HashSet<String>,
    ) -> ProposalId {
        require!(
            self.is_allowed_to_edit_proposal(id, Option::None),
            "The account is not allowed to edit this proposal"
        );
        let editor_id = env::predecessor_account_id();
        let mut proposal: Proposal = self
            .proposals
            .get(id.into())
            .unwrap_or_else(|| panic!("Proposal id {} not found", id))
            .into();

        let proposal_body = body.clone().latest_version();

        let old_body = proposal.snapshot.body.clone();
        let labels = self.update_and_check_rfp_link(id, body.clone(), Some(old_body.clone()), labels);

        let current_timeline = old_body.latest_version().timeline;

        require!(
            self.has_moderator(editor_id.clone())
                || editor_id.clone() == env::current_account_id()
                || current_timeline.is_draft()
                    && (proposal_body.timeline.is_empty_review()
                        || proposal_body.timeline.is_draft())
                || current_timeline.can_be_cancelled() && proposal_body.timeline.is_cancelled(),
            "This account is only allowed to change proposal status from DRAFT to REVIEW"
        );

        require!(
            proposal_body.timeline.is_draft() ||  proposal_body.timeline.is_review() || proposal_body.timeline.is_cancelled() || proposal_body.supervisor.is_some(),
            "You can't change the timeline of the proposal to this status without adding a supervisor"
        );

        require!(self.proposal_categories.contains(&proposal_body.category), "Unknown category");

        let old_snapshot = proposal.snapshot.clone();
        let old_labels_set = old_snapshot.labels.clone();
        let new_labels = labels;
        let new_snapshot = ProposalSnapshot {
            editor_id: editor_id.clone(),
            timestamp: env::block_timestamp(),
            labels: new_labels.clone(),
            body: body,
        };
        proposal.snapshot = new_snapshot;
        proposal.snapshot_history.push(old_snapshot);
        let proposal_author = proposal.author_id.clone();
        self.proposals.replace(id.try_into().unwrap(), &proposal.into());

        // Update labels index.
        let new_labels_set = new_labels;
        let labels_to_remove = &old_labels_set - &new_labels_set;
        let labels_to_add = &new_labels_set - &old_labels_set;
        require!(
            self.is_allowed_to_use_labels(
                Some(editor_id.clone()),
                labels_to_remove.iter().cloned().collect()
            ),
            "Not allowed to remove these labels"
        );
        require!(
            self.is_allowed_to_use_labels(
                Some(editor_id.clone()),
                labels_to_add.iter().cloned().collect()
            ),
            "Not allowed to add these labels"
        );

        for label_to_remove in labels_to_remove {
            let mut proposals = self.label_to_proposals.get(&label_to_remove).unwrap();
            proposals.remove(&id);
            self.label_to_proposals.insert(&label_to_remove, &proposals);
        }

        for label_to_add in labels_to_add {
            let mut proposals = self.label_to_proposals.get(&label_to_add).unwrap_or_default();
            proposals.insert(id);
            self.label_to_proposals.insert(&label_to_add, &proposals);
        }

        crate::notify::notify_edit_proposal(id, proposal_author);
        id
    }
}