//! Public methods of data model/state migrations between the versions.
//! Should be invocable only by the owner and in most cases should be called only once though the
//! latter is not asserted.

use crate::*;
use near_sdk::{env, near_bindgen, Promise};
use std::cmp::min;
use std::collections::HashSet;

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct ContractV1 {
    pub posts: Vector<VersionedPost>,
    pub post_to_parent: LookupMap<PostId, PostId>,
    pub post_to_children: LookupMap<PostId, Vec<PostId>>,
    pub label_to_posts: UnorderedMap<String, HashSet<PostId>>,
}

// From ContractV1 to ContractV2
#[near_bindgen]
impl Contract {
    fn unsafe_add_acl() {
        let ContractV1 { posts, post_to_parent, post_to_children, label_to_posts } =
            env::state_read().unwrap();
        env::state_write(&ContractV2 {
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
pub struct ContractV2 {
    pub posts: Vector<VersionedPost>,
    pub post_to_parent: LookupMap<PostId, PostId>,
    pub post_to_children: LookupMap<PostId, Vec<PostId>>,
    pub label_to_posts: UnorderedMap<String, HashSet<PostId>>,
    pub access_control: AccessControl,
}

// From ContractV2 to ContractV3
#[near_bindgen]
impl Contract {
    fn unsafe_add_post_authors() {
        let ContractV2 { posts, post_to_parent, post_to_children, label_to_posts, access_control } =
            env::state_read().unwrap();
        let authors = UnorderedMap::new(StorageKey::AuthorToAuthorPosts);

        env::state_write(&ContractV3 {
            posts,
            post_to_parent,
            post_to_children,
            label_to_posts,
            access_control,
            authors,
        });
    }

    fn unsafe_insert_old_post_authors(start: u64, end: u64) -> StateVersion {
        let mut contract: ContractV3 = env::state_read().unwrap();
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
pub struct ContractV3 {
    pub posts: Vector<VersionedPost>,
    pub post_to_parent: LookupMap<PostId, PostId>,
    pub post_to_children: LookupMap<PostId, Vec<PostId>>,
    pub label_to_posts: UnorderedMap<String, HashSet<PostId>>,
    pub access_control: AccessControl,
    pub authors: UnorderedMap<AccountId, HashSet<PostId>>,
}

// From ContractV3 to ContractV4
#[near_bindgen]
impl Contract {
    fn unsafe_add_communities() {
        let ContractV3 {
            posts,
            post_to_parent,
            post_to_children,
            label_to_posts,
            access_control,
            authors,
        } = env::state_read().unwrap();
        env::state_write(&ContractV4 {
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
pub struct ContractV4 {
    pub posts: Vector<VersionedPost>,
    pub post_to_parent: LookupMap<PostId, PostId>,
    pub post_to_children: LookupMap<PostId, Vec<PostId>>,
    pub label_to_posts: UnorderedMap<String, HashSet<PostId>>,
    pub access_control: AccessControl,
    pub authors: UnorderedMap<AccountId, HashSet<PostId>>,
    pub communities: UnorderedMap<String, CommunityV1>,
}

// From ContractV4 to ContractV5
#[near_bindgen]
impl Contract {
    fn unsafe_add_featured_communities() {
        let ContractV4 {
            posts,
            post_to_parent,
            post_to_children,
            label_to_posts,
            access_control,
            authors,
            communities,
        } = env::state_read().unwrap();
        env::state_write(&ContractV5 {
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

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct CommunityV1 {
    pub handle: CommunityHandle,
    pub admins: Vec<AccountId>,
    pub name: String,
    pub description: String,
    pub bio_markdown: Option<String>,
    pub logo_url: String,
    pub banner_url: String,
    pub tag: String,
    pub github_handle: Option<String>,
    pub telegram_handle: Option<String>,
    pub twitter_handle: Option<String>,
    pub website_url: Option<String>,
    /// JSON string of github board configuration
    pub github: Option<String>,
    pub sponsorship: Option<bool>,
    pub wiki1: Option<WikiPage>,
    pub wiki2: Option<WikiPage>,
}

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct ContractV5 {
    pub posts: Vector<VersionedPost>,
    pub post_to_parent: LookupMap<PostId, PostId>,
    pub post_to_children: LookupMap<PostId, Vec<PostId>>,
    pub label_to_posts: UnorderedMap<String, HashSet<PostId>>,
    pub access_control: AccessControl,
    pub authors: UnorderedMap<AccountId, HashSet<PostId>>,
    pub communities: UnorderedMap<String, CommunityV1>,
    pub featured_communities: Vec<FeaturedCommunity>,
}

// From ContractV5 to ContractV6
#[near_bindgen]
impl Contract {
    fn unsafe_multiple_telegrams() {
        let ContractV5 {
            posts,
            post_to_parent,
            post_to_children,
            label_to_posts,
            access_control,
            authors,
            mut communities,
            featured_communities,
        } = env::state_read().unwrap();

        let migrated_communities: Vec<(String, CommunityV2)> = communities
            .iter()
            .map(|(community_handle, community)| {
                (
                    community_handle,
                    CommunityV2 {
                        handle: community.handle,
                        admins: community.admins,
                        name: community.name,
                        description: community.description,
                        bio_markdown: community.bio_markdown,
                        logo_url: community.logo_url,
                        banner_url: community.banner_url,
                        tag: community.tag,
                        github_handle: community.github_handle,
                        telegram_handle: community.telegram_handle.iter().cloned().collect(),
                        twitter_handle: community.twitter_handle,
                        website_url: community.website_url,
                        github: community.github,
                        sponsorship: community.sponsorship,
                        wiki1: community.wiki1,
                        wiki2: community.wiki2,
                    },
                )
            })
            .collect();

        communities.clear();

        let mut communities_new = UnorderedMap::new(StorageKey::Communities);

        for (k, v) in migrated_communities {
            communities_new.insert(&k, &v);
        }

        env::state_write(&ContractV6 {
            posts,
            post_to_parent,
            post_to_children,
            label_to_posts,
            access_control,
            authors,
            communities: communities_new,
            featured_communities,
        });
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct CommunityV2 {
    pub handle: CommunityHandle,
    pub admins: Vec<AccountId>,
    pub name: String,
    pub description: String,
    pub bio_markdown: Option<String>,
    pub logo_url: String,
    pub banner_url: String,
    pub tag: String,
    pub github_handle: Option<String>,
    pub telegram_handle: Vec<String>,
    pub twitter_handle: Option<String>,
    pub website_url: Option<String>,
    /// JSON string of github board configuration
    pub github: Option<String>,
    pub sponsorship: Option<bool>,
    pub wiki1: Option<WikiPage>,
    pub wiki2: Option<WikiPage>,
}

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct ContractV6 {
    pub posts: Vector<VersionedPost>,
    pub post_to_parent: LookupMap<PostId, PostId>,
    pub post_to_children: LookupMap<PostId, Vec<PostId>>,
    pub label_to_posts: UnorderedMap<String, HashSet<PostId>>,
    pub access_control: AccessControl,
    pub authors: UnorderedMap<AccountId, HashSet<PostId>>,
    pub communities: UnorderedMap<String, CommunityV2>,
    pub featured_communities: Vec<FeaturedCommunity>,
}

// From ContractV6 to ContractV7
#[near_bindgen]
impl Contract {
    fn unsafe_add_workspaces() {
        let ContractV6 {
            posts,
            post_to_parent,
            post_to_children,
            label_to_posts,
            access_control,
            authors,
            mut communities,
            featured_communities,
        } = env::state_read().unwrap();

        let migrated_communities: Vec<(String, CommunityV3)> = communities
            .iter()
            .map(|(community_handle, community)| {
                (
                    community_handle,
                    CommunityV3 {
                        handle: community.handle,
                        admins: community.admins,
                        name: community.name,
                        description: community.description,
                        bio_markdown: community.bio_markdown,
                        logo_url: community.logo_url,
                        banner_url: community.banner_url,
                        tag: community.tag,
                        github_handle: community.github_handle,
                        telegram_handle: community.telegram_handle,
                        twitter_handle: community.twitter_handle,
                        website_url: community.website_url,
                        github: community.github,
                        wiki1: community.wiki1,
                        wiki2: community.wiki2,
                        board: None,
                    },
                )
            })
            .collect();

        communities.clear();

        let mut communities_new = UnorderedMap::new(StorageKey::Communities);

        for (k, v) in migrated_communities {
            communities_new.insert(&k, &v);
        }

        env::state_write(&ContractV7 {
            posts,
            post_to_parent,
            post_to_children,
            label_to_posts,
            access_control,
            authors,
            communities: communities_new,
            featured_communities,
        });
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct CommunityV3 {
    pub handle: CommunityHandle,
    pub admins: Vec<AccountId>,
    pub name: String,
    pub description: String,
    pub bio_markdown: Option<String>,
    pub logo_url: String,
    pub banner_url: String,
    pub tag: String,
    pub github_handle: Option<String>,
    pub telegram_handle: Vec<String>,
    pub twitter_handle: Option<String>,
    pub website_url: Option<String>,
    /// JSON string of github board configuration
    pub github: Option<String>,
    /// JSON string of kanban board configuration
    pub board: Option<String>,
    pub wiki1: Option<WikiPage>,
    pub wiki2: Option<WikiPage>,
}

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct ContractV7 {
    pub posts: Vector<VersionedPost>,
    pub post_to_parent: LookupMap<PostId, PostId>,
    pub post_to_children: LookupMap<PostId, Vec<PostId>>,
    pub label_to_posts: UnorderedMap<String, HashSet<PostId>>,
    pub access_control: AccessControl,
    pub authors: UnorderedMap<AccountId, HashSet<PostId>>,
    pub communities: UnorderedMap<String, CommunityV3>,
    pub featured_communities: Vec<FeaturedCommunity>,
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub(crate) enum StateVersion {
    V1,
    V2,
    V3 { done: bool, migrated_count: u64 },
    V4,
    V5,
    V6,
    V7,
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
            StateVersion::V5 => {
                Contract::unsafe_multiple_telegrams();
                state_version_write(&StateVersion::V6);
            }
            StateVersion::V6 => {
                Contract::unsafe_add_workspaces();
                state_version_write(&StateVersion::V7);
            }
            _ => {
                return Contract::migration_done();
            }
        }
        Contract::needs_migration();
    }
}
