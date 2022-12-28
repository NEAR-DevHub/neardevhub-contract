pub mod migrations;
pub mod post;
pub mod str_serializers;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::Vector;
use near_sdk::{env, near_bindgen, AccountId, Balance, PanicOnDefault};
use post::*;
use std::str::FromStr;

near_sdk::setup_alloc!();

type IdeaId = u64;
type AttestationId = u64;
type SubmissionId = u64;
type SponsorshipId = u64;
type CommentId = u64;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub ideas: Vector<VersionedIdea>,
    pub submissions: Vector<VersionedSubmission>,
    pub attestations: Vector<VersionedAttestation>,
    pub sponsorships: Vector<VersionedSponsorship>,
    pub comments: Vector<VersionedComment>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {
            ideas: Vector::new(StorageKey::Ideas),
            submissions: Vector::new(StorageKey::Submissions),
            attestations: Vector::new(StorageKey::Attestations),
            sponsorships: Vector::new(StorageKey::Sponsorships),
            comments: Vector::new(StorageKey::Comments),
        }
    }

    pub fn add_idea(&mut self, name: String, description: String) {
        let id = self.ideas.len();
        self.ideas.push(
            &Idea {
                id,
                name,
                description,
                author_id: env::predecessor_account_id(),
                timestamp: env::block_timestamp(),
                status: PostStatus::Open,
                likes: Default::default(),
                comments: vec![],
                submissions: vec![],
            }
            .into(),
        )
    }

    pub fn get_idea(&self, idea_id: IdeaId) -> Idea {
        self.ideas.get(idea_id).unwrap().into()
    }

    pub fn get_num_ideas(&self) -> IdeaId {
        self.ideas.len()
    }

    pub fn get_ideas(&self) -> Vec<Idea> {
        self.ideas.iter().map(|vi| vi.into()).collect()
    }

    pub fn add_submission(&mut self, idea_id: IdeaId, name: String, description: String) {
        let id = self.submissions.len();

        let mut idea: Idea = self.ideas.get(idea_id).expect("Submission id not found").into();
        idea.submissions.push(id);
        self.ideas.replace(idea_id, &idea.into());

        self.submissions.push(
            &Submission {
                id,
                name,
                description,
                author_id: env::predecessor_account_id(),
                timestamp: env::block_timestamp(),
                status: PostStatus::Open,
                likes: Default::default(),
                comments: vec![],
                idea_id,
                attestations: vec![],
                sponsorships: vec![],
            }
            .into(),
        );
    }

    pub fn get_submissions(&self, idea_id: IdeaId) -> Vec<Submission> {
        let idea: Idea = self.ideas.get(idea_id).expect("Idea id not found").into();
        idea.submissions.iter().map(|id| self.submissions.get(*id).unwrap().into()).collect()
    }

    pub fn get_submission(&self, submission_id: SubmissionId) -> Submission {
        self.submissions.get(submission_id).unwrap().into()
    }

    pub fn add_attestation(
        &mut self,
        submission_id: SubmissionId,
        name: String,
        description: String,
    ) {
        let id = self.attestations.len();

        let mut submission: Submission =
            self.submissions.get(submission_id).expect("Submission id not found").into();
        submission.attestations.push(id);
        self.submissions.replace(submission_id, &submission.into());

        self.attestations.push(
            &Attestation {
                id,
                name,
                description,
                author_id: env::predecessor_account_id(),
                timestamp: env::block_timestamp(),
                status: PostStatus::Open,
                likes: Default::default(),
                comments: vec![],
                submission_id,
            }
            .into(),
        );
    }

    pub fn get_attestations(&self, submission_id: SubmissionId) -> Vec<Attestation> {
        let submission: Submission =
            self.submissions.get(submission_id).expect("Submission id not found").into();
        submission
            .attestations
            .iter()
            .map(|id| self.attestations.get(*id).unwrap().into())
            .collect()
    }

    pub fn get_attestation(&self, attestation_id: AttestationId) -> Attestation {
        self.attestations.get(attestation_id).unwrap().into()
    }

    pub fn add_sponsorship(
        &mut self,
        submission_id: SubmissionId,
        name: String,
        description: String,
        sponsorship_token: String,
        amount: String,
        supervisor: AccountId,
    ) {
        let id = self.sponsorships.len();

        let mut submission: Submission =
            self.submissions.get(submission_id).expect("Submission id not found").into();
        submission.sponsorships.push(id);
        self.submissions.replace(submission_id, &submission.into());

        self.sponsorships.push(
            &Sponsorship {
                id,
                name,
                description,
                author_id: env::predecessor_account_id(),
                timestamp: env::block_timestamp(),
                status: PostStatus::Open,
                likes: Default::default(),
                comments: vec![],
                submission_id,
                sponsorship_token: SponsorshipToken::from_str(&sponsorship_token).unwrap(),
                amount: Balance::from_str(&amount).unwrap(),
                supervisor,
            }
            .into(),
        );
    }

    pub fn get_sponsorships(&self, submission_id: SubmissionId) -> Vec<Sponsorship> {
        let submission: Submission =
            self.submissions.get(submission_id).expect("Submission id not found").into();
        submission
            .sponsorships
            .iter()
            .map(|id| self.sponsorships.get(*id).unwrap().into())
            .collect()
    }

    pub fn get_sponsorship(&self, sponsorship_id: SponsorshipId) -> Sponsorship {
        self.sponsorships.get(sponsorship_id).unwrap().into()
    }

    pub fn like(&mut self, post_type: PostType, post_id: u64) {
        let like =
            Like { author_id: env::predecessor_account_id(), timestamp: env::block_timestamp() };
        match post_type {
            PostType::Idea => {
                let mut idea: Idea = self.ideas.get(post_id).expect("Idea id not found").into();
                idea.likes.insert(like);
                self.ideas.replace(post_id, &idea.into());
            }
            PostType::Submission => {
                let mut submission: Submission =
                    self.submissions.get(post_id).expect("Submission id not found").into();
                submission.likes.insert(like);
                self.submissions.replace(post_id, &submission.into());
            }
            PostType::Attestation => {
                let mut attestation: Attestation =
                    self.attestations.get(post_id).expect("Attestation id not found").into();
                attestation.likes.insert(like);
                self.attestations.replace(post_id, &attestation.into());
            }
            PostType::Sponsorship => {
                let mut sponsorship: Sponsorship =
                    self.sponsorships.get(post_id).expect("Sponsorship id not found").into();
                sponsorship.likes.insert(like);
                self.sponsorships.replace(post_id, &sponsorship.into());
            }
            PostType::Comment => {
                let mut comment: Comment =
                    self.comments.get(post_id).expect("Comment id not found").into();
                comment.likes.insert(like);
                self.comments.replace(post_id, &comment.into());
            }
        }
    }

    pub fn comment(&mut self, post_type: PostType, post_id: u64, description: String) {
        let id = self.comments.len();
        let comment = Comment {
            id,
            author_id: env::predecessor_account_id(),
            timestamp: env::block_timestamp(),
            description,
            likes: Default::default(),
            comments: vec![],
        };
        self.comments.push(&comment.into());
        match post_type {
            PostType::Idea => {
                let mut idea: Idea = self.ideas.get(post_id).expect("Idea id not found").into();
                idea.comments.push(id);
                self.ideas.replace(post_id, &idea.into());
            }
            PostType::Submission => {
                let mut submission: Submission =
                    self.submissions.get(post_id).expect("Submission id not found").into();
                submission.comments.push(id);
                self.submissions.replace(post_id, &submission.into());
            }
            PostType::Attestation => {
                let mut attestation: Attestation =
                    self.attestations.get(post_id).expect("Attestation id not found").into();
                attestation.comments.push(id);
                self.attestations.replace(post_id, &attestation.into());
            }
            PostType::Sponsorship => {
                let mut sponsorship: Sponsorship =
                    self.sponsorships.get(post_id).expect("Sponsorship id not found").into();
                sponsorship.comments.push(id);
                self.sponsorships.replace(post_id, &sponsorship.into());
            }
            PostType::Comment => {
                let mut comment: Comment =
                    self.comments.get(post_id).expect("Comment id not found").into();
                comment.comments.push(id);
                self.comments.replace(post_id, &comment.into());
            }
        }
    }

    pub fn get_comments(&self, post_type: PostType, post_id: u64) -> Vec<Comment> {
        let comment_ids = match post_type {
            PostType::Idea => {
                let idea: Idea = self.ideas.get(post_id).expect("Idea id not found").into();
                idea.comments
            }
            PostType::Submission => {
                let submission: Submission =
                    self.submissions.get(post_id).expect("Submission id not found").into();
                submission.comments
            }
            PostType::Attestation => {
                let attestation: Attestation =
                    self.attestations.get(post_id).expect("Attestation id not found").into();
                attestation.comments
            }
            PostType::Sponsorship => {
                let sponsorship: Sponsorship =
                    self.sponsorships.get(post_id).expect("Sponsorship id not found").into();
                sponsorship.comments
            }
            PostType::Comment => {
                let comment: Comment =
                    self.comments.get(post_id).expect("Comment id not found").into();
                comment.comments
            }
        };
        comment_ids.iter().map(|id| self.comments.get(*id).unwrap().into()).collect()
    }

    pub fn get_comment(&self, comment_id: CommentId) -> Comment {
        self.comments.get(comment_id).unwrap().into()
    }
}
