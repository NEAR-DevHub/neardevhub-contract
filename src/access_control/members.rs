use crate::access_control::rules::Rule;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub enum Member {
    Account(String),
    Team(String),
}

/// JSON string representation prefix of `Member::Team` variant.
const TEAM: &str = "team:";

impl ToString for Member {
    fn to_string(&self) -> String {
        match self {
            Member::Account(s) => s.to_string(),
            Member::Team(s) => format!("{}{}", TEAM, s).to_string(),
        }
    }
}

impl Serialize for Member {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serialize::serialize(&self.to_string(), serializer)
    }
}

impl<'de> Deserialize<'de> for Member {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let full_str: String = Deserialize::deserialize(deserializer)?;
        if let Some(s) = full_str.strip_prefix(TEAM) {
            Ok(Member::Team(s.to_string()))
        } else {
            Ok(Member::Account(full_str))
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct MemberMetadata {
    description: String,
    children: HashSet<Member>,
    parents: HashSet<Member>,
    permissions: HashMap<Rule, HashSet<ActionType>>,
}

#[derive(
    BorshSerialize,
    BorshDeserialize,
    Serialize,
    Deserialize,
    Clone,
    PartialOrd,
    PartialEq,
    Ord,
    Eq,
    Hash,
)]
#[serde(crate = "near_sdk::serde")]
pub enum ActionType {
    /// Can edit posts that have these labels.
    EditPost,
    /// Can add/remove labels that fall under these rules.
    UseLabels,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "member_metadata_version")]
pub enum VersionedMemberMetadata {
    V0(MemberMetadata),
}

impl VersionedMemberMetadata {
    pub fn last_version(&self) -> MemberMetadata {
        match self {
            VersionedMemberMetadata::V0(v0) => v0.clone(),
        }
    }
}

impl From<MemberMetadata> for VersionedMemberMetadata {
    fn from(m: MemberMetadata) -> Self {
        VersionedMemberMetadata::V0(m)
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct MembersList {
    #[serde(flatten)]
    pub members: HashMap<Member, VersionedMemberMetadata>,
}

impl MembersList {
    /// Get members that do not belong to any team.
    pub fn get_root_members(self) -> HashMap<Member, VersionedMemberMetadata> {
        self.members.into_iter().filter(|(_, v)| v.last_version().parents.is_empty()).collect()
    }

    /// Whether given account has special permissions for a post with the given labels.
    pub fn check_permissions(&self, account: String, labels: Vec<String>) -> HashSet<ActionType> {
        let mut stack = HashSet::new();
        stack.insert(Member::Account(account));

        let mut res = HashSet::new();
        while let Some(member) = stack.iter().next().cloned() {
            stack.remove(&member);

            let metadata = self
                .members
                .get(&member)
                .unwrap_or_else(|| panic!("Metadata not found for {:#?}", member))
                .last_version();

            for (rule, permissions) in metadata.permissions.iter() {
                if match rule {
                    Rule::ExactMatch(rule) => {
                        // `.find` requires mutable argument.
                        labels.iter().filter(|label| rule == *label).next().is_some()
                    }
                    Rule::StartsWith(rule) => {
                        // `.find` requires mutable argument.
                        labels.iter().filter(|label| label.starts_with(rule)).next().is_some()
                    }
                } {
                    for p in permissions {
                        res.insert(p.clone());
                    }
                }
            }

            for add_member in metadata.parents.iter() {
                stack.insert(add_member.clone());
            }
        }
        res
    }

    pub fn add_member(&mut self, member: Member, metadata: VersionedMemberMetadata) {
        assert!(
            self.members.insert(member.clone(), metadata.clone()).is_none(),
            "Member already exists"
        );

        // Update child members that this member is a parent of.
        for child in &metadata.last_version().children {
            match self.members.entry(child.clone()) {
                Entry::Occupied(mut occ) => {
                    let MemberMetadata { description, children, mut parents, permissions } =
                        occ.get().last_version();
                    assert!(parents.insert(member.clone()), "Child already had this parent");
                    let new_child = MemberMetadata { description, children, parents, permissions };
                    occ.insert(new_child.into());
                }
                Entry::Vacant(_) => {
                    panic!("Member declares a child {:#?} that does not exist", child)
                }
            }
        }

        // Update parent members that this member is now a child of.
        for parent in &metadata.last_version().parents {
            match self.members.entry(parent.clone()) {
                Entry::Occupied(mut occ) => {
                    let MemberMetadata { description, mut children, parents, permissions } =
                        occ.get().last_version();
                    assert!(children.insert(member.clone()), "Parent already had this child");
                    let new_parent = MemberMetadata { description, children, parents, permissions };
                    occ.insert(new_parent.into());
                }
                Entry::Vacant(_) => {
                    panic!("Member declares a parent {:#?} that does not exist", parent)
                }
            }
        }
    }

    pub fn remove_member(&mut self, member: &Member) {
        let metadata = self.members.remove(member).expect("Member does not exist");

        // Update child members that this member is not a parent of anymore.
        for child in &metadata.last_version().children {
            match self.members.entry(child.clone()) {
                Entry::Occupied(mut occ) => {
                    let MemberMetadata { description, children, mut parents, permissions } =
                        occ.get().last_version();
                    assert!(parents.remove(member), "Child did not have this parent.");
                    let new_child = MemberMetadata { description, children, parents, permissions };
                    occ.insert(new_child.into());
                }
                Entry::Vacant(_) => {
                    panic!("Member declares a child {:#?} that does not exist", child)
                }
            }
        }

        // Update parent members that this member is not a child of anymore.
        for parent in &metadata.last_version().parents {
            match self.members.entry(parent.clone()) {
                Entry::Occupied(mut occ) => {
                    let MemberMetadata { description, mut children, parents, permissions } =
                        occ.get().last_version();
                    assert!(children.remove(member), "Parent did not have this child.");
                    let new_parent = MemberMetadata { description, children, parents, permissions };
                    occ.insert(new_parent.into());
                }
                Entry::Vacant(_) => {
                    panic!("Member declares a parent {:#?} that does not exist", parent)
                }
            }
        }
    }

    pub fn edit_member(&mut self, member: Member, metadata: VersionedMemberMetadata) {
        self.remove_member(&member);
        self.add_member(member, metadata);
    }
}
