use crate::access_control::rules::Rule;
use near_sdk::{near, AccountId};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};

#[near(serializers=[borsh, json])]
#[derive(Clone, Debug, PartialOrd, PartialEq, Ord, Eq, Hash)]
#[serde(from = "String")]
#[serde(into = "String")]
pub enum Member {
    /// NEAR account names do not allow `:` character so this structure cannot be abused.
    Account(AccountId),
    Team(String),
}

/// JSON string representation prefix of `Member::Team` variant.
const TEAM: &str = "team:";

impl From<String> for Member {
    fn from(full_str: String) -> Self {
        if let Some(s) = full_str.strip_prefix(TEAM) {
            Member::Team(s.to_string())
        } else {
            Member::Account(full_str.parse().unwrap())
        }
    }
}

impl Into<String> for Member {
    fn into(self) -> String {
        match self {
            Member::Account(s) => s.to_string(),
            Member::Team(s) => format!("{}{}", TEAM, s).to_string(),
        }
    }
}

#[near(serializers=[borsh, json])]
#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct MemberMetadata {
    pub description: String,
    pub permissions: HashMap<Rule, HashSet<ActionType>>,
    pub children: HashSet<Member>,
    pub parents: HashSet<Member>,
}

#[near(serializers=[borsh, json])]
#[derive(Clone, PartialOrd, PartialEq, Ord, Eq, Hash, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum ActionType {
    /// Can edit posts that have these labels.
    EditPost,
    /// Can add/remove labels that fall under these rules.
    UseLabels,
}

#[near(serializers=[borsh, json])]
#[derive(Clone, Debug, Eq, PartialEq)]
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

#[near(serializers=[borsh, json])]
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct MembersList {
    #[serde(flatten)]
    pub members: HashMap<Member, VersionedMemberMetadata>,
}

