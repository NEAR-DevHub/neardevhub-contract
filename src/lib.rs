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
enum SponsorshipToken {
    Native,
    NEP141 { address: AccountId },
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Sponsorship {
    sponsorship_id: SponsorshipId,
    idea_id: IdeaId,
    sponsor: AccountId,
    timestamp: Timestamp,
    description: String,
    sponsorship_token: SponsorshipToken,
    amount: Balance,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SponsorshipInput {
    idea_id: IdeaId,
    sponsor: AccountId,
    description: String,
    sponsorship_token: SponsorshipToken,
    amount: Balance,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
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

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Idea {
    name: String,
    description: String,
    submitter_id: AccountId,
    timestamp: Timestamp,
    submissions: Vec<SubmissionId>,
    sponsorships: Vec<SponsorshipId>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct IdeaOutput {
    #[serde(with = "u64_dec_format")]
    idea_id: u64,
    name: String,
    description: String,
    submitter_id: AccountId,
    #[serde(with = "u64_dec_format")]
    timestamp: Timestamp,
}

impl IdeaOutput {
    fn from(idea_id: u64, idea: Idea) -> IdeaOutput {
        IdeaOutput {
            idea_id,
            name: idea.name,
            description: idea.description,
            submitter_id: idea.submitter_id,
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
    sponsorships: Vec<SponsorshipId>,
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

    pub fn add_idea(&mut self, name: String, description: String) {
        self.ideas.push(&Idea {
            name,
            description,
            submitter_id: env::predecessor_account_id(),
            timestamp: env::block_timestamp(),
            submissions: vec![],
            sponsorships: vec![],
        })
    }

    pub fn get_idea(&self, idea_id: IdeaId) -> IdeaOutput {
        IdeaOutput::from(idea_id, self.ideas.get(idea_id).unwrap())
    }

    pub fn get_num_ideas(&self) -> IdeaId {
        self.ideas.len()
    }

    pub fn get_ideas(&self, from_index: Option<IdeaId>, limit: Option<IdeaId>) -> Vec<IdeaOutput> {
        let from_index = from_index.unwrap_or(0);
        let limit = limit.unwrap_or(self.ideas.len());
        (from_index..std::cmp::min(from_index + limit, self.ideas.len()))
            .map(|idea_id| IdeaOutput::from(idea_id, self.ideas.get(idea_id).unwrap()))
            .collect()
    }

    pub fn add_submission(&mut self, submission_inp: SubmissionInput) {
        let idea_id = submission_inp.idea_id;
        let mut idea = self.ideas.get(idea_id).expect("Submission id not found");
        idea.submissions.push(self.submissions.len());
        self.ideas.replace(idea_id, &idea);

        self.submissions.push(&Submission {
            idea_id: submission_inp.idea_id,
            account_id: env::predecessor_account_id(),
            description: submission_inp.description,
            timestamp: env::block_timestamp(),
            attestations: vec![],
            sponsorships: vec![],
        });
    }

    pub fn get_submissions(&self, idea_id: IdeaId) -> Vec<Submission> {
        let idea = self.ideas.get(idea_id).expect("Idea id not found");
        idea.submissions.iter().map(|id| self.submissions.get(*id).unwrap()).collect()
    }

    pub fn add_attestation(&mut self, attestation_inp: AttestationInput) {
        let new_attestation_id = self.attestations.len();

        let mut submission =
            self.submissions.get(attestation_inp.submission_id).expect("Submission id not found");
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

    pub fn add_sponsorship(&mut self, sponsorship_inp: SponsorshipInput) {
        let new_sponsorship_id = self.sponsorships.len();

        let mut idea = self.ideas.get(sponsorship_inp.idea_id).expect("Idea id not found");
        idea.sponsorships.push(new_sponsorship_id);
        self.ideas.replace(sponsorship_inp.idea_id, &idea);

        self.sponsorships.push(&Sponsorship {
            sponsorship_id: new_sponsorship_id,
            idea_id: sponsorship_inp.idea_id,
            sponsor: sponsorship_inp.sponsor,
            timestamp: env::block_timestamp(),
            description: sponsorship_inp.description,
            sponsorship_token: sponsorship_inp.sponsorship_token,
            amount: sponsorship_inp.amount,
        });
    }

    pub fn get_sponsorships(&self, idea_id: IdeaId) -> Vec<Sponsorship> {
        let idea = self.ideas.get(idea_id).expect("Idea id not found");
        idea.sponsorships.iter().map(|id| self.sponsorships.get(*id).unwrap()).collect()
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
