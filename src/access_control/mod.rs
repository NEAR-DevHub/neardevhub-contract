use crate::access_control::members::{Member, MembersList, VersionedMemberMetadata};
use crate::access_control::rules::{Rule, RulesList};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub mod members;
pub mod rules;

#[derive(BorshSerialize, BorshDeserialize, Deserialize, Serialize, Clone, Default)]
#[serde(crate = "near_sdk::serde")]
pub struct AccessControl {
    pub rules_list: RulesList,
    pub members_list: MembersList,
}

use crate::*;
use near_sdk::near_bindgen;

#[near_bindgen]
impl Contract {
    pub fn get_access_control_info(&self) -> &AccessControl {
        &self.access_control
    }

    pub fn is_restricted_label(&self, label: &String) -> bool {
        self.access_control.rules_list.is_restricted(label)
    }

    pub fn find_restricted_labels(&self, labels: Vec<String>) -> HashSet<String> {
        self.access_control.rules_list.find_restricted(labels)
    }

    pub fn set_restricted_rules(&mut self, rules: RulesList) {
        self.access_control.rules_list.set_restricted(rules)
    }

    pub fn unset_restricted_rules(&mut self, rules: Vec<Rule>) {
        self.access_control.rules_list.unset_restricted(rules)
    }

    pub fn get_root_members(&self) -> HashMap<Member, VersionedMemberMetadata> {
        self.access_control.members_list.get_root_members()
    }

    pub fn add_member(&mut self, member: Member, metadata: VersionedMemberMetadata) {
        self.access_control.members_list.add_member(member, metadata)
    }

    pub fn remove_member(&mut self, member: &Member) {
        self.access_control.members_list.remove_member(member)
    }

    pub fn edit_member(&mut self, member: Member, metadata: VersionedMemberMetadata) {
        self.access_control.members_list.edit_member(member, metadata)
    }
}