impl MembersList {
    /// Get members that do not belong to any team.
    pub fn get_root_members(&self) -> HashMap<Member, VersionedMemberMetadata> {
        self.members
            .iter()
            .filter(|(_, v)| v.last_version().parents.is_empty())
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Whether given account has special permissions for a post with the given labels.
    /// Labels are restricted labels.
    pub fn check_permissions(
        &self,
        account: AccountId,
        labels: Vec<String>,
    ) -> HashSet<ActionType> {
        let member_account = Member::Account(account);
        if !self.members.contains_key(&member_account) {
            return HashSet::new();
        }

        let mut stack = HashSet::new();
        stack.insert(member_account);

        let mut permissions = HashSet::new();
        while let Some(member) = stack.iter().next().cloned() {
            stack.remove(&member);

            let metadata = self
                .members
                .get(&member)
                .unwrap_or_else(|| panic!("Metadata not found for {:#?}", member))
                .last_version();

            for (member_rule, member_permissions) in metadata.permissions {
                if member_rule.applies_to_any(&labels) {
                    permissions.extend(member_permissions);
                }
            }

            stack.extend(metadata.parents);
        }
        permissions
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

    pub fn get_moderators(&self) -> HashSet<Member> {
        self.members
            .get(&Member::Team("moderators".to_string()))
            .map(|team| team.last_version().children)
            .unwrap_or(HashSet::new())
    }
}

#[cfg(test)]
mod tests {
    use crate::access_control::members::{
        ActionType, Member, MemberMetadata, MembersList, VersionedMemberMetadata,
    };
    use crate::access_control::rules::Rule;
    use near_sdk::serde_json;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn member_serialization() {
        let member = Member::Account("alice.near".parse().unwrap());
        assert_eq!(serde_json::to_value(&member).unwrap(), serde_json::json!("alice.near"));

        let member = Member::Team("funding".to_string());
        assert_eq!(serde_json::to_value(&member).unwrap(), serde_json::json!("team:funding"));
    }

    #[test]
    fn member_deserialization() {
        let member: Member = serde_json::from_str(r#""alice.near""#).unwrap();
        assert_eq!(member, Member::Account("alice.near".parse().unwrap()));

        let member: Member = serde_json::from_str(r#""team:funding""#).unwrap();
        assert_eq!(member, Member::Team("funding".to_string()));
    }

    fn root_member() -> (Member, VersionedMemberMetadata) {
        (
            Member::Account("devgovgigs.near".parse().unwrap()),
            MemberMetadata {
                description: "Main account can do anything".to_string(),
                permissions: HashMap::from([
                    (
                        Rule::StartsWith("wg-".to_string()),
                        HashSet::from([ActionType::EditPost, ActionType::UseLabels]),
                    ),
                    (
                        Rule::StartsWith("funding".to_string()),
                        HashSet::from([ActionType::EditPost, ActionType::UseLabels]),
                    ),
                    (
                        Rule::StartsWith("mnw".to_string()),
                        HashSet::from([ActionType::EditPost, ActionType::UseLabels]),
                    ),
                ]),
                ..Default::default()
            }
            .into(),
        )
    }

    fn moderator_member(name: &str) -> (Member, VersionedMemberMetadata) {
        (
            Member::Account(name.parse().unwrap()),
            MemberMetadata {
                description: format!("{} inherits everything from moderator group.", name)
                    .to_string(),
                parents: HashSet::from([Member::Team("moderators".to_string())]),
                ..Default::default()
            }
            .into(),
        )
    }

    fn moderators() -> (Member, VersionedMemberMetadata) {
        (
            Member::Team("moderators".to_string()),
            MemberMetadata {
                description: "Moderators can do anything except funding posts.".to_string(),
                permissions: HashMap::from([
                    (
                        Rule::StartsWith("wg-".to_string()),
                        HashSet::from([ActionType::EditPost, ActionType::UseLabels]),
                    ),
                    (
                        Rule::StartsWith("mnw".to_string()),
                        HashSet::from([ActionType::EditPost, ActionType::UseLabels]),
                    ),
                ]),
                children: HashSet::from([
                    Member::Account("ori.near".parse().unwrap()),
                    Member::Account("max.near".parse().unwrap()),
                    Member::Account("vlad.near".parse().unwrap()),
                ]),
                ..Default::default()
            }
            .into(),
        )
    }

    fn create_list() -> MembersList {
        MembersList {
            members: HashMap::from([
                moderators(),
                root_member(),
                moderator_member("ori.near"),
                moderator_member("max.near"),
                moderator_member("vlad.near"),
            ]),
        }
    }

    #[test]
    fn get_root_members() {
        let list = create_list();
        let root_members: HashSet<_> = list.get_root_members().keys().cloned().collect();
        assert_eq!(
            root_members,
            HashSet::from([
                Member::Team("moderators".to_string()),
                Member::Account("devgovgigs.near".parse().unwrap())
            ])
        );
    }

    #[test]
    fn check_permissions() {
        let list = create_list();
        let actual = list.check_permissions(
            "max.near".parse().unwrap(),
            vec!["wg-protocol".to_string(), "funding-requested".to_string()],
        );
        assert_eq!(
            actual,
            serde_json::from_value::<HashSet<ActionType>>(serde_json::json!([
                "edit-post",
                "use-labels"
            ]))
            .unwrap()
        );

        let actual = list
            .check_permissions("max.near".parse().unwrap(), vec!["funding-requested".to_string()]);
        assert!(actual.is_empty());
    }

    #[test]
    fn check_permissions_rules_any() {
        let mut list = create_list();
        let given_permissions = HashSet::from([ActionType::EditPost, ActionType::UseLabels]);
        list.add_member(
            Member::Account("thomasguntenaar.near".parse().unwrap()),
            MemberMetadata {
                description: "Account has `Any` rules without labels".to_string(),
                permissions: HashMap::from([(Rule::Any(), given_permissions.clone())]),
                ..Default::default()
            }
            .into(),
        );
        // Without labels
        assert_eq!(
            list.check_permissions("thomasguntenaar.near".parse().unwrap(), vec![]),
            given_permissions,
        );

        // With labels
        assert_eq!(
            list.check_permissions(
                "thomasguntenaar.near".parse().unwrap(),
                vec!["funding-requested".to_string()]
            ),
            given_permissions,
        )
    }

    #[test]
    fn check_permissions_of_not_member() {
        let list = create_list();
        let actual: HashSet<ActionType> = list.check_permissions(
            "random.near".parse().unwrap(),
            vec!["wg-protocol".to_string(), "funding-requested".to_string()],
        );
        assert!(actual.is_empty());
    }

    #[test]
    fn add_remove_member() {
        let mut list = create_list();
        list.add_member(
            Member::Account("bob.near".parse().unwrap()),
            MemberMetadata {
                parents: HashSet::from([Member::Team("moderators".to_string())]),
                ..Default::default()
            }
            .into(),
        );
        list.remove_member(&Member::Account("bob.near".parse().unwrap()));
        assert_eq!(list, create_list());
    }
}
