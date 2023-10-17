use core::fmt;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::serde::Serializer;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, AccountId};

use crate::post::StorageKey;

pub type CommunityHandle = String;

pub type AddOnId = String;

pub type AddOnConfigId = String;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum TemplateType {
    Viewer,
    Configurator,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CommunityInputs {
    pub handle: CommunityHandle,
    pub name: String,
    pub tag: String,
    pub description: String,
    pub logo_url: String,
    pub banner_url: String,
    pub bio_markdown: Option<String>,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CommunityMetadata {
    pub admins: Vec<AccountId>,
    pub handle: CommunityHandle,
    pub name: String,
    pub tag: String,
    pub description: String,
    pub logo_url: String,
    pub banner_url: String,
    pub bio_markdown: Option<String>,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CommunityFeatureFlags {
    pub telegram: bool,
    pub github: bool,
    pub board: bool,
    pub wiki: bool,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct WikiPage {
    name: String,
    content_markdown: String,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AddOnConfig {
    pub id: AddOnConfigId,
    pub parameters: String,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct CommunityAddOn {
    pub addon_id: AddOnId,
    pub display_name: String,
    pub icon: String,
    pub enabled: bool,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AddOn {
    pub id: AddOnId,
    pub title: String,
    pub description: String,
    pub icon: String,
    pub widgets: LookupMap<TemplateType, String>,
}

impl Clone for AddOn {
    fn clone(&self) -> Self {
        AddOn {
            id: self.id.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            icon: self.icon.clone(),
            widgets: clone_widgets(&self.widgets),
        }
    }
}

impl PartialEq for AddOn {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl fmt::Debug for AddOn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // You specify how the struct should be formatted here
        write!(f,
        "AddOn {{ id: {}, title: {}, description: {}, icon: {}, widgets: {{viewer: {}, configurator: {}}} }}",
        self.id,
        self.title,
        self.description,
        self.icon,
        self.widgets
        .get(&TemplateType::Viewer).unwrap_or("default".to_string()),
        self.widgets
        .get(&TemplateType::Configurator).unwrap_or("default".to_string()))
    }
}

// impl Serialize for AddOn {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer
//     {
//         let mut s = serializer.serialize_struct("AddOn", 5)?;
//         // s.serialize_field("id", &self.id)?;
//         // s.serialize_field("title", &self.title)?;
//         // s.serialize_field("description", &self.description)?;
//         // s.serialize_field("icon", &self.icon)?;
//         // s.serialize_map("widgets", &self.widgets)?;
//         // s.serialize_field("widgets")
//         // s.serialize_map(2);
//         // s.end()
//         s
//     }
// }

fn clone_widgets(original: &LookupMap<TemplateType, String>) -> LookupMap<TemplateType, String> {
    let mut cloned = LookupMap::new(StorageKey::TemplateType);

    cloned.insert(&TemplateType::Viewer, &original.get(&TemplateType::Viewer).unwrap());
    cloned.insert(&TemplateType::Configurator, &original.get(&TemplateType::Configurator).unwrap());

    return cloned;
}

impl AddOn {
    pub fn validate(&self) {
        if !matches!(self.id.chars().count(), 3..=120) {
            panic!("Add-on id must contain 3 to 120 characters");
        }

        if !matches!(self.title.chars().count(), 3..=120) {
            panic!("Add-on title must contain 3 to 120 characters");
        }

        if !matches!(self.description.chars().count(), 3..=120) {
            panic!("Add-on description must contain 3 to 120 characters");
        }
        if !matches!(self.widgets.get(&TemplateType::Viewer).chars().count(), 6..=240) {
            panic!("Add-on viewer must contain 6 to 240 characters");
        }
        if !matches!(self.widgets.get(&TemplateType::Configurator).chars().count(), 0..=240) {
            panic!("Add-on configurator must contain 0 to 240 characters");
        }
        if !matches!(self.icon.chars().count(), 6..=60) {
            panic!("Add-on icon must contain 6 to 60 characters");
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Community {
    pub admins: Vec<AccountId>,
    pub handle: CommunityHandle,
    pub name: String,
    pub tag: String,
    pub description: String,
    pub logo_url: String,
    pub banner_url: String,
    pub bio_markdown: Option<String>,
    pub github_handle: Option<String>,
    pub telegram_handle: Vec<String>,
    pub twitter_handle: Option<String>,
    pub website_url: Option<String>,
    /// JSON string of github board configuration
    pub github: Option<String>,
    /// JSON string of kanban board configuration
    pub board: Option<String>,
    pub wiki1: Option<WikiPage>,
    pub wiki2: Option<WikiPage>,
    pub features: CommunityFeatureFlags,
    pub addons: Vec<CommunityAddOn>,
    pub configs: LookupMap<AddOnConfigId, AddOnConfig>,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct FeaturedCommunity {
    pub handle: CommunityHandle,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CommunityPermissions {
    pub can_configure: bool,
    pub can_delete: bool,
}

impl Community {
    pub fn validate(&self) {
        if !matches!(self.handle.chars().count(), 3..=40) {
            panic!("Community handle must contain 3 to 40 characters");
        }

        if !matches!(self.name.chars().count(), 3..=30) {
            panic!("Community name must contain 3 to 30 characters");
        }

        if !matches!(self.tag.chars().count(), 3..=30) {
            panic!("Community tag must contain 3 to 30 characters");
        }

        if !matches!(self.description.chars().count(), 6..=60) {
            panic!("Community description must contain 6 to 60 characters");
        }

        if self.bio_markdown.is_some()
            && !matches!(
                self.bio_markdown.to_owned().map_or(0, |text| text.chars().count()),
                3..=200
            )
        {
            panic!("Community bio must contain 3 to 200 characters");
        }
    }

    pub fn add_addon(&mut self, addon_config: CommunityAddOn) {
        self.addons.push(addon_config);
    }

    pub fn remove_addon(&mut self, addon2: CommunityAddOn) {
        self.addons.retain(|addon| addon != &addon2);
    }

    pub fn set_addons(&mut self, addons: Vec<CommunityAddOn>) {
        self.addons = addons;
    }

    pub fn set_default_admin(&mut self) {
        if self.admins.is_empty() {
            self.admins = vec![env::predecessor_account_id()];
        }
    }
}
