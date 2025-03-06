use crate::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(BorshDeserialize, BorshSerialize, Clone, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ChangeLog {
    pub block_id: u64,
    pub block_timestamp: u64,
    pub change_log_type: ChangeLogType,
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum ChangeLogType {
    Proposal(ProposalId),
    RFP(RFPId),
}

// The changelog will be a FIFO queue of changes with the length of 50.
impl Contract {
    pub fn add_change_log(&mut self, change_log_type: ChangeLogType) {
        let new_log = ChangeLog {
            block_id: env::block_height(),
            block_timestamp: env::block_timestamp(),
            change_log_type,
        };
        if self.change_log.len() >= 50 {
            self.change_log.pop_front();
        }
        self.change_log.push_back(new_log);
    }
}

// TODO remove this if not necessary
#[derive(BorshDeserialize, BorshSerialize, Clone, Default, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ChangeLogQueue(pub VecDeque<ChangeLog>);

// TODO remove this
// Add methods to delegate to the inner VecDeque
impl ChangeLogQueue {
    pub fn new() -> Self {
        Self(VecDeque::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn pop_front(&mut self) -> Option<ChangeLog> {
        self.0.pop_front()
    }

    pub fn push_back(&mut self, value: ChangeLog) {
        self.0.push_back(value)
    }

    pub fn iter(&self) -> impl Iterator<Item = &ChangeLog> {
        self.0.iter()
    }
}
