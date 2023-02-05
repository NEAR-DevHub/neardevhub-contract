use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{HashMap, HashSet};

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct RulesList {
    #[serde(flatten)]
    pub rules: HashMap<Rule, VersionedRuleMetadata>,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct RuleMetadata {
    pub description: String,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "rule_metadata_version")]
pub enum VersionedRuleMetadata {
    V0(RuleMetadata),
}

impl From<RuleMetadata> for VersionedRuleMetadata {
    fn from(rm: RuleMetadata) -> Self {
        VersionedRuleMetadata::V0(rm)
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialOrd, PartialEq, Ord, Eq, Hash, Debug)]
pub enum Rule {
    ExactMatch(String),
    StartsWith(String),
}

/// JSON string representation prefix of Rule::StartsWith variant.
const STARTS_WITH: &str = "starts-with:";

impl ToString for Rule {
    fn to_string(&self) -> String {
        match self {
            Rule::ExactMatch(s) => s.to_string(),
            Rule::StartsWith(s) => format!("{}{}", STARTS_WITH, s).to_string(),
        }
    }
}

impl Serialize for Rule {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serialize::serialize(&self.to_string(), serializer)
    }
}

impl<'de> Deserialize<'de> for Rule {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let full_str = <String as Deserialize>::deserialize(deserializer)?;
        if let Some(s) = full_str.strip_prefix(STARTS_WITH) {
            Ok(Rule::StartsWith(s.to_string()))
        } else {
            Ok(Rule::ExactMatch(full_str))
        }
    }
}

impl Rule {
    /// Check if this rule applies to a label.
    pub fn applies(&self, label: &String) -> bool {
        match self {
            Rule::ExactMatch(rule) => rule == label,
            Rule::StartsWith(rule) => label.starts_with(rule),
        }
    }
}

impl RulesList {
    /// Is this a restricted label.
    pub fn is_restricted(&self, label: &String) -> bool {
        self.rules.keys().find(|key| key.applies(label)).is_some()
    }

    /// Get restricted labels out of this list.
    pub fn find_restricted(&self, ref labels: Vec<String>) -> HashSet<String> {
        self.rules
            .keys()
            .map(|key| match key {
                Rule::ExactMatch(rule) => {
                    labels.into_iter().filter(|label| rule == *label).collect::<Vec<_>>()
                }
                Rule::StartsWith(rule) => {
                    labels.into_iter().filter(|label| label.starts_with(rule)).collect::<Vec<_>>()
                }
            })
            .flatten()
            .cloned()
            .collect()
    }

    /// Set rules as restricted. Can be also used to override metadata on existing rules.
    pub fn set_restricted(&mut self, rules: Self) {
        for (rule, metadata) in rules.rules {
            self.rules.insert(rule, metadata);
        }
    }

    /// Unset rules as restricted.
    pub fn unset_restricted(&mut self, rules: Vec<Rule>) {
        for rule in rules {
            self.rules.remove(&rule);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::access_control::rules::{Rule, RuleMetadata, RulesList, VersionedRuleMetadata};
    use near_sdk::serde_json;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn rule_serialization() {
        let rule = Rule::ExactMatch("wg-protocol".to_string());
        assert_eq!(serde_json::to_value(&rule).unwrap(), serde_json::json!("wg-protocol"));

        let rule = Rule::StartsWith("funding".to_string());
        assert_eq!(serde_json::to_value(&rule).unwrap(), serde_json::json!("starts-with:funding"));
    }

    #[test]
    fn rule_deserialization() {
        let rule: Rule = serde_json::from_str(r#""wg-protocol""#).unwrap();
        assert_eq!(rule, Rule::ExactMatch("wg-protocol".to_string()));

        let rule: Rule = serde_json::from_str(r#""starts-with:funding""#).unwrap();
        assert_eq!(rule, Rule::StartsWith("funding".to_string()));
    }

    fn create_list() -> RulesList {
        RulesList {
            rules: HashMap::from([
                (
                    Rule::ExactMatch("wg-protocol".to_string()),
                    RuleMetadata { description: "For Protocol WG only".to_string() }.into(),
                ),
                (
                    Rule::ExactMatch("wg-tools".to_string()),
                    RuleMetadata { description: "For Tools WG only".to_string() }.into(),
                ),
                (
                    Rule::StartsWith("funding".to_string()),
                    RuleMetadata { description: "For funding team only".to_string() }.into(),
                ),
                (
                    Rule::StartsWith("mnw".to_string()),
                    RuleMetadata { description: "For Wallet WG only".to_string() }.into(),
                ),
            ]),
        }
    }

    #[test]
    fn rule_list_serialization_deserialization() {
        let list = create_list();

        let list_json = serde_json::json!(
            {
                "wg-protocol": { "description": "For Protocol WG only", "rule_metadata_version": "V0"},
                "wg-tools": {"description": "For Tools WG only", "rule_metadata_version": "V0" },
                "starts-with:funding": {"description": "For funding team only", "rule_metadata_version": "V0" },
                "starts-with:mnw": {"description": "For Wallet WG only", "rule_metadata_version": "V0" }
            }
        );
        assert_eq!(serde_json::to_value(list.clone()).unwrap(), list_json);
        assert_eq!(serde_json::from_value::<RulesList>(list_json).unwrap(), list);
    }

    #[test]
    fn is_restricted() {
        let list = create_list();
        assert!(list.is_restricted(&"wg-protocol".to_string()));
        assert!(list.is_restricted(&"wg-tools".to_string()));
        assert!(!list.is_restricted(&"wg-wallet".to_string()));
        assert!(list.is_restricted(&"funding".to_string()));
        assert!(list.is_restricted(&"fundingfoobar".to_string()));
        assert!(list.is_restricted(&"funding-requested".to_string()));
        assert!(!list.is_restricted(&"nofunding".to_string()));
        assert!(list.is_restricted(&"mnw".to_string()));
        assert!(list.is_restricted(&"mnw-approved".to_string()));
        assert!(!list.is_restricted(&"nomnw".to_string()));
    }

    #[test]
    fn find_restricted() {
        let list = create_list();
        let actual = list.find_restricted(vec![
            "wg-protocol".to_string(),
            "wg-tools".to_string(),
            "wg-wallet".to_string(),
            "funding".to_string(),
            "funding".to_string(),
            "fundingfoobar".to_string(),
            "fundingfoobar".to_string(),
            "funding-requested".to_string(),
            "nofunding".to_string(),
            "nofunding".to_string(),
            "mnw".to_string(),
            "mnw-approved".to_string(),
            "nomnw".to_string(),
        ]);
        let expected = HashSet::from([
            "wg-protocol".to_string(),
            "wg-tools".to_string(),
            "funding".to_string(),
            "fundingfoobar".to_string(),
            "funding-requested".to_string(),
            "mnw".to_string(),
            "mnw-approved".to_string(),
        ]);
        assert_eq!(actual, expected);
    }
}
