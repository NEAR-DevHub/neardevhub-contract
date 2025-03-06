use crate::*;
// use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
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
#[near(serializers=[borsh, json])]
pub struct ChangeLogQueue(pub VecDeque<ChangeLog>);

// #[serde(with = "vec_deque_as_vec")]
// mod vec_deque_as_vec {
//     use super::*;
//     use near_sdk::serde::{Deserialize, Deserializer, Serializer};

//     pub fn serialize<S, T>(deque: &VecDeque<T>, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//         T: near_sdk::serde::Serialize,
//     {
//         let vec: Vec<&T> = deque.iter().collect();
//         vec.serialize(serializer)
//     }

//     pub fn deserialize<'de, D, T>(deserializer: D) -> Result<VecDeque<T>, D::Error>
//     where
//         D: Deserializer<'de>,
//         T: Deserialize<'de>,
//     {
//         let vec = Vec::<T>::deserialize(deserializer)?;
//         Ok(VecDeque::from(vec))
//     }
// }

// impl BorshSerialize for ChangeLogQueue {
//     fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
//         let vec: Vec<_> = self.0.iter().cloned().collect();
//         borsh::BorshSerialize::serialize(&vec, writer)
//     }
// }

// impl BorshDeserialize for ChangeLogQueue {
//     fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
//         let vec: Vec<ChangeLog> = borsh::BorshDeserialize::deserialize(buf)?;
//         Ok(Self(VecDeque::from(vec)))
//     }

//     fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
//         let vec: Vec<ChangeLog> = borsh::BorshDeserialize::deserialize_reader(reader)?;
//         Ok(Self(VecDeque::from(vec)))
//     }
// }

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
