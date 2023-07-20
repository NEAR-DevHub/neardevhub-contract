//! Public methods of data model/state migrations between the versions.
//! Should be invocable only by the owner and in most cases should be called only once though the
//! latter is not asserted.

use crate::*;
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

// From OldContractV2 to OldContractV3
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

        env::state_write(&OldContractV3 {
            posts,
            post_to_parent,
            post_to_children,
            label_to_posts,
            access_control,
            authors,
        });
    }

    fn unsafe_insert_old_post_authors(start: u64, end: u64) -> StateVersion {
        let mut contract: OldContractV3 = env::state_read().unwrap();
        let total = contract.posts.len();
        let end = min(total, end);
        for i in start..end {
            let versioned_post = contract.posts.get(i);
            if let Some(versioned_post) = versioned_post {
                let post: Post = versioned_post.into();
                let mut author_posts =
                    contract.authors.get(&post.author_id).unwrap_or_else(|| HashSet::new());
                author_posts.insert(post.id);
                contract.authors.insert(&post.author_id, &author_posts);
            }
        }
        env::state_write(&contract);
        StateVersion::V3 { done: end == total, migrated_count: end }
    }
}

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct OldContractV3 {
    pub posts: Vector<VersionedPost>,
    pub post_to_parent: LookupMap<PostId, PostId>,
    pub post_to_children: LookupMap<PostId, Vec<PostId>>,
    pub label_to_posts: UnorderedMap<String, HashSet<PostId>>,
    pub access_control: AccessControl,
    pub authors: UnorderedMap<AccountId, HashSet<PostId>>,
}

// From OldContractV3 to Contract
#[near_bindgen]
impl Contract {
    fn unsafe_add_communities() {
        let OldContractV3 {
            posts,
            post_to_parent,
            post_to_children,
            label_to_posts,
            access_control,
            authors,
        } = env::state_read().unwrap();
        env::state_write(&OldContractV4 {
            posts,
            post_to_parent,
            post_to_children,
            label_to_posts,
            access_control,
            authors,
            communities: UnorderedMap::new(StorageKey::Communities),
        });
    }
}

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct OldContractV4 {
    pub posts: Vector<VersionedPost>,
    pub post_to_parent: LookupMap<PostId, PostId>,
    pub post_to_children: LookupMap<PostId, Vec<PostId>>,
    pub label_to_posts: UnorderedMap<String, HashSet<PostId>>,
    pub access_control: AccessControl,
    pub authors: UnorderedMap<AccountId, HashSet<PostId>>,
    pub communities: UnorderedMap<String, Community>,
}

// From OldContractV3 to Contract
#[near_bindgen]
impl Contract {
    fn unsafe_add_featured_communities() {
        let OldContractV4 {
            posts,
            post_to_parent,
            post_to_children,
            label_to_posts,
            access_control,
            authors,
            communities,
        } = env::state_read().unwrap();
        env::state_write(&Contract {
            posts,
            post_to_parent,
            post_to_children,
            label_to_posts,
            access_control,
            authors,
            communities,
            featured_communities: Vec::new(),
        });
    }
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub(crate) enum StateVersion {
    V1,
    V2,
    V3 { done: bool, migrated_count: u64 },
    V4,
    V5,
}

const VERSION_KEY: &[u8] = b"VERSION";

fn state_version_read() -> StateVersion {
    env::storage_read(VERSION_KEY)
        .map(|data| {
            StateVersion::try_from_slice(&data).expect("Cannot deserialize the contract state.")
        })
        .unwrap_or(StateVersion::V2) // StateVersion is introduced in production contract with V2 State.
}

pub(crate) fn state_version_write(version: &StateVersion) {
    let data = version.try_to_vec().expect("Cannot serialize the contract state.");
    env::storage_write(VERSION_KEY, &data);
    near_sdk::log!("Migrated to version: {:?}", version);
}

#[near_bindgen]
impl Contract {
    pub fn unsafe_self_upgrade() {
        near_sdk::assert_self();

        let contract = env::input().expect("No contract code is attached in input");
        Promise::new(env::current_account_id())
            .deploy_contract(contract)
            .then(Promise::new(env::current_account_id()).function_call(
                b"unsafe_migrate".to_vec(),
                Vec::new(),
                0u128,
                env::prepaid_gas() - 60_000_000_000_000u64,
            ))
            .as_return();
    }

    fn migration_done() {
        near_sdk::log!("Migration done.");
        env::value_return(b"\"done\"");
    }

    fn needs_migration() {
        env::value_return(b"\"needs-migration\"");
    }

    pub fn unsafe_migrate() {
        near_sdk::assert_self();
        let current_version = state_version_read();
        near_sdk::log!("Migrating from version: {:?}", current_version);
        match current_version {
            StateVersion::V1 => {
                Contract::unsafe_add_acl();
                state_version_write(&StateVersion::V2);
            }
            StateVersion::V2 => {
                Contract::unsafe_add_post_authors();
                state_version_write(&StateVersion::V3 { done: false, migrated_count: 0 })
            }
            StateVersion::V3 { done: false, migrated_count } => {
                let new_version =
                    Contract::unsafe_insert_old_post_authors(migrated_count, migrated_count + 100);
                state_version_write(&new_version);
            }
            StateVersion::V3 { done: true, migrated_count: _ } => {
                Contract::unsafe_add_communities();
                state_version_write(&StateVersion::V4);
            }
            StateVersion::V4 => {
                Contract::unsafe_add_featured_communities();
                state_version_write(&StateVersion::V5);
            }
            _ => {
                return Contract::migration_done();
            }
        }
        Contract::needs_migration();
    }
}
