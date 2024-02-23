use crate::str_serializers::*;
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, NearSchema, Timestamp};
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Ord, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct Like {
    pub author_id: AccountId,
    #[serde(
        serialize_with = "u64_dec_format::serialize",
        deserialize_with = "u64_dec_format::deserialize"
    )]
    pub timestamp: Timestamp,
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
