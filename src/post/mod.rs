mod attestation;
mod comment;
mod github;
mod idea;
mod like;
mod solution;
mod sponsorship;

use crate::str_serializers::*;
pub use attestation::*;
pub use comment::*;
pub use idea::*;
pub use like::*;
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, BorshStorageKey, CryptoHash, NearSchema, Timestamp};
pub use solution::*;
pub use sponsorship::*;
use std::collections::HashSet;

pub type Balance = u128;
pub type PostId = u64;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub enum PostType {
    Comment,
    Idea,
    Solution,
    Attestation,
    Sponsorship,
    Github,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub enum PostStatus {
    Open,
    Closed { reason: String },
}

#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "near_sdk::borsh")]
pub enum StorageKey {
    Ideas,
    Solutions,
    Attestations,
    Sponsorships,
    Comments,
    Posts,
    PostToParent,
    PostToChildren,
    /// Deprecated due to damaged storage state.
    LabelToPosts,
    LabelToPostsV2,
    AuthorToAuthorPosts,
    AuthorPosts(CryptoHash),
    Communities,
    AddOns,
    Proposals,
    LabelToProposals,
    AuthorProposals,
    RFPs,
    LabelToRFPs,
    RFPLinkedProposals,
    LabelInfo,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "post_version")]
#[borsh(crate = "near_sdk::borsh")]
pub enum VersionedPost {
    V0(Post),
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct Post {
    pub id: PostId,
    pub author_id: AccountId,
    pub likes: HashSet<Like>,
    pub snapshot: PostSnapshot,
    // Excludes the current snapshot itself.
    pub snapshot_history: Vec<PostSnapshot>,
}

type PostTag = String;

impl From<VersionedPost> for Post {
    fn from(vp: VersionedPost) -> Self {
        match vp {
            VersionedPost::V0(v0) => v0,
        }
    }
}

impl From<Post> for VersionedPost {
    fn from(p: Post) -> Self {
        VersionedPost::V0(p)
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct PostSnapshot {
    pub editor_id: AccountId,
    #[serde(
        serialize_with = "u64_dec_format::serialize",
        deserialize_with = "u64_dec_format::deserialize"
    )]
    pub timestamp: Timestamp,
    pub labels: HashSet<PostTag>,
    #[serde(flatten)]
    pub body: PostBody,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "post_type")]
#[borsh(crate = "near_sdk::borsh")]
pub enum PostBody {
    Comment(VersionedComment),
    Idea(VersionedIdea),
    Solution(VersionedSolution),
    Attestation(VersionedAttestation),
    Sponsorship(VersionedSponsorship),
}

pub fn get_post_description(post: Post) -> String {
    return match post.snapshot.body.clone() {
        PostBody::Comment(comment) => comment.latest_version().description,
        PostBody::Idea(idea) => idea.latest_version().description,
        PostBody::Solution(solution) => solution.latest_version().description,
        PostBody::Attestation(attestation) => attestation.latest_version().description,
        PostBody::Sponsorship(sponsorship) => sponsorship.latest_version().description,
    };
}
