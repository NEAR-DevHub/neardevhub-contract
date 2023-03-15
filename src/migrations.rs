//! Public methods of data model/state migrations between the versions.
//! Should be invocable only by the owner and in most cases should be called only once though the
//! latter is not asserted.

use crate::*;
use near_sdk::{env, near_bindgen};
use std::cmp::min;
use std::collections::HashSet;

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct OldContractV1 {
    pub posts: Vector<VersionedPost>,
    pub post_to_parent: LookupMap<PostId, PostId>,
    pub post_to_children: LookupMap<PostId, Vec<PostId>>,
    pub label_to_posts: UnorderedMap<String, HashSet<PostId>>,
}

// From OldContractV1 to OldContractV2
#[near_bindgen]
impl Contract {
    pub fn unsafe_add_acl() {
        near_sdk::assert_self();
        let OldContractV1 { posts, post_to_parent, post_to_children, label_to_posts } =
            env::state_read().unwrap();
        env::state_write(&OldContractV2 {
            posts,
            post_to_parent,
            post_to_children,
            label_to_posts,
            access_control: Default::default(),
        });
    }
}

// // Fake vector purely for the sake of overriding initialization.
// #[derive(BorshSerialize, BorshDeserialize)]
// pub struct FakeVector {
//     len: u64,
//     prefix: Vec<u8>,
// }
//
// impl FakeVector {
//     pub fn new<S>(len: u64, prefix: S) -> Self
//     where
//         S: IntoStorageKey,
//     {
//         Self { len, prefix: prefix.into_storage_key() }
//     }
// }

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct OldContractV2 {
    pub posts: Vector<VersionedPost>,
    pub post_to_parent: LookupMap<PostId, PostId>,
    pub post_to_children: LookupMap<PostId, Vec<PostId>>,
    pub label_to_posts: UnorderedMap<String, HashSet<PostId>>,
    pub access_control: AccessControl,
}

// From OldContractV2 to Contract
#[near_bindgen]
impl Contract {
    pub fn unsafe_add_post_authors() {
        near_sdk::assert_self();
        let OldContractV2 {
            posts,
            post_to_parent,
            post_to_children,
            label_to_posts,
            access_control,
        } = env::state_read().unwrap();
        let authors = UnorderedMap::new(StorageKey::AuthorToAuthorPosts);

        env::state_write(&Contract {
            posts,
            post_to_parent,
            post_to_children,
            label_to_posts,
            access_control,
            authors,
        });
    }

    pub fn add_old_post_authors(&mut self, start: u64, end: u64) {
        near_sdk::assert_self();
        let total = self.posts.len();
        let end = min(total, end);
        for i in start..end {
            let versioned_post = self.posts.get(i);
            if let Some(versioned_post) = versioned_post {
                let post: Post = versioned_post.into();
                let mut author_posts =
                    self.authors.get(&post.author_id).unwrap_or_else(|| HashSet::new());
                author_posts.insert(post.id);
                self.authors.insert(&post.author_id, &author_posts);
            }
        }
    }
}
