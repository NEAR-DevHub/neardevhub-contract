use crate::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use std::collections::VecDeque;
#[derive(Clone)]
#[near(serializers=[borsh, json])]
pub struct ChangeLog {
    pub block_id: u64,
    pub block_timestamp: u64,
    pub change_log_type: ChangeLogType,
}

#[derive(Clone)]
#[near(serializers=[borsh, json])]
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

#[derive(Clone, Default)]
pub struct ChangeLogQueue(pub VecDeque<ChangeLog>);

impl BorshSerialize for ChangeLogQueue {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let vec: Vec<_> = self.0.iter().cloned().collect();
        borsh::BorshSerialize::serialize(&vec, writer)
    }
}

impl BorshDeserialize for ChangeLogQueue {
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let vec: Vec<ChangeLog> = borsh::BorshDeserialize::deserialize(buf)?;
        Ok(Self(VecDeque::from(vec)))
    }

    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let vec: Vec<ChangeLog> = borsh::BorshDeserialize::deserialize_reader(reader)?;
        Ok(Self(VecDeque::from(vec)))
    }
}

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
