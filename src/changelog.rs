use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::NearSchema;

use crate::*;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct ChangeLog {
    pub block_id: u64,
    pub changed_object_id: u32,
    pub change_log_type: ChangeLogType,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, NearSchema)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub enum ChangeLogType {
    Proposal(ProposalId),
    RFP(RFPId),
}

// The changelog will be a FIFO queue of changes with the length of 50.
impl Contract {
    pub fn add_change_log(&mut self, new_log: ChangeLog) {
        if self.change_log.len() >= 50 {
            for i in 1..self.change_log.len() {
                let log = self.change_log.get(i).unwrap();
                self.change_log.replace(i - 1, &log);
            }
            self.change_log.replace(49, &new_log);
        } else {
            self.change_log.push(&new_log);
        }
    }
}
