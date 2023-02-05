use crate::access_control::members::MembersList;
use crate::access_control::rules::RulesList;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};

pub mod members;
pub mod rules;

#[derive(BorshSerialize, BorshDeserialize, Deserialize, Serialize, Clone, Default)]
#[serde(crate = "near_sdk::serde")]
pub struct AccessControl {
    pub rules_list: RulesList,
    pub members_list: MembersList,
}
