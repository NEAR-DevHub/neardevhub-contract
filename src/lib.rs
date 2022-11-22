use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, Vector};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, near_bindgen, AccountId, Balance, BorshStorageKey, PanicOnDefault, Promise,
    PromiseOrValue, Timestamp,
};

near_sdk::setup_alloc!();

type AttestationId = u64;
type AttesterId = AccountId;
type SubmissionId = u64;

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Ideas,
    Submissions,
    IdeaToSubmissions,
    Attestations,
    Sponsorships,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Attestation {
    attestation_id: AttestationId,
    attester: AccountId,
    timestamp: Timestamp,
    submission_id: SubmissionId,
    description: String,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AttestationInput {
    submission_id: SubmissionId,
    description: String,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum IdeaStatus {
    /// Open when idea is still not done.
    Open,
    /// If idea violates terms and conditions, it gets reported.
    Reported,
    /// Confirmed that idea is done by given submission id.
    Done(usize),
    /// Closed without payout. Either outlived or done outside of this payouts.
    Closed,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Idea {
    name: String,
    description: String,
    amount: Balance,
    submitter_id: AccountId,
    status: IdeaStatus,
    timestamp: Timestamp,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct IdeaOutput {
    #[serde(with = "u64_dec_format")]
    idea_id: u64,
    name: String,
    description: String,
    #[serde(with = "u128_dec_format")]
    amount: Balance,
    submitter_id: AccountId,
    status: IdeaStatus,
    #[serde(with = "u64_dec_format")]
    timestamp: Timestamp,
}

impl IdeaOutput {
    fn from(idea_id: u64, idea: Idea) -> IdeaOutput {
        IdeaOutput {
            idea_id,
            name: idea.name,
            description: idea.description,
            amount: idea.amount,
            submitter_id: idea.submitter_id,
            status: idea.status,
            timestamp: idea.timestamp,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Submission {
    #[serde(with = "u64_dec_format")]
    idea_id: u64,
    account_id: AccountId,
    description: String,
    #[serde(with = "u64_dec_format")]
    timestamp: Timestamp,
    attestations: Vec<AttestationId>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SubmissionInput {
    idea_id: u64,
    description: String,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub ideas: Vector<Idea>,
    pub submissions: Vector<Submission>,
    pub idea_to_submissions: LookupMap<u64, Vec<SubmissionId>>,
    pub attestations: Vector<Attestation>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {
            ideas: Vector::new(StorageKey::Ideas),
            submissions: Vector::new(StorageKey::Submissions),
            idea_to_submissions: LookupMap::new(StorageKey::IdeaToSubmissions),
            attestations: Vector::new(StorageKey::Attestations),
        }
    }

    pub fn get_idea(&self, idea_id: u64) -> IdeaOutput {
        IdeaOutput::from(idea_id, self.ideas.get(idea_id).unwrap())
    }

    pub fn get_num_ideas(&self) -> u64 {
        self.ideas.len()
    }

    pub fn get_submissions(&self, idea_id: u64) -> Vec<Submission> {
        let ids = self.idea_to_submissions.get(&idea_id).unwrap_or_default();
        ids.into_iter().map(|id| self.submissions.get(id).unwrap()).collect()
    }

    pub fn get_ideas(&self, from_index: Option<u64>, limit: Option<u64>) -> Vec<IdeaOutput> {
        let from_index = from_index.unwrap_or(0);
        let limit = limit.unwrap_or(self.ideas.len());
        (from_index..std::cmp::min(from_index + limit, self.ideas.len()))
            .map(|idea_id| IdeaOutput::from(idea_id, self.ideas.get(idea_id).unwrap()))
            .collect()
    }

    #[payable]
    pub fn add_idea(&mut self, name: String, description: String) {
        self.ideas.push(&Idea {
            name,
            description,
            amount: env::attached_deposit(),
            submitter_id: env::predecessor_account_id(),
            status: IdeaStatus::Open,
            timestamp: env::block_timestamp(),
        })
    }

    pub fn add_submission(&mut self, submission_inp: SubmissionInput) {
        let idea_id = submission_inp.idea_id;
        let mut submission_ids = self.idea_to_submissions.get(&idea_id).unwrap_or_default();
        submission_ids.push(self.submissions.len());
        self.idea_to_submissions.insert(&idea_id, &submission_ids);

        self.submissions.push(&Submission {
            idea_id: submission_inp.idea_id,
            account_id: env::predecessor_account_id(),
            description: submission_inp.description,
            timestamp: env::block_timestamp(),
            attestations: vec![],
        });
    }

    pub fn add_attestation(&mut self, attestation_inp: AttestationInput) {
        let mut submission =
            self.submissions.get(attestation_inp.submission_id).expect("Submission id not found");
        let new_attestation_id = self.attestations.len();
        submission.attestations.push(new_attestation_id);
        self.submissions.replace(attestation_inp.submission_id, &submission);

        self.attestations.push(&Attestation {
            attestation_id: new_attestation_id,
            attester: env::predecessor_account_id(),
            timestamp: env::block_timestamp(),
            submission_id: attestation_inp.submission_id,
            description: attestation_inp.description,
        });
    }
    pub fn get_attestations(&self, submission_id: SubmissionId) -> Vec<Attestation> {
        let submission = self.submissions.get(submission_id).expect("Submission id not found");
        submission.attestations.iter().map(|id| self.attestations.get(*id).unwrap()).collect()
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
