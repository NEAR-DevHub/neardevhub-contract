pub mod migrations;
pub mod post;
pub mod stats;
pub mod str_serializers;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, Vector};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault};
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

    pub fn add_post(&mut self, parent_id: Option<PostId>, body: PostBody, labels: HashSet<String>) {
        let parent_id = parent_id.unwrap_or(ROOT_POST_ID);
        let id = self.posts.len();
        let author_id = env::predecessor_account_id();
        let editor_id = author_id.clone();

        for label in &labels {
            let mut other_posts = self.label_to_posts.get(label).unwrap_or_default();
            other_posts.insert(id);
            self.label_to_posts.insert(label, &other_posts);
        }
        let labels = labels.into_iter().map(|name| Label { name }).collect();

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

    pub fn get_posts_by_label(&self, label: String) -> Vec<PostId> {
        let mut res: Vec<_> =
            self.label_to_posts.get(&label).unwrap_or_default().into_iter().collect();
        res.sort();
        res
    }

    pub fn get_all_labels(&self) -> Vec<String> {
        let mut res: Vec<_> = self.label_to_posts.keys().collect();
        res.sort();
        res
    }

    pub fn is_allowed_to_edit(&self, post_id: PostId, editor: Option<AccountId>) -> bool {
        let post: Post = self.posts.get(post_id).expect("Post id not found").into();
        let editor = match editor {
            None => env::predecessor_account_id(),
            Some(e) => e,
        };
        // TODO: Allow moderators to edit posts.
        editor == env::current_account_id() || editor == post.author_id
    }

    pub fn edit_post(&mut self, id: PostId, body: PostBody, labels: HashSet<String>) {
        assert!(
            self.is_allowed_to_edit(id, Option::None),
            "The account is not allowed to edit this post"
        );
        let editor_id = env::predecessor_account_id();
        let mut post: Post = self.posts.get(id).expect("Post id not found").into();

        let old_snapshot = post.snapshot.clone();
        let old_labels_set = old_snapshot.labels.clone();
        let new_labels: HashSet<_> = labels.into_iter().map(|name| Label { name }).collect();
        let new_snapshot = PostSnapshot {
            editor_id,
            timestamp: env::block_timestamp(),
            labels: new_labels.clone(),
            body,
        };
        post.snapshot = new_snapshot;
        post.snapshot_history.push(old_snapshot);
        self.posts.push(&post.into());

        // Update labels index.

        let new_labels_set = new_labels;
        let labels_to_remove = &old_labels_set - &new_labels_set;
        let labels_to_add = &new_labels_set - &old_labels_set;
        for label_to_remove in labels_to_remove {
            let mut posts = self.label_to_posts.get(&label_to_remove.name).unwrap();
            posts.remove(&id);
            self.label_to_posts.insert(&label_to_remove.name, &posts);
        }

        for label_to_add in labels_to_add {
            let mut posts = self.label_to_posts.get(&label_to_add.name).unwrap();
            posts.insert(id);
            self.label_to_posts.insert(&label_to_add.name, &posts);
        }
    }
}
