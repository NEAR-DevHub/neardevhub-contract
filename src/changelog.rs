use crate::*;

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
            self.change_log.swap_remove(0);
        }
        self.change_log.push(&new_log);
    }
}
