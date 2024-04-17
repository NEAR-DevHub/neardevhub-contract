use near_sdk::{env, ext_contract, near, require, AccountId, Gas, NearToken};

pub type CommunityHandle = String;

pub type AddOnId = String;

#[near(serializers=[borsh, json])]
#[derive(Clone)]
pub struct CommunityInputs {
    pub handle: CommunityHandle,
    pub name: String,
    pub tag: String,
    pub description: String,
    pub logo_url: String,
    pub banner_url: String,
    pub bio_markdown: Option<String>,
}

#[near(serializers=[borsh, json])]
#[derive(Clone)]
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

#[near(serializers=[borsh, json])]
#[derive(Clone)]
pub struct CommunityFeatureFlags {
    pub telegram: bool,
    pub github: bool,
    pub board: bool,
    pub wiki: bool,
}

#[near(serializers=[borsh, json])]
#[derive(Clone)]
pub struct WikiPage {
    name: String,
    content_markdown: String,
}

#[near(serializers=[borsh, json])]
#[derive(Clone, PartialEq, Debug)]
pub struct CommunityAddOn {
    pub id: String,
    pub addon_id: AddOnId,
    pub display_name: String,
    pub enabled: bool,
    pub parameters: String,
}

#[near(serializers=[borsh, json])]
#[derive(Clone, PartialEq, Debug)]
pub struct AddOn {
    pub id: AddOnId,
    pub title: String,
    pub description: String,
    pub icon: String,
    // The path to the view on the community page
    pub view_widget: String,
    // The path to the view on the community configuration page
    pub configurator_widget: String,
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
        if !matches!(self.view_widget.chars().count(), 6..=240) {
            panic!("Add-on viewer must contain 6 to 240 characters");
        }
        if !matches!(self.configurator_widget.chars().count(), 0..=240) {
            panic!("Add-on configurator must contain 0 to 240 characters");
        }
        if !matches!(self.icon.chars().count(), 6..=60) {
            panic!("Add-on icon must contain 6 to 60 characters");
        }
    }
}

#[near(serializers=[borsh, json])]
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
    pub telegram_handle: Option<String>,
    pub twitter_handle: Option<String>,
    pub website_url: Option<String>,
    pub addons: Vec<CommunityAddOn>,
}

#[near(serializers=[borsh, json])]
#[derive(Clone)]
pub struct FeaturedCommunity {
    pub handle: CommunityHandle,
}

#[near(serializers=[borsh, json])]
#[derive(Clone)]
pub struct CommunityPermissions {
    pub can_configure: bool,
    pub can_delete: bool,
}

impl Community {
    pub fn validate(&self) {
        require!(
            matches!(self.handle.chars().count(), 3..=40),
            "Community handle must contain 3 to 40 characters"
        );
        require!(
            self.handle.parse::<AccountId>().is_ok() && !self.handle.contains('.'),
            "Community handle should be lowercase alphanumeric symbols separated either by `_` or `-`, separators are not permitted to immediately follow each other, start or end with separators"
        );
        require!(
            matches!(self.name.chars().count(), 3..=30),
            "Community name must contain 3 to 30 characters"
        );
        require!(
            matches!(self.tag.chars().count(), 3..=30),
            "Community tag must contain 3 to 30 characters"
        );
        require!(
            matches!(self.description.chars().count(), 6..=60),
            "Community description must contain 6 to 60 characters"
        );
        require!(
            self.bio_markdown.is_none()
                || matches!(
                    self.bio_markdown.to_owned().map_or(0, |text| text.chars().count()),
                    3..=200
                ),
            "Community bio must contain 3 to 200 characters"
        );
    }

    pub fn add_addon(&mut self, addon_config: CommunityAddOn) {
        self.addons.push(addon_config);
    }

    pub fn remove_addon(&mut self, addon_to_remove: CommunityAddOn) {
        self.addons.retain(|addon| addon != &addon_to_remove);
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

pub fn get_devhub_community_factory() -> AccountId {
    format!("community.{}", env::current_account_id()).parse().unwrap()
}

pub fn get_devhub_discussions_factory(handle: &CommunityHandle) -> AccountId {
    get_devhub_community_account(handle).parse().unwrap()
}

pub fn get_devhub_community_account(handle: &CommunityHandle) -> String {
    format!("{}.{}", handle, get_devhub_community_factory())
}

pub fn get_devhub_discussions_account(handle: &CommunityHandle) -> String {
    format!("discussions.{}", get_devhub_community_account(handle))
}

#[ext_contract(ext_devhub_community_factory)]
pub trait DevhubCommunityFactory {
    fn create_community_account(&mut self, community: String);
}

#[ext_contract(ext_devhub_community)]
pub trait DevhubCommunity {
    fn destroy(&mut self);

    fn create_discussions_account(&mut self);
}

pub const CREATE_COMMUNITY_BALANCE: NearToken = NearToken::from_near(4);
pub const CREATE_DISCUSSION_BALANCE: NearToken = NearToken::from_near(2);
pub const CREATE_COMMUNITY_GAS: Gas = Gas::from_tgas(200);
pub const UPDATE_COMMUNITY_GAS: Gas = Gas::from_tgas(30);
pub const DELETE_COMMUNITY_GAS: Gas = Gas::from_tgas(30);
pub const SET_COMMUNITY_SOCIALDB_GAS: Gas = Gas::from_tgas(30);
pub const CREATE_DISCUSSION_GAS: Gas = Gas::from_tgas(30);
