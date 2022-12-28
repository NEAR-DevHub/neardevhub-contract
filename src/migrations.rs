//! Public methods of data model/state migrations between the versions.
//! Should be invocable only by the owner and in most cases should be called only once though the
//! latter is not asserted.

use crate::*;
use near_sdk::{near_bindgen, IntoStorageKey};
use std::collections::HashMap;

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

    pub fn unsafe_initiate_posts() {
        assert_eq!(
            env::current_account_id(),
            env::predecessor_account_id(),
            "Can only be called by the account itself"
        );

        {
            let posts: Vector<Post> = Vector::new(StorageKey::Posts);
            let data = posts.try_to_vec().expect("Cannot serialize the contract state.");
            env::storage_write(
                &StorageKey::Posts.try_to_vec().expect("Cannot serialize posts key"),
                &data,
            );
        }

        {
            let post_to_parent: LookupMap<PostId, PostId> =
                LookupMap::new(StorageKey::PostToParent);
            let data = post_to_parent.try_to_vec().expect("Cannot serialize the contract state.");
            env::storage_write(
                &StorageKey::PostToParent
                    .try_to_vec()
                    .expect("Cannot serialize post to parent key"),
                &data,
            );
        }

        {
            let post_to_children: LookupMap<PostId, Vec<PostId>> =
                LookupMap::new(StorageKey::PostToChildren);
            let data = post_to_children.try_to_vec().expect("Cannot serialize the contract state.");
            env::storage_write(
                &StorageKey::PostToChildren
                    .try_to_vec()
                    .expect("Cannot serialize post to children key"),
                &data,
            );
        }

        env::state_write(&FakeContract {
            ideas: FakeVector::new(32, StorageKey::Ideas),
            submissions: FakeVector::new(6, StorageKey::Submissions),
            attestations: FakeVector::new(1, StorageKey::Attestations),
            sponsorships: FakeVector::new(1, StorageKey::Sponsorships),
            comments: FakeVector::new(15, StorageKey::Comments),
            posts: Vector::new(StorageKey::Posts),
            post_to_parent: LookupMap::new(StorageKey::PostToParent),
            post_to_children: LookupMap::new(StorageKey::PostToChildren),
        });
    }

    /// This code was used to migrate comments to new version.
    /// Adds id.
    pub fn unsafe_copy_to_posts(&mut self) {
        assert_eq!(
            env::current_account_id(),
            env::predecessor_account_id(),
            "Can only be called by the account itself"
        );
        self.posts.clear();

        // A pair (old id, post).
        let mut posts_to_add = vec![];

        for idea_id in 0..self.ideas.len() {
            let Idea {
                id,
                name,
                description,
                author_id,
                timestamp,
                status,
                likes,
                comments,
                submissions,
            } = self.ideas.get(idea_id).unwrap().into();

            // Fields that are not useful anymore.
            let _ = status;

            // Will be used in the next loop;
            let _ = (comments, submissions);

            let new_post = VersionedPost::V0(Post {
                author_id: author_id.clone(),
                likes,
                snapshot: PostSnapshot {
                    editor_id: author_id,
                    timestamp,
                    labels: Default::default(),
                    body: PostBody::Idea(VersionedIdea::V1(IdeaV1 { name, description })),
                },
                snapshot_history: vec![],
            });
            posts_to_add.push((id, new_post));
        }

        for submission_id in 0..self.submissions.len() {
            let Submission {
                id,
                name,
                description,
                author_id,
                timestamp,
                status,
                likes,
                comments,
                idea_id,
                attestations,
                sponsorships,
            } = self.submissions.get(submission_id).unwrap().into();

            // Fields that are not useful anymore.
            let _ = (status, idea_id);

            // Will be used in the next loop;
            let _ = (comments, attestations, sponsorships);

            let new_post = VersionedPost::V0(Post {
                author_id: author_id.clone(),
                likes,
                snapshot: PostSnapshot {
                    editor_id: author_id,
                    timestamp,
                    labels: Default::default(),
                    body: PostBody::Submission(VersionedSubmission::V1(SubmissionV1 {
                        name,
                        description,
                    })),
                },
                snapshot_history: vec![],
            });
            posts_to_add.push((id, new_post));
        }

        for attestation_id in 0..self.attestations.len() {
            let Attestation {
                id,
                name,
                description,
                author_id,
                timestamp,
                status,
                likes,
                comments,
                submission_id,
            } = self.attestations.get(attestation_id).unwrap().into();

            // Fields that are not useful anymore.
            let _ = (status, submission_id);

            // Will be used in the next loop;
            let _ = comments;

            let new_post = VersionedPost::V0(Post {
                author_id: author_id.clone(),
                likes,
                snapshot: PostSnapshot {
                    editor_id: author_id,
                    timestamp,
                    labels: Default::default(),
                    body: PostBody::Attestation(VersionedAttestation::V1(AttestationV1 {
                        name,
                        description,
                    })),
                },
                snapshot_history: vec![],
            });
            posts_to_add.push((id, new_post));
        }

        for sponsorship_id in 0..self.sponsorships.len() {
            let Sponsorship {
                id,
                name,
                description,
                author_id,
                timestamp,
                status,
                likes,
                comments,
                submission_id,
                sponsorship_token,
                amount,
                supervisor,
            } = self.sponsorships.get(sponsorship_id).unwrap().into();

            // Fields that are not useful anymore.
            let _ = (status, submission_id);

            // Will be used in the next loop;
            let _ = comments;

            let new_post = VersionedPost::V0(Post {
                author_id: author_id.clone(),
                likes,
                snapshot: PostSnapshot {
                    editor_id: author_id,
                    timestamp,
                    labels: Default::default(),
                    body: PostBody::Sponsorship(VersionedSponsorship::V1(SponsorshipV1 {
                        name,
                        description,
                        sponsorship_token,
                        amount,
                        supervisor,
                    })),
                },
                snapshot_history: vec![],
            });

            posts_to_add.push((id, new_post));
        }

        for comment_id in 0..self.comments.len() {
            let Comment { id, author_id, timestamp, description, likes, comments } =
                self.comments.get(comment_id).unwrap().into();

            // Will be used in the next loop;
            let _ = comments;

            let new_post = VersionedPost::V0(Post {
                author_id: author_id.clone(),
                likes,
                snapshot: PostSnapshot {
                    editor_id: author_id,
                    timestamp,
                    labels: Default::default(),
                    body: PostBody::Comment(VersionedComment::V2(CommentV2 { description })),
                },
                snapshot_history: vec![],
            });

            posts_to_add.push((id, new_post));
        }

        // Pretend like posts were added in time sequential order, just in case for the future.
        posts_to_add.sort_by_key(|p| match &p.1 {
            VersionedPost::V0(v0) => v0.snapshot.timestamp,
        });

        let mut old_to_new_id_comment: HashMap<u64, PostId> = HashMap::new();
        let mut old_to_new_id_idea: HashMap<u64, PostId> = HashMap::new();
        let mut old_to_new_id_submission: HashMap<u64, PostId> = HashMap::new();
        let mut old_to_new_id_attestation: HashMap<u64, PostId> = HashMap::new();
        let mut old_to_new_id_sponsorship: HashMap<u64, PostId> = HashMap::new();

        for new_id in 0..posts_to_add.len() {
            let (old_id, post) = posts_to_add[new_id].clone();
            match &post {
                VersionedPost::V0(post) => match post.snapshot.body {
                    PostBody::Comment(_) => {
                        old_to_new_id_comment.insert(old_id, new_id as u64);
                    }
                    PostBody::Idea(_) => {
                        old_to_new_id_idea.insert(old_id, new_id as u64);
                    }
                    PostBody::Submission(_) => {
                        old_to_new_id_submission.insert(old_id, new_id as u64);
                    }
                    PostBody::Attestation(_) => {
                        old_to_new_id_attestation.insert(old_id, new_id as u64);
                    }
                    PostBody::Sponsorship(_) => {
                        old_to_new_id_sponsorship.insert(old_id, new_id as u64);
                    }
                },
            }
            self.posts.push(&post);
        }
        for new_id in 0..posts_to_add.len() as u64 {
            let (old_id, post) = &posts_to_add[new_id as usize];
            #[allow(irrefutable_let_patterns)]
            if let VersionedPost::V0(post) = &post {
                match &post.snapshot.body {
                    PostBody::Comment(_) => {
                        let c: Comment = self.comments.get(*old_id).unwrap().into();
                        let mut new_children = vec![];
                        for old_child_id in c.comments {
                            let new_child_id = old_to_new_id_comment[&old_child_id];
                            self.post_to_parent.insert(&new_child_id, &new_id);
                            new_children.push(new_child_id);
                        }
                        self.post_to_children.insert(&new_id, &new_children);
                    }
                    PostBody::Idea(_) => {
                        let i: Idea = self.ideas.get(*old_id).unwrap().into();
                        let mut new_children = vec![];
                        for old_child_id in i.comments {
                            let new_child_id = old_to_new_id_comment[&old_child_id];
                            self.post_to_parent.insert(&new_child_id, &new_id);
                            new_children.push(new_child_id);
                        }
                        for old_child_id in i.submissions {
                            let new_child_id = old_to_new_id_submission[&old_child_id];
                            self.post_to_parent.insert(&new_child_id, &new_id);
                            new_children.push(new_child_id);
                        }
                        self.post_to_children.insert(&new_id, &new_children);
                    }
                    PostBody::Submission(_) => {
                        let s: Submission = self.submissions.get(*old_id).unwrap().into();
                        let mut new_children = vec![];
                        for old_child_id in s.comments {
                            let new_child_id = old_to_new_id_comment[&old_child_id];
                            self.post_to_parent.insert(&new_child_id, &new_id);
                            new_children.push(new_child_id);
                        }
                        for old_child_id in s.attestations {
                            let new_child_id = old_to_new_id_attestation[&old_child_id];
                            self.post_to_parent.insert(&new_child_id, &new_id);
                            new_children.push(new_child_id);
                        }
                        for old_child_id in s.sponsorships {
                            let new_child_id = old_to_new_id_sponsorship[&old_child_id];
                            self.post_to_parent.insert(&new_child_id, &new_id);
                            new_children.push(new_child_id);
                        }
                        self.post_to_children.insert(&new_id, &new_children);
                    }
                    PostBody::Attestation(_) => {
                        let a: Attestation = self.attestations.get(*old_id).unwrap().into();
                        let mut new_children = vec![];
                        for old_child_id in a.comments {
                            let new_child_id = old_to_new_id_attestation[&old_child_id];
                            self.post_to_parent.insert(&new_child_id, &new_id);
                            new_children.push(new_child_id);
                        }
                        self.post_to_children.insert(&new_id, &new_children);
                    }
                    PostBody::Sponsorship(_) => {
                        let s: Sponsorship = self.sponsorships.get(*old_id).unwrap().into();
                        let mut new_children = vec![];
                        for old_child_id in s.comments {
                            let new_child_id = old_to_new_id_sponsorship[&old_child_id];
                            self.post_to_parent.insert(&new_child_id, &new_id);
                            new_children.push(new_child_id);
                        }
                        self.post_to_children.insert(&new_id, &new_children);
                    }
                }
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
    pub ideas: FakeVector,
    pub submissions: FakeVector,
    pub attestations: FakeVector,
    pub sponsorships: FakeVector,
    pub comments: FakeVector,
    pub posts: Vector<Post>,
    pub post_to_parent: LookupMap<PostId, PostId>,
    pub post_to_children: LookupMap<PostId, Vec<PostId>>,
}
