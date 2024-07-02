pub mod repost;
pub mod timeline;

use std::collections::HashSet;

pub use self::timeline::TimelineStatus;

use crate::Contract;
use crate::proposal::{Proposal, ProposalId, VersionedProposalBody};
use crate::notify::get_text_mentions;
use crate::str_serializers::*;

use near_sdk::{env, require, near, AccountId, BlockHeight, Timestamp};

pub type RFPId = u32;

type PostTag = String;

#[near(serializers=[borsh, json])]
#[derive(Clone)]
#[serde(tag = "rfp_version")]
pub enum VersionedRFP {
    V0(RFP),
}

#[near(serializers=[borsh, json])]
#[derive(Clone)]
pub struct RFP {
    pub id: RFPId,
    pub author_id: AccountId,
    #[serde(
        serialize_with = "u64_dec_format::serialize",
        deserialize_with = "u64_dec_format::deserialize"
    )]
    pub social_db_post_block_height: BlockHeight,
    pub snapshot: RFPSnapshot,
    // Excludes the current snapshot itself.
    // Contains the block height when the RFP was added or edited.
    pub snapshot_history: Vec<BlockHeight>,
}

impl From<VersionedRFP> for RFP {
    fn from(vp: VersionedRFP) -> Self {
        match vp {
            VersionedRFP::V0(v0) => v0,
        }
    }
}

impl From<RFP> for VersionedRFP {
    fn from(p: RFP) -> Self {
        VersionedRFP::V0(p)
    }
}

#[near(serializers=[borsh, json])]
#[derive(Clone)]
pub struct RFPSnapshot {
    pub editor_id: AccountId,
    #[serde(
        serialize_with = "u64_dec_format::serialize",
        deserialize_with = "u64_dec_format::deserialize"
    )]
    pub timestamp: Timestamp,
    #[serde(
        serialize_with = "u64_dec_format::serialize",
        deserialize_with = "u64_dec_format::deserialize"
    )]
    pub block_height: BlockHeight,
    pub labels: HashSet<PostTag>,
    #[serde(flatten)]
    pub body: VersionedRFPBody,
    pub linked_proposals: HashSet<RFPId>,
}

#[near(serializers=[borsh, json])]
#[derive(Clone)]
pub struct RFPBodyV0 {
    pub name: String,
    pub summary: String,
    pub description: String,
    pub timeline: TimelineStatus,
    #[serde(
        serialize_with = "u64_dec_format::serialize",
        deserialize_with = "u64_dec_format::deserialize"
    )]
    pub submission_deadline: Timestamp,
}

#[near(serializers=[borsh, json])]
#[derive(Clone)]
#[serde(tag = "rfp_body_version")]
pub enum VersionedRFPBody {
    V0(RFPBodyV0),
}

impl From<VersionedRFPBody> for RFPBodyV0 {
    fn from(solution: VersionedRFPBody) -> Self {
        match solution {
            VersionedRFPBody::V0(v0) => v0,
        }
    }
}

impl From<RFPBodyV0> for VersionedRFPBody {
    fn from(p: RFPBodyV0) -> Self {
        VersionedRFPBody::V0(p)
    }
}

impl VersionedRFPBody {
    pub fn latest_version(self) -> RFPBodyV0 {
        self.into()
    }
}

pub fn get_subscribers(proposal_body: &RFPBodyV0) -> Vec<String> {
    let result = [
        get_text_mentions(proposal_body.description.as_str()),
        get_text_mentions(proposal_body.summary.as_str()),
    ]
    .concat();
    result
}

enum LinkedProposalChangeOperation {
    Add,
    Remove,
}


impl Contract {
    fn assert_can_link_unlink_rfp(&self, rfp_id: Option<RFPId>) {
        if let Some(rfp_id) = rfp_id {
            let rfp: RFP = self
                .rfps
                .get(rfp_id.into())
                .unwrap_or_else(|| panic!("RFP id {} not found", rfp_id))
                .into();
            require!(
                rfp.snapshot.body.latest_version().timeline.is_accepting_submissions() || self.is_allowed_to_write_rfps(env::predecessor_account_id()),
                format!("The RFP {} is not in the Accepting Submissions state, so you can't link or unlink to this RFP", rfp_id)
            );
        }
    }

    fn get_rfp_labels(&self, rfp_id: RFPId) -> HashSet<String> {
        let rfp: RFP = self
            .rfps
            .get(rfp_id.into())
            .unwrap_or_else(|| panic!("RFP id {} not found", rfp_id))
            .into();
        rfp.snapshot.labels
    }

    pub(crate) fn get_linked_proposals_in_rfp(&self, rfp_id: RFPId) -> HashSet<ProposalId> {
        let rfp: RFP = self.get_rfp(rfp_id).into();
        rfp.snapshot.linked_proposals
    }

