use near_sdk::{env, near_bindgen, PromiseOrValue, AccountId, Promise, Balance, BorshStorageKey, PanicOnDefault, Timestamp};
use near_sdk::collections::{LookupMap, Vector};
use near_sdk::borsh::{self, BorshSerialize, BorshDeserialize};
use near_sdk::serde::{Serialize, Deserialize};

near_sdk::setup_alloc!();

type AttestationId = u64;
type AttesterId = AccountId;


#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Ideas,
    Submissions,
    Attestations,
    Sponsorships,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Attestation {
    attester: AccountId,
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
    reviewer_id: AccountId,
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
    reviewer_id: AccountId,
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
            reviewer_id: idea.reviewer_id,
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
    pub default_reviewer_id: AccountId,
    pub ideas: Vector<Idea>,
    pub submissions: LookupMap<u64, Vec<Submission>>,
    pub attestations: Vector<Attestation>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(default_reviewer_id: AccountId) -> Self {
        Self {
            default_reviewer_id,
            ideas: Vector::new(StorageKey::Ideas),
            submissions: LookupMap::new(StorageKey::Submissions),
            attestations: Vector::new(StorageKey::Attestations)
        }
    }

    pub fn set_default_reviewer(&mut self, reviewer_id: AccountId) {
        self.default_reviewer_id = reviewer_id;
    }

    pub fn get_idea(&self, idea_id: u64) -> IdeaOutput {
        IdeaOutput::from(idea_id, self.ideas.get(idea_id).unwrap())
    }

    pub fn get_num_ideas(&self) -> u64 {
        self.ideas.len()
    }

    pub fn get_submissions(&self, idea_id: u64) -> Vec<Submission> {
        self.submissions.get(&idea_id).unwrap_or(vec![])
    }

    pub fn get_ideas(&self, from_index: Option<u64>, limit: Option<u64>) -> Vec<IdeaOutput> {
        let from_index = from_index.unwrap_or(0);
        let limit = limit.unwrap_or(self.ideas.len());
        (from_index..std::cmp::min(from_index + limit, self.ideas.len())).map(
            |idea_id| IdeaOutput::from(idea_id, self.ideas.get(idea_id).unwrap())).collect()
    }

    #[payable]
    pub fn add_idea(&mut self, name: String, description: String, reviewer_id: Option<AccountId>) {
        self.ideas.push(&Idea {
            name,
            description,
            amount: env::attached_deposit(),
            submitter_id: env::predecessor_account_id(),
            reviewer_id: reviewer_id.unwrap_or(self.default_reviewer_id.clone()),
            status: IdeaStatus::Open,
            timestamp: env::block_timestamp(),
        })
    }

    pub fn add_submission(&mut self, submission: SubmissionInput) {
        let idea_id = submission.idea_id;
        let mut submissions = self.submissions.get(&idea_id).unwrap_or_default();
        submissions.push(Submission {
            idea_id: submission.idea_id,
            account_id: env::predecessor_account_id(),
            description: submission.description,
            timestamp: env::block_timestamp(),
            attestations: vec![]
        });
        self.submissions.insert(&idea_id, &submissions);
    }

    pub fn reassign_reviewer(&mut self, idea_id: u64, reviewer_id: AccountId) {
        let mut idea = self.ideas.get(idea_id).unwrap();
        assert_eq!(env::predecessor_account_id(), idea.reviewer_id, "Only current reviewer can reassign");
        idea.reviewer_id = reviewer_id;
        self.ideas.replace(idea_id, &idea);
    }

    #[payable]
    pub fn donate(&mut self, idea_id: u64) {
        let mut idea = self.ideas.get(idea_id).unwrap();
        idea.amount += env::attached_deposit();
        self.ideas.replace(idea_id, &idea);
    }

    pub fn review(&mut self, idea_id: u64, status: IdeaStatus) -> PromiseOrValue<()> {
        let mut idea = self.ideas.get(idea_id).unwrap();
        assert_eq!(env::predecessor_account_id(), idea.reviewer_id, "Only current reviewer can review");
        assert_eq!(idea.status, IdeaStatus::Open, "Idea must be open to change status");
        let submissions = self.submissions.get(&idea_id).unwrap_or_default();
        let ret = match &status {
            IdeaStatus::Done(submission_id) => {
                PromiseOrValue::Promise(Promise::new(submissions[*submission_id].account_id.clone()).transfer(idea.amount))
            }
            _ => PromiseOrValue::Value(())
        };
        idea.status = status;
        self.ideas.replace(idea_id, &idea);
        ret
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
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
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
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}
