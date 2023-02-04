use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{HashMap, HashSet};

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct RulesList {
    #[serde(flatten)]
    pub rules: HashMap<Rule, VersionedRuleMetadata>,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct RuleMetadata {
    pub description: String,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "rule_metadata_version")]
pub enum VersionedRuleMetadata {
    V0(RuleMetadata),
}

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialOrd, PartialEq, Ord, Eq, Hash)]
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
        let full_str: String = Deserialize::deserialize(deserializer)?;
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
