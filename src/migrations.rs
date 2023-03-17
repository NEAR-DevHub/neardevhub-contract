//! Public methods of data model/state migrations between the versions.
//! Should be invocable only by the owner and in most cases should be called only once though the
//! latter is not asserted.

use crate::*;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, Promise};
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
    fn unsafe_add_acl() {
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
    fn unsafe_add_post_authors() {
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

    fn insert_old_post_authors(&mut self, start: u64, end: u64) {
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

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum MigrateTo {
    V2,
    V3(V2ToV3Step),
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum V2ToV3Step {
    AddPostAuthorsField,
    InsertPostAuthors { start: u64, end: u64 },
}

#[near_bindgen]
impl Contract {
    // compile twice to include current devgovgigs.wasm
    pub fn unsafe_self_upgrade() {
        near_sdk::assert_self();

        let contract = env::input().expect("No contract code is attached in input");
        Promise::new(env::current_account_id()).deploy_contract(contract);
    }

    // Without `&mut self`, `unsafe_migrate` skips `near_bindgen`, which loads state, borsh deserialize and parse `input`.
    pub fn unsafe_migrate() {
        near_sdk::assert_self();
        let to: MigrateTo = near_sdk::serde_json::from_slice(
            &near_sdk::env::input().expect("Expected input since method has arguments."),
        )
        .expect("Failed to deserialize input from JSON.");

        match to {
            MigrateTo::V2 => Contract::unsafe_add_acl(),
            MigrateTo::V3(V2ToV3Step::AddPostAuthorsField) => Contract::unsafe_add_post_authors(),
            _ => panic!("unsupported unsafe_migrate step"),
        }
    }

    // With `&mut self`, `migrate` leverages `near_bindgen`.
    pub fn migrate(&mut self, to: MigrateTo) {
        near_sdk::assert_self();
        match to {
            MigrateTo::V3(V2ToV3Step::InsertPostAuthors { start, end }) => {
                self.insert_old_post_authors(start, end)
            }
            _ => panic!("unsupported migrate step"),
        }
    }
}
