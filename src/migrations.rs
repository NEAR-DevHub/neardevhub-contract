//! Public methods of data model/state migrations between the versions.
//! Should be invocable only by the owner and in most cases should be called only once though the
//! latter is not asserted.

use crate::*;
use near_sdk::{near_bindgen, IntoStorageKey};

#[near_bindgen]
impl Contract {
    // pub fn unsafe_initiate_labels_remove_old_collections() {
    //     assert_eq!(
    //         env::current_account_id(),
    //         env::predecessor_account_id(),
    //         "Can only be called by the account itself"
    //     );
    //
    //     // First remove old collections.
    //     let mut ideas: Vector<VersionedIdea> = Vector::new(StorageKey::Ideas);
    //     let mut submissions: Vector<VersionedSubmission> = Vector::new(StorageKey::Submissions);
    //     let mut attestations: Vector<VersionedAttestation> = Vector::new(StorageKey::Attestations);
    //     let mut sponsorships: Vector<VersionedSponsorship> = Vector::new(StorageKey::Sponsorships);
    //     let mut comments: Vector<VersionedComment> = Vector::new(StorageKey::Comments);
    //
    //     ideas.clear();
    //     submissions.clear();
    //     attestations.clear();
    //     sponsorships.clear();
    //     comments.clear();
    //
    //     env::state_write(&FakeContract {
    //         posts: FakeVector::new(64, StorageKey::Posts),
    //         post_to_parent: LookupMap::new(StorageKey::PostToParent),
    //         post_to_children: LookupMap::new(StorageKey::PostToChildren),
    //         label_to_posts: UnorderedMap::new(StorageKey::LabelToPostsV2),
    //     });
    // }
    //
    // pub fn unsafe_purge_one_post() {
    //     assert_eq!(
    //         env::current_account_id(),
    //         env::predecessor_account_id(),
    //         "Can only be called by the account itself"
    //     );
    //
    //     env::state_write(&FakeContract {
    //         posts: FakeVector::new(64, StorageKey::Posts),
    //         post_to_parent: LookupMap::new(StorageKey::PostToParent),
    //         post_to_children: LookupMap::new(StorageKey::PostToChildren),
    //         label_to_posts: UnorderedMap::new(StorageKey::LabelToPostsV2),
    //     });
    // }

    pub fn unsafe_fix_missing_children(&mut self) {
        assert_eq!(
            env::current_account_id(),
            env::predecessor_account_id(),
            "Can only be called by the account itself"
        );
        for id in 0..self.posts.len() {
            if self.post_to_children.get(&id).is_none() {
                self.post_to_children.insert(&id, &vec![]);
            }
        }
    }
}

// Fake vector purely for the sake of overriding initialization.
#[derive(BorshSerialize, BorshDeserialize)]
pub struct FakeVector {
    len: u64,
    prefix: Vec<u8>,
}

impl FakeVector {
    pub fn new<S>(len: u64, prefix: S) -> Self
    where
        S: IntoStorageKey,
    {
        Self { len, prefix: prefix.into_storage_key() }
    }
}

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct FakeContract {
    pub posts: FakeVector,
    pub post_to_parent: LookupMap<PostId, PostId>,
    pub post_to_children: LookupMap<PostId, Vec<PostId>>,
    pub label_to_posts: UnorderedMap<String, HashSet<PostId>>,
}
