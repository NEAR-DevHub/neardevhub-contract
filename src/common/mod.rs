use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{BorshStorageKey, CryptoHash, NearSchema};

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "post_version")]
#[borsh(crate = "near_sdk::borsh")]
pub enum VersionedPost {
    V0,
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
    ChangeLog,
}
