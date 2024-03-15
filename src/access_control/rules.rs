use near_sdk::near;
use std::collections::{HashMap, HashSet};

#[near(serializers=[borsh, json])]
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct RulesList {
    #[serde(flatten)]
    pub rules: HashMap<Rule, VersionedRuleMetadata>,
}

#[near(serializers=[borsh, json])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuleMetadata {
    pub description: String,
}

#[near(serializers=[borsh, json])]
#[derive(Clone, Debug, Eq, PartialEq)]
#[serde(tag = "rule_metadata_version")]
pub enum VersionedRuleMetadata {
    V0(RuleMetadata),
}

impl From<RuleMetadata> for VersionedRuleMetadata {
    fn from(rm: RuleMetadata) -> Self {
        VersionedRuleMetadata::V0(rm)
    }
}

#[near(serializers=[borsh, json])]
#[derive(Clone, PartialOrd, PartialEq, Ord, Eq, Hash, Debug)]
#[serde(from = "String", into = "String")]
pub enum Rule {
    /// Labels can be any string, but rules are created by the NEAR account owner of this contract,
    /// or small circle of moderators. So this code cannot be abused. Likely creating a label that
    /// mimics a rule makes this label only more restrictive, so there might be nothing to exploit.
    /// TODO: Add extra logic to prevent malicious rules creation by creating labels that mimic rules.
    ExactMatch(String),
    StartsWith(String),
    Any(),
}

/// JSON string representation prefix of Rule::StartsWith variant.
const STARTS_WITH: &str = "starts-with:";
const ANY: &str = "*";

impl From<String> for Rule {
    fn from(full_str: String) -> Self {
        if full_str == ANY {
            Rule::Any()
        } else if let Some(s) = full_str.strip_prefix(STARTS_WITH) {
            Rule::StartsWith(s.to_string())
        } else {
            Rule::ExactMatch(full_str)
        }
    }
}

impl Into<String> for Rule {
    fn into(self) -> String {
        match self {
            Rule::ExactMatch(s) => s.to_string(),
            Rule::StartsWith(s) => format!("{}{}", STARTS_WITH, s).to_string(),
            Rule::Any() => ANY.to_string(),
        }
    }
}

impl Rule {
    /// Check if this rule applies to a label.
    pub fn applies(&self, label: &str) -> bool {
        match self {
            Rule::ExactMatch(rule) => label == rule,
            Rule::StartsWith(rule) => label.starts_with(rule),
            Rule::Any() => true,
        }
    }

    /// Check if this rule applies to any of the labels.
    pub fn applies_to_any(&self, labels: &[String]) -> bool {
        match self {
            Rule::ExactMatch(rule) => labels.iter().any(|label| label == rule),
            Rule::StartsWith(rule) => labels.iter().any(|label| label.starts_with(rule.as_str())),
            Rule::Any() => true,
        }
    }
}

impl RulesList {
    /// Is this a restricted label.
    pub fn is_restricted(&self, label: &str) -> bool {
        self.rules.keys().any(|rule| rule.applies(label))
    }

    /// Get restricted labels out of this list.
    pub fn find_restricted(&self, labels: &[String]) -> HashSet<String> {
        self.rules
            .keys()
            .map(|key| match key {
                Rule::ExactMatch(rule) => {
                    labels.iter().filter(|label| label == &rule).collect::<Vec<_>>()
                }
                Rule::StartsWith(rule) => {
                    labels.iter().filter(|label| label.starts_with(rule)).collect::<Vec<_>>()
                }
                Rule::Any() => {
                    vec![]
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
    use crate::access_control::rules::{Rule, RuleMetadata, RulesList};
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
        let actual = list.find_restricted(&[
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
