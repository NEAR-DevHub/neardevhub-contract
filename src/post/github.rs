use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct GithubV0 {
    pub github_link: String,
    pub name: String,
    pub description: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "github_version")]
pub enum VersionedGithub {
    V0(GithubV0),
}

impl From<GithubV0> for VersionedGithub {
    fn from(v0: GithubV0) -> Self {
        VersionedGithub::V0(v0)
    }
}

impl From<VersionedGithub> for GithubV0 {
    fn from(vg: VersionedGithub) -> Self {
        match vg {
            VersionedGithub::V0(v0) => v0,
        }
    }
}
