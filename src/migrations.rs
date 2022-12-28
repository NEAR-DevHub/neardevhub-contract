//! Public methods of data model/state migrations between the versions.
//! Should be invocable only by the owner and in most cases should be called only once though the
//! latter is not asserted.

use crate::*;
use near_sdk::near_bindgen;

#[near_bindgen]
impl Contract {
    /// This code was called only once to upgrade the contract to contain comments.
    pub fn unsafe_initiate_comments() {
        assert_eq!(
            env::current_account_id(),
            env::predecessor_account_id(),
            "Can only be called by the account itself"
        );
        let v: Vector<Comment> = Vector::new(StorageKey::Comments);
        let data = v.try_to_vec().expect("Cannot serialize the contract state.");
        env::storage_write(
            &StorageKey::Comments.try_to_vec().expect("Cannot serialize comments key"),
            &data,
        );

        env::state_write(&Self::new());
    }

    /// This code was used to migrate comments to new version.
    /// Adds id.
    pub fn unsafe_migrate_comments_to_v1(&mut self) {
        assert_eq!(
            env::current_account_id(),
            env::predecessor_account_id(),
            "Can only be called by the account itself"
        );
        for i in 0..self.comments.len() {
            let c: Comment = self.comments.get(i).unwrap().into();
            let new_c: VersionedComment = Comment {
                id: i,
                author_id: c.author_id,
                timestamp: c.timestamp,
                description: c.description,
                likes: c.likes,
                comments: c.comments,
            }
            .into();
            self.comments.replace(i, &new_c);
        }
    }
}
