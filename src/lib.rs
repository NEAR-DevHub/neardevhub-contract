pub mod migrations;
pub mod post;
pub mod stats;
pub mod str_serializers;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, Vector};
use near_sdk::{env, near_bindgen, PanicOnDefault};
use post::*;
use std::collections::HashSet;

near_sdk::setup_alloc!();

type PostId = u64;
type IdeaId = u64;
type AttestationId = u64;
type SubmissionId = u64;
type SponsorshipId = u64;
type CommentId = u64;

/// An imaginary top post representing the landing page.
const ROOT_POST_ID: u64 = u64::MAX;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub posts: Vector<VersionedPost>,
    pub post_to_parent: LookupMap<PostId, PostId>,
    pub post_to_children: LookupMap<PostId, Vec<PostId>>,
    pub label_to_posts: UnorderedMap<String, HashSet<PostId>>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {
            posts: Vector::new(StorageKey::Posts),
            post_to_parent: LookupMap::new(StorageKey::PostToParent),
            post_to_children: LookupMap::new(StorageKey::PostToChildren),
            label_to_posts: UnorderedMap::new(StorageKey::LabelToPosts),
        }
    }

    /// If `parent_id` is not provided get all landing page posts. Otherwise, get all posts under
    /// `parent_id` post.
    pub fn get_posts(&self, parent_id: Option<PostId>) -> Vec<VersionedPost> {
        let parent_id = parent_id.unwrap_or(ROOT_POST_ID);
        let children_ids = self.post_to_children.get(&parent_id).expect("Parent id not found");
        children_ids
            .into_iter()
            .map(|id| self.posts.get(id).expect("Post with id not found. Broken state invariant."))
            .collect()
    }

    pub fn get_post(&self, post_id: PostId) -> VersionedPost {
        self.posts.get(post_id).expect("Post id not found")
    }

    pub fn get_children_ids(&self, post_id: Option<PostId>) -> Vec<PostId> {
        let post_id = post_id.unwrap_or(ROOT_POST_ID);
        self.post_to_children.get(&post_id).expect("Parent id not found")
    }

    pub fn get_parent_id(&self, post_id: PostId) -> Option<PostId> {
        let res = self.post_to_parent.get(&post_id).expect("Parent id not found");
        if res == ROOT_POST_ID {
            Option::None
        } else {
            Option::Some(res)
        }
    }

    pub fn add_like(&mut self, post_id: PostId) {
        let mut post: Post = self.posts.get(post_id).expect("Post id not found").into();
        let like =
            Like { author_id: env::predecessor_account_id(), timestamp: env::block_timestamp() };
        post.likes.insert(like);
        self.posts.replace(post_id, &post.into());
    }

    pub fn add_post(&mut self, parent_id: Option<PostId>, body: PostBody, labels: HashSet<Label>) {
        let parent_id = parent_id.unwrap_or(ROOT_POST_ID);
        let id = self.posts.len();
        let author_id = env::predecessor_account_id();
        let editor_id = author_id.clone();
        let post = Post {
            id,
            author_id,
            likes: Default::default(),
            snapshot: PostSnapshot { editor_id, timestamp: env::block_timestamp(), labels, body },
            snapshot_history: vec![],
        };
        self.posts.push(&post.into());
        self.post_to_parent.insert(&id, &parent_id);

        let mut siblings = self.post_to_children.get(&parent_id).expect("Parent id not found");
        siblings.push(id);
        self.post_to_children.insert(&parent_id, &siblings);
    }
}
