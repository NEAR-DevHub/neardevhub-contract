use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::Vector;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, Balance, BorshStorageKey, PanicOnDefault, Timestamp};

near_sdk::setup_alloc!();

type IdeaId = u64;
type AttestationId = u64;
type SubmissionId = u64;
type SponsorshipId = u64;

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Ideas,
    Submissions,
    Attestations,
    Sponsorships,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum PostStatus {
    Open,
    Closed { reason: String },
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum SponsorshipToken {
    Native,
    NEP141 { address: AccountId },
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

    // Specific fields
    #[serde(with = "u64_dec_format")]
    submission_id: IdeaId,
    sponsorship_token: SponsorshipToken,
    amount: Balance,
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

    //Specific fields
    #[serde(with = "u64_dec_format")]
    submission_id: SubmissionId,
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

    // Specific fields
    #[serde(with = "u64_dec_format")]
    idea_id: u64,
    attestations: Vec<AttestationId>,
    sponsorships: Vec<SponsorshipId>,
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

    // Specific fields
    submissions: Vec<SubmissionId>,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub treasury: AccountId,
    pub ideas: Vector<Idea>,
    pub submissions: Vector<Submission>,
    pub attestations: Vector<Attestation>,
    pub sponsorships: Vector<Sponsorship>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(alternative_treasury: Option<AccountId>) -> Self {
        Self {
            treasury: alternative_treasury.unwrap_or(env::current_account_id()),
            ideas: Vector::new(StorageKey::Ideas),
            submissions: Vector::new(StorageKey::Submissions),
            attestations: Vector::new(StorageKey::Attestations),
            sponsorships: Vector::new(StorageKey::Sponsorships),
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
        self.ideas.push(&Idea {
            id,
            name,
            description,
            author_id: env::predecessor_account_id(),
            timestamp: env::block_timestamp(),
            status: PostStatus::Open,
            submissions: vec![],
        })
    }

    pub fn get_idea(&self, idea_id: IdeaId) -> Idea {
        self.ideas.get(idea_id).unwrap()
    }

    pub fn get_num_ideas(&self) -> IdeaId {
        self.ideas.len()
    }

    pub fn get_ideas(&self, from_index: Option<IdeaId>, limit: Option<IdeaId>) -> Vec<Idea> {
        let from_index = from_index.unwrap_or(0);
        let limit = limit.unwrap_or(self.ideas.len());
        (from_index..std::cmp::min(from_index + limit, self.ideas.len()))
            .map(|idea_id| self.ideas.get(idea_id).unwrap())
            .collect()
    }

    pub fn add_submission(&mut self, idea_id: IdeaId, name: String, description: String) {
        let id = self.submissions.len();

        let mut idea = self.ideas.get(idea_id).expect("Submission id not found");
        idea.submissions.push(id);
        self.ideas.replace(idea_id, &idea);

        self.submissions.push(&Submission {
            id,
            name,
            description,
            author_id: env::predecessor_account_id(),
            timestamp: env::block_timestamp(),
            status: PostStatus::Open,
            idea_id,
            attestations: vec![],
            sponsorships: vec![],
        });
    }

    pub fn get_submissions(&self, idea_id: IdeaId) -> Vec<Submission> {
        let idea = self.ideas.get(idea_id).expect("Idea id not found");
        idea.submissions.iter().map(|id| self.submissions.get(*id).unwrap()).collect()
    }

    pub fn add_attestation(
        &mut self,
        submission_id: SubmissionId,
        name: String,
        description: String,
    ) {
        let id = self.attestations.len();

        let mut submission = self.submissions.get(submission_id).expect("Submission id not found");
        submission.attestations.push(id);
        self.submissions.replace(submission_id, &submission);

        self.attestations.push(&Attestation {
            id,
            name,
            description,
            author_id: env::predecessor_account_id(),
            timestamp: env::block_timestamp(),
            status: PostStatus::Open,
            submission_id,
        });
    }

    pub fn get_attestations(&self, submission_id: SubmissionId) -> Vec<Attestation> {
        let submission = self.submissions.get(submission_id).expect("Submission id not found");
        submission.attestations.iter().map(|id| self.attestations.get(*id).unwrap()).collect()
    }

    pub fn add_sponsorship(
        &mut self,
        submission_id: SubmissionId,
        name: String,
        description: String,
        sponsorship_token: SponsorshipToken,
        amount: Balance,
    ) {
        let id = self.sponsorships.len();

        let mut submission = self.submissions.get(submission_id).expect("Submission id not found");
        submission.attestations.push(id);
        self.submissions.replace(submission_id, &submission);

        self.sponsorships.push(&Sponsorship {
            id,
            name,
            description,
            author_id: env::predecessor_account_id(),
            timestamp: env::block_timestamp(),
            status: PostStatus::Open,
            submission_id,
            sponsorship_token,
            amount,
        });
    }

    pub fn get_sponsorships(&self, submission_id: SubmissionId) -> Vec<Sponsorship> {
        let submission = self.submissions.get(submission_id).expect("Submission id not found");
        submission.sponsorships.iter().map(|id| self.sponsorships.get(*id).unwrap()).collect()
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
