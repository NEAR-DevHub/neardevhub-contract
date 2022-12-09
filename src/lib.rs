use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::Vector;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, Balance, BorshStorageKey, PanicOnDefault, Timestamp};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

near_sdk::setup_alloc!();

type IdeaId = u64;
type AttestationId = u64;
type SubmissionId = u64;
type SponsorshipId = u64;
type CommentId = u64;

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Ideas,
    Submissions,
    Attestations,
    Sponsorships,
    Comments,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum PostType {
    Idea,
    Submission,
    Attestation,
    Sponsorship,
    Comment,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum PostStatus {
    Open,
    Closed { reason: String },
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Like {
    author_id: AccountId,
    #[serde(with = "u64_dec_format")]
    timestamp: Timestamp,
}

impl Hash for Like {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.author_id.hash(state)
    }
}

impl PartialEq for Like {
    fn eq(&self, other: &Self) -> bool {
        self.author_id.eq(&other.author_id)
    }
}

impl PartialOrd for Like {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.author_id.partial_cmp(&other.author_id)
    }
}

impl Eq for Like {}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Comment {
    author_id: AccountId,
    #[serde(with = "u64_dec_format")]
    timestamp: Timestamp,
    description: String,
    likes: HashSet<Like>,
    comments: Vec<CommentId>,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum VersionedComment {
    V0(Comment),
}

impl From<VersionedComment> for Comment {
    fn from(vc: VersionedComment) -> Self {
        match vc {
            VersionedComment::V0(v0) => v0,
        }
    }
}

impl From<Comment> for VersionedComment {
    fn from(c: Comment) -> Self {
        VersionedComment::V0(c)
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum SponsorshipToken {
    Near,
    NEP141 { address: AccountId },
}

impl FromStr for SponsorshipToken {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "near" => Ok(Self::Near),
            _ => Ok(Self::NEP141 { address: s.to_string() }),
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Sponsorship {
    // Common fields
    id: SponsorshipId,
    name: String,
    description: String,
    author_id: AccountId,
    #[serde(with = "u64_dec_format")]
    timestamp: Timestamp,
    status: PostStatus,
    likes: HashSet<Like>,
    comments: Vec<CommentId>,

    // Specific fields
    #[serde(with = "u64_dec_format")]
    submission_id: IdeaId,
    sponsorship_token: SponsorshipToken,
    amount: Balance,
    supervisor: AccountId,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum VersionedSponsorship {
    V0(Sponsorship),
}

impl From<VersionedSponsorship> for Sponsorship {
    fn from(vs: VersionedSponsorship) -> Self {
        match vs {
            VersionedSponsorship::V0(v0) => v0,
        }
    }
}

impl From<Sponsorship> for VersionedSponsorship {
    fn from(s: Sponsorship) -> Self {
        VersionedSponsorship::V0(s)
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Attestation {
    // Common fields
    id: AttestationId,
    name: String,
    description: String,
    author_id: AccountId,
    #[serde(with = "u64_dec_format")]
    timestamp: Timestamp,
    status: PostStatus,
    likes: HashSet<Like>,
    comments: Vec<CommentId>,

    //Specific fields
    #[serde(with = "u64_dec_format")]
    submission_id: SubmissionId,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum VersionedAttestation {
    V0(Attestation),
}

impl From<VersionedAttestation> for Attestation {
    fn from(va: VersionedAttestation) -> Self {
        match va {
            VersionedAttestation::V0(v0) => v0,
        }
    }
}

impl From<Attestation> for VersionedAttestation {
    fn from(a: Attestation) -> Self {
        VersionedAttestation::V0(a)
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Submission {
    // Common fields
    id: SubmissionId,
    name: String,
    description: String,
    author_id: AccountId,
    #[serde(with = "u64_dec_format")]
    timestamp: Timestamp,
    status: PostStatus,
    likes: HashSet<Like>,
    comments: Vec<CommentId>,

    // Specific fields
    #[serde(with = "u64_dec_format")]
    idea_id: u64,
    attestations: Vec<AttestationId>,
    sponsorships: Vec<SponsorshipId>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum VersionedSubmission {
    V0(Submission),
}

impl From<VersionedSubmission> for Submission {
    fn from(vs: VersionedSubmission) -> Self {
        match vs {
            VersionedSubmission::V0(v0) => v0,
        }
    }
}

impl From<Submission> for VersionedSubmission {
    fn from(s: Submission) -> Self {
        VersionedSubmission::V0(s)
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Idea {
    // Common Fields
    id: IdeaId,
    name: String,
    description: String,
    author_id: AccountId,
    timestamp: Timestamp,
    status: PostStatus,
    likes: HashSet<Like>,
    comments: Vec<CommentId>,

    // Specific fields
    submissions: Vec<SubmissionId>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum VersionedIdea {
    V0(Idea),
}

impl From<VersionedIdea> for Idea {
    fn from(vi: VersionedIdea) -> Self {
        match vi {
            VersionedIdea::V0(v0) => v0,
        }
    }
}

impl From<Idea> for VersionedIdea {
    fn from(idea: Idea) -> Self {
        VersionedIdea::V0(idea)
    }
}

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
    /// This code was called only once to upgrade the contract to contain comments.
    pub fn initiate_comments() {
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

    /// Clear all of the state.
    pub fn root_purge(&mut self) {
        assert_eq!(
            env::current_account_id(),
            env::predecessor_account_id(),
            "Can only be called by the account itself"
        );
        self.ideas.clear();
        self.submissions.clear();
        self.sponsorships.clear();
        self.attestations.clear();
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
        let comment = Comment {
            author_id: env::predecessor_account_id(),
            timestamp: env::block_timestamp(),
            description,
            likes: Default::default(),
            comments: vec![],
        };
        let comment_id = self.comments.len();
        self.comments.push(&comment.into());
        match post_type {
            PostType::Idea => {
                let mut idea: Idea = self.ideas.get(post_id).expect("Idea id not found").into();
                idea.comments.push(comment_id);
                self.ideas.replace(post_id, &idea.into());
            }
            PostType::Submission => {
                let mut submission: Submission =
                    self.submissions.get(post_id).expect("Submission id not found").into();
                submission.comments.push(comment_id);
                self.submissions.replace(post_id, &submission.into());
            }
            PostType::Attestation => {
                let mut attestation: Attestation =
                    self.attestations.get(post_id).expect("Attestation id not found").into();
                attestation.comments.push(comment_id);
                self.attestations.replace(post_id, &attestation.into());
            }
            PostType::Sponsorship => {
                let mut sponsorship: Sponsorship =
                    self.sponsorships.get(post_id).expect("Sponsorship id not found").into();
                sponsorship.comments.push(comment_id);
                self.sponsorships.replace(post_id, &sponsorship.into());
            }
            PostType::Comment => {
                let mut comment: Comment =
                    self.comments.get(post_id).expect("Comment id not found").into();
                comment.comments.push(comment_id);
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
}

pub mod u128_dec_format {
    use near_sdk::serde::de;
    use near_sdk::serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(num: &u128, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&num.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u128, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?.parse().map_err(de::Error::custom)
    }
}

pub mod u64_dec_format {
    use near_sdk::serde::de;
    use near_sdk::serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(num: &u64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&num.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?.parse().map_err(de::Error::custom)
    }
}