    fn change_linked_proposal_in_rfp(&mut self, rfp_id: RFPId, proposal_id: ProposalId, operation: LinkedProposalChangeOperation) {
        let mut rfp: RFP = self.get_rfp(rfp_id).into();
        let mut linked_proposals = rfp.snapshot.linked_proposals.clone();
        match operation {
            LinkedProposalChangeOperation::Add => {
                linked_proposals.insert(proposal_id);
            }
            LinkedProposalChangeOperation::Remove => {
                linked_proposals.remove(&proposal_id);
            }
        }
        rfp.snapshot_history.push(rfp.snapshot.block_height);
        let new_snapshot = RFPSnapshot {
            editor_id: env::predecessor_account_id(),
            timestamp: env::block_timestamp(),
            block_height: env::block_height(),
            labels: rfp.snapshot.labels,
            body: rfp.snapshot.body,
            linked_proposals: linked_proposals,
        };
        rfp.snapshot = new_snapshot;
        self.rfps.replace(rfp_id.try_into().unwrap(), &rfp.clone().into());
    }

    fn add_linked_proposal_in_rfp(&mut self, rfp_id: RFPId, proposal_id: ProposalId) {
        self.change_linked_proposal_in_rfp(rfp_id, proposal_id, LinkedProposalChangeOperation::Add);
    }

    fn remove_linked_proposal_in_rfp(&mut self, rfp_id: RFPId, proposal_id: ProposalId) {
        self.change_linked_proposal_in_rfp(rfp_id, proposal_id, LinkedProposalChangeOperation::Remove);
    }

    pub(crate) fn update_and_check_rfp_link(
        &mut self,
        proposal_id: ProposalId,
        new_proposal_body: VersionedProposalBody,
        old_proposal_body: Option<VersionedProposalBody>,
        labels: HashSet<String>,
    ) -> HashSet<String> {
        let mut labels = labels;
        let new_body = new_proposal_body.clone().latest_version();
        let old_rfp_id =
            old_proposal_body.clone().map(|old| old.latest_version().linked_rfp).flatten();
        if new_body.linked_rfp != old_rfp_id {
            self.assert_can_link_unlink_rfp(new_body.linked_rfp);
            self.assert_can_link_unlink_rfp(old_rfp_id);
            if let Some(old_rfp_id) = old_rfp_id {
                self.remove_linked_proposal_in_rfp(old_rfp_id, proposal_id);
            }
            if let Some(new_rfp_id) = new_body.linked_rfp {
                self.add_linked_proposal_in_rfp(new_rfp_id, proposal_id);
            }
        }
        if let Some(new_rfp_id) = new_body.linked_rfp {
            labels = self.get_rfp_labels(new_rfp_id);
        }
        labels
    }

    pub(crate) fn edit_rfp_internal(
        &mut self,
        id: RFPId,
        body: VersionedRFPBody,
        labels: HashSet<String>,
    ) -> RFPId {
        let editor_id: AccountId = env::predecessor_account_id();
        require!(
            self.is_allowed_to_write_rfps(editor_id.clone()),
            "The account is not allowed to edit RFPs"
        );

        let mut rfp: RFP = self.get_rfp(id).into();

        let rfp_body = body.clone().latest_version();

        if rfp_body.timeline.is_proposal_selected() {
            let has_approved_proposal = self.get_rfp_linked_proposals(id)
                .into_iter()
                .filter_map(|proposal_id| self.proposals.get(proposal_id.into()))
                .any(|proposal|  Into::<Proposal>::into(proposal).snapshot.body.latest_version().timeline.was_approved());
            require!(has_approved_proposal, "Cannot change RFP status to Proposal Selected without an approved proposal linked to this RFP");
        }

        let old_snapshot = rfp.snapshot.clone();
        let old_labels_set = old_snapshot.labels.clone();
        let new_labels = labels;
        rfp.snapshot_history.push(rfp.snapshot.block_height);
        let new_snapshot = RFPSnapshot {
            editor_id: env::predecessor_account_id(),
            timestamp: env::block_timestamp(),
            block_height: env::block_height(),
            labels: new_labels.clone(),
            body: body,
            linked_proposals: old_snapshot.linked_proposals.clone(),
        };
        rfp.snapshot = new_snapshot;
        self.rfps.replace(id.try_into().unwrap(), &rfp.clone().into());

        // Update labels index.
        let new_labels_set = new_labels;

        if old_labels_set != new_labels_set {
            for proposal_id in self.get_rfp_linked_proposals(id) {
                self.update_proposal_labels(proposal_id, new_labels_set.clone());
            }
        }

        let labels_to_remove = &old_labels_set - &new_labels_set;
        let labels_to_add: HashSet<String> = &new_labels_set - &old_labels_set;
        for label_to_remove in labels_to_remove {
            let mut rfps = self.label_to_rfps.get(&label_to_remove).unwrap();
            rfps.remove(&id);
            self.label_to_rfps.insert(&label_to_remove, &rfps);
        }

        for label_to_add in labels_to_add {
            let mut rfps = self.label_to_rfps.get(&label_to_add).unwrap_or_default();
            rfps.insert(id);
            self.label_to_rfps.insert(&label_to_add, &rfps);
        }

        crate::notify::notify_rfp_subscribers(&rfp, self.get_moderators());
        id
    }
}