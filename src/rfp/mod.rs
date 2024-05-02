pub mod repost;
pub mod timeline;

use std::collections::HashSet;

pub use self::timeline::TimelineStatus;

use crate::notify::get_text_mentions;
use crate::str_serializers::*;

use near_sdk::{near, AccountId, BlockHeight, Timestamp};

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
    // // Excludes the current snapshot itself.
    pub snapshot_history: Vec<RFPSnapshot>,
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
    pub labels: HashSet<PostTag>,
    #[serde(flatten)]
    pub body: VersionedRFPBody,
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
