pub mod access_control;
pub mod community;
pub mod debug;
pub mod migrations;
mod notify;
pub mod post;
mod repost;
mod social_db;
pub mod stats;
pub mod str_serializers;

use crate::access_control::members::ActionType;
use crate::access_control::members::Member;
use crate::access_control::AccessControl;
use community::*;
use post::*;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, Vector};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault};

use std::collections::HashSet;

near_sdk::setup_alloc!();

type PostId = u64;
type IdeaId = u64;
type AttestationId = u64;
type SubmissionId = u64;
type SponsorshipId = u64;
type CommentId = u64;

/// An imaginary top post representing the landing page.
const ROOT_POST_ID: u64 = u64::MAX;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub posts: Vector<VersionedPost>,
    pub post_to_parent: LookupMap<PostId, PostId>,
    pub post_to_children: LookupMap<PostId, Vec<PostId>>,
    pub label_to_posts: UnorderedMap<String, HashSet<PostId>>,
    pub access_control: AccessControl,
    pub authors: UnorderedMap<AccountId, HashSet<PostId>>,
    pub communities: UnorderedMap<CommunityHandle, Community>,
    pub featured_communities: Vec<FeaturedCommunity>,
    pub available_addons: UnorderedMap<CommunityAddOnId, CommunityAddOn>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        migrations::state_version_write(&migrations::StateVersion::V7);
        let mut contract = Self {
            posts: Vector::new(StorageKey::Posts),
            post_to_parent: LookupMap::new(StorageKey::PostToParent),
            post_to_children: LookupMap::new(StorageKey::PostToChildren),
            label_to_posts: UnorderedMap::new(StorageKey::LabelToPostsV2),
            access_control: AccessControl::default(),
            authors: UnorderedMap::new(StorageKey::AuthorToAuthorPosts),
            communities: UnorderedMap::new(StorageKey::Communities),
            featured_communities: Vec::new(),
            available_addons: UnorderedMap::new(StorageKey::AddOns),
        };
        contract.post_to_children.insert(&ROOT_POST_ID, &Vec::new());
        contract
    }

    /// If `parent_id` is not provided get all landing page posts. Otherwise, get all posts under
    /// `parent_id` post.
    pub fn get_posts(&self, parent_id: Option<PostId>) -> Vec<VersionedPost> {
        near_sdk::log!("get_posts");
        let parent_id = parent_id.unwrap_or(ROOT_POST_ID);
        let children_ids = self
            .post_to_children
            .get(&parent_id)
            .unwrap_or_else(|| panic!("Parent id {} not found", parent_id));
        children_ids
            .into_iter()
            .map(|id| {
                self.posts
                    .get(id)
                    .unwrap_or_else(|| panic!("Post id {} not found. Broken state invariant", id))
            })
            .collect()
    }

    pub fn get_post(&self, post_id: PostId) -> VersionedPost {
        near_sdk::log!("get_post");
        self.posts.get(post_id).unwrap_or_else(|| panic!("Post id {} not found", post_id))
    }

    pub fn get_all_post_ids(&self) -> Vec<PostId> {
        (0..self.posts.len()).into_iter().collect()
    }

    pub fn get_children_ids(&self, post_id: Option<PostId>) -> Vec<PostId> {
        near_sdk::log!("get_children_ids");
        let post_id = post_id.unwrap_or(ROOT_POST_ID);
        self.post_to_children
            .get(&post_id)
            .unwrap_or_else(|| panic!("Parent id {} not found", post_id))
    }

    pub fn get_parent_id(&self, post_id: PostId) -> Option<PostId> {
        near_sdk::log!("get_parent_id");
        let res = self
            .post_to_parent
            .get(&post_id)
            .unwrap_or_else(|| panic!("Parent id {} not found", post_id));
        if res == ROOT_POST_ID {
            Option::None
        } else {
            Option::Some(res)
        }
    }

    #[payable]
    pub fn add_like(&mut self, post_id: PostId) {
        near_sdk::log!("add_like");
        let mut post: Post = self
            .posts
            .get(post_id)
            .unwrap_or_else(|| panic!("Post id {} not found", post_id))
            .into();
        let post_author = post.author_id.clone();
        let like =
            Like { author_id: env::predecessor_account_id(), timestamp: env::block_timestamp() };
        post.likes.insert(like);
        self.posts.replace(post_id, &post.into());
        notify::notify_like(post_id, post_author);
    }

    #[payable]
    pub fn add_post(&mut self, parent_id: Option<PostId>, body: PostBody, labels: HashSet<String>) {
        near_sdk::log!("add_post");
        let parent_id = parent_id.unwrap_or(ROOT_POST_ID);
        let id = self.posts.len();
        let author_id = env::predecessor_account_id();
        let editor_id = author_id.clone();
        assert!(
            self.is_allowed_to_use_labels(
                Some(editor_id.clone()),
                labels.iter().cloned().collect()
            ),
            "Cannot use these labels"
        );

        for label in &labels {
            let mut other_posts = self.label_to_posts.get(label).unwrap_or_default();
            other_posts.insert(id);
            self.label_to_posts.insert(label, &other_posts);
        }
        let post = Post {
            id,
            author_id: author_id.clone(),
            likes: Default::default(),
            snapshot: PostSnapshot { editor_id, timestamp: env::block_timestamp(), labels, body },
            snapshot_history: vec![],
        };
        self.posts.push(&post.clone().into());
        self.post_to_parent.insert(&id, &parent_id);

        let mut siblings = self
            .post_to_children
            .get(&parent_id)
            .unwrap_or_else(|| panic!("Parent id {} not found", parent_id));
        siblings.push(id);
        self.post_to_children.insert(&parent_id, &siblings);

        // Don't forget to add an empty list of your own children.
        self.post_to_children.insert(&id, &vec![]);

        let mut author_posts = self.authors.get(&author_id).unwrap_or_else(|| HashSet::new());
        author_posts.insert(post.id);
        self.authors.insert(&post.author_id, &author_posts);

        let desc = get_post_description(post.clone());

        if parent_id != ROOT_POST_ID {
            let parent_post: Post = self
                .posts
                .get(parent_id)
                .unwrap_or_else(|| panic!("Parent post with id {} not found", parent_id))
                .into();
            let parent_author = parent_post.author_id;
            notify::notify_reply(parent_id, parent_author);
        } else {
            repost::repost(post);
        }
        notify::notify_mentions(desc.as_str(), id);
    }

    pub fn get_posts_by_author(&self, author: AccountId) -> Vec<PostId> {
        self.authors.get(&author).map(|posts| posts.into_iter().collect()).unwrap_or(Vec::new())
    }

    pub fn get_posts_by_label(&self, label: String) -> Vec<PostId> {
        near_sdk::log!("get_posts_by_label");
        let mut res: Vec<_> =
            self.label_to_posts.get(&label).unwrap_or_default().into_iter().collect();
        res.sort();
        res
    }

    pub fn get_all_labels(&self) -> Vec<String> {
        near_sdk::log!("get_all_labels");
        let mut res: Vec<_> = self.label_to_posts.keys().collect();
        res.sort();
        res
    }

    pub fn get_all_authors(&self) -> Vec<String> {
        near_sdk::log!("get_all_authors");
        let mut res: Vec<_> = self.authors.keys().collect();
        res.sort();
        res
    }

    pub fn is_allowed_to_edit(&self, post_id: PostId, editor: Option<AccountId>) -> bool {
        near_sdk::log!("is_allowed_to_edit");
        let post: Post = self
            .posts
            .get(post_id)
            .unwrap_or_else(|| panic!("Post id {} not found", post_id))
            .into();
        let editor = editor.unwrap_or_else(env::predecessor_account_id);
        // First check for simple cases.
        if editor == env::current_account_id() || editor == post.author_id {
            return true;
        }

        // Then check for complex case.
        self.access_control
            .members_list
            .check_permissions(editor, &post.snapshot.labels.into_iter().collect::<Vec<_>>())
            .contains(&ActionType::EditPost)
    }

    pub fn is_allowed_to_use_labels(&self, editor: Option<AccountId>, labels: Vec<String>) -> bool {
        let editor = editor.unwrap_or_else(env::predecessor_account_id);
        // First check for simple cases.
        if editor == env::current_account_id() {
            return true;
        }
        let restricted_labels = self.access_control.rules_list.find_restricted(&labels);
        if restricted_labels.is_empty() {
            return true;
        }
        self.access_control
            .members_list
            .check_permissions(editor, &labels)
            .contains(&ActionType::UseLabels)
    }

    pub fn get_all_allowed_labels(&self, editor: AccountId) -> Vec<String> {
        near_sdk::log!("get_all_allowed_labels");
        let mut res: Vec<_> = self
            .label_to_posts
            .keys()
            .filter(|label| {
                self.is_allowed_to_use_labels(Some(editor.clone()), vec![label.clone()])
            })
            .collect();
        res.sort();
        res
    }

    #[payable]
    pub fn edit_post(&mut self, id: PostId, body: PostBody, labels: HashSet<String>) {
        near_sdk::log!("edit_post");
        assert!(
            self.is_allowed_to_edit(id, Option::None),
            "The account is not allowed to edit this post"
        );
        let editor_id = env::predecessor_account_id();
        let mut post: Post =
            self.posts.get(id).unwrap_or_else(|| panic!("Post id {} not found", id)).into();

        let old_snapshot = post.snapshot.clone();
        let old_labels_set = old_snapshot.labels.clone();
        let new_labels = labels;
        let new_snapshot = PostSnapshot {
            editor_id: editor_id.clone(),
            timestamp: env::block_timestamp(),
            labels: new_labels.clone(),
            body,
        };
        post.snapshot = new_snapshot;
        post.snapshot_history.push(old_snapshot);
        let post_author = post.author_id.clone();
        self.posts.replace(id, &post.into());

        // Update labels index.

        let new_labels_set = new_labels;
        let labels_to_remove = &old_labels_set - &new_labels_set;
        let labels_to_add = &new_labels_set - &old_labels_set;
        assert!(
            self.is_allowed_to_use_labels(
                Some(editor_id.clone()),
                labels_to_remove.iter().cloned().collect()
            ),
            "Not allowed to remove these labels"
        );
        assert!(
            self.is_allowed_to_use_labels(
                Some(editor_id.clone()),
                labels_to_add.iter().cloned().collect()
            ),
            "Not allowed to add these labels"
        );

        for label_to_remove in labels_to_remove {
            let mut posts = self.label_to_posts.get(&label_to_remove).unwrap();
            posts.remove(&id);
            self.label_to_posts.insert(&label_to_remove, &posts);
        }

        for label_to_add in labels_to_add {
            let mut posts = self.label_to_posts.get(&label_to_add).unwrap_or_default();
            posts.insert(id);
            self.label_to_posts.insert(&label_to_add, &posts);
        }

        notify::notify_edit(id, post_author);
    }

    #[allow(unused_mut)]
    pub fn create_community(&mut self, mut inputs: CommunityInputs) {
        if self.get_community(inputs.handle.to_owned()).is_some() {
            panic!("Community already exists");
        }

        let mut new_community = Community {
            admins: vec![],
            handle: inputs.handle,
            name: inputs.name,
            tag: inputs.tag,
            description: inputs.description,
            logo_url: inputs.logo_url,
            banner_url: inputs.banner_url,
            bio_markdown: inputs.bio_markdown,
            github_handle: None,
            telegram_handle: vec![],
            twitter_handle: None,
            website_url: None,
            github: None,
            board: None,
            wiki1: None,
            wiki2: None,

            features: CommunityFeatureFlags {
                telegram: true,
                github: true,
                board: true,
                wiki: true,
            },
            addon_list: vec![],
        };

        new_community.validate();
        new_community.set_default_admin();
        self.communities.insert(&new_community.handle, &new_community);
    }

    pub fn get_community(&self, handle: CommunityHandle) -> Option<Community> {
        self.communities.get(&handle)
    }

    pub fn get_community_metadata(&self, handle: CommunityHandle) -> Option<CommunityMetadata> {
        self.communities.get(&handle).map(|community| CommunityMetadata {
            admins: community.admins,
            handle: community.handle,
            name: community.name,
            tag: community.tag,
            description: community.description,
            logo_url: community.logo_url,
            banner_url: community.banner_url,
            bio_markdown: community.bio_markdown,
        })
    }

    pub fn get_account_community_permissions(
        &self,
        account_id: AccountId,
        community_handle: CommunityHandle,
    ) -> CommunityPermissions {
        let community = self.get_community(community_handle.to_owned()).expect(
            format!("Community with handle `{}` does not exist", community_handle).as_str(),
        );

        CommunityPermissions {
            can_configure: community.admins.contains(&account_id)
                || self.has_moderator(account_id.to_owned()),

            can_delete: self.has_moderator(account_id),
        }
    }

    pub fn get_all_communities_metadata(&self) -> Vec<CommunityMetadata> {
        near_sdk::log!("get_all_communities");
        self.communities
            .iter()
            .map(|(handle, community)| CommunityMetadata {
                admins: community.admins,
                handle,
                name: community.name,
                tag: community.tag,
                description: community.description,
                logo_url: community.logo_url,
                banner_url: community.banner_url,
                bio_markdown: community.bio_markdown,
            })
            .collect()
    }

    pub fn get_addon(&self, id: CommunityAddOnId) -> Option<CommunityAddOn> {
        self.available_addons.get(&id)
    }

    pub fn get_available_addons(&self) -> Vec<CommunityAddOn> {
        self.available_addons.iter().map(|(_id, add_on)| add_on).collect()
    }

    // Only the contract admin and DevHub moderators
    pub fn create_new_addon(&mut self, input: CommunityAddOn) {
        if !self.has_moderator(env::predecessor_account_id())
            && env::predecessor_account_id() != env::current_account_id()
        {
            panic!("Only the admin and moderators can create new add-ons");
        }
        if self.get_addon(input.id.to_owned()).is_some() {
            panic!("Add-on with this id already exists");
        }
        self.available_addons.insert(&input.id, &input);
    }

    pub fn delete_addon(&mut self, id: CommunityAddOnId) {
        // Also delete from communities
        if !self.has_moderator(env::predecessor_account_id())
            && env::predecessor_account_id() != env::current_account_id()
        {
            panic!("Only the admin and moderators can delete add-ons");
        }
        let addon =
            self.get_addon(id.clone()).expect(&format!("Add-on with id `{}` does not exist", id));

        for (_id, community) in self.communities.iter() {
            // Try to remove add on from community
            self.remove_community_addon(community.handle, addon.id.clone())
        }

        self.available_addons.remove(&addon.id);
    }

    pub fn edit_addon(&mut self, input: CommunityAddOn) {
        if !self.has_moderator(env::predecessor_account_id())
            && env::predecessor_account_id() != env::current_account_id()
        {
            panic!("Only the admin and moderators can edit add-ons");
        }
        self.available_addons.insert(&input.id.clone(), &input);
    }

    pub fn get_community_addons(&self, handle: CommunityHandle) -> Vec<CommunityAddOn> {
        let community = self
            .get_community(handle.to_owned())
            .expect(format!("Community with handle `{}` does not exist", handle).as_str());
        community
            .addon_list
            .iter()
            .map(|addon_config| {
                self.get_addon(addon_config.addon_id.clone()).expect("add-on missing")
                // TODO add-on should never be missing because they are removed from community when they are removed from available add-ons
                // write a test
            })
            .collect()
    }

    pub fn add_community_addon(
        &self,
        community_handle: CommunityHandle,
        addon_config: CommunityAddOnConfig,
    ) {
        let mut community = self.get_community(community_handle.to_owned()).expect(
            format!("Community with handle `{}` does not exist", community_handle).as_str(),
        );
        if !self.has_moderator(env::predecessor_account_id())
            && env::predecessor_account_id() != env::current_account_id()
            && !community.admins.contains(&env::predecessor_account_id())
        {
            panic!("Only moderators and community admins can delete add-ons to a community");
        }
        if self.get_addon(addon_config.addon_id.to_owned()).is_none() {
            panic!("No add-on exists with this id");
        }

        community.addon_list.push(addon_config);
    }

    pub fn edit_community_addon(
        &self,
        community_handle: CommunityHandle,
        addon_config: CommunityAddOnConfig,
    ) {
        self.remove_community_addon(community_handle.clone(), addon_config.config_id.clone());
        self.add_community_addon(community_handle, addon_config);
    }

    pub fn remove_community_addon(
        &self,
        community_handle: CommunityHandle,
        config_id: CommunityAddOnConfigId,
    ) {
        let mut community = self.get_community(community_handle.to_owned()).expect(
            format!("Community with handle `{}` does not exist", community_handle).as_str(),
        );

        if !self.has_moderator(env::predecessor_account_id())
            && env::predecessor_account_id() != env::current_account_id()
            && !community.admins.contains(&env::predecessor_account_id())
        {
            panic!("Only moderators and community admins can remove add-ons from a community");
        }
        community.addon_list.retain(|config| config.config_id != config_id);
    }

    fn get_editable_community(&self, handle: &CommunityHandle) -> Option<Community> {
        if self
            .get_account_community_permissions(env::predecessor_account_id(), handle.to_owned())
            .can_configure
        {
            return self.get_community(handle.to_owned());
        } else {
            return None;
        };
    }

    #[allow(unused_mut)]
    pub fn update_community(&mut self, handle: CommunityHandle, mut community: Community) {
        let target_community = self
            .get_editable_community(&handle)
            .expect("Only community admins and hub moderators can configure communities");

        community.validate();
        community.set_default_admin();

        if target_community.handle == community.handle {
            self.communities.insert(&target_community.handle, &community);
        } else {
            if self.communities.get(&community.handle).is_some() {
                panic!("Community handle `{}` is already taken", community.handle);
            }

            self.communities.remove(&target_community.handle);
            self.communities.insert(&community.handle, &community);
        }
    }

    pub fn update_community_feature_flags(
        &mut self,
        handle: CommunityHandle,
        features: CommunityFeatureFlags,
    ) {
        let mut community = self
            .get_editable_community(&handle)
            .expect("Only community admins and hub moderators can configure communities");

        community.features = features;
        self.communities.insert(&handle, &community);
    }

    pub fn update_community_github(&mut self, handle: CommunityHandle, github: Option<String>) {
        let mut community = self
            .get_editable_community(&handle)
            .expect("Only community admins and hub moderators can configure boards");

        community.github = github;
        self.communities.insert(&handle, &community);
    }

    pub fn update_community_board(&mut self, handle: CommunityHandle, board: Option<String>) {
        let mut community = self
            .get_editable_community(&handle)
            .expect("Only community admins and hub moderators can configure boards");

        community.board = board;
        self.communities.insert(&handle, &community);
    }

    pub fn update_community_wiki1(&mut self, handle: CommunityHandle, wiki1: Option<WikiPage>) {
        let mut community = self
            .get_editable_community(&handle)
            .expect("Only community admins and hub moderators can edit wiki");

        community.wiki1 = wiki1;
        self.communities.insert(&handle, &community);
    }

    pub fn update_community_wiki2(&mut self, handle: CommunityHandle, wiki2: Option<WikiPage>) {
        let mut community = self
            .get_editable_community(&handle)
            .expect("Only community admins and hub moderators can edit wiki");

        community.wiki2 = wiki2;
        self.communities.insert(&handle, &community);
    }

    pub fn delete_community(&mut self, handle: CommunityHandle) {
        if !self.has_moderator(env::predecessor_account_id()) {
            panic!("Only moderators can delete community");
        }

        let community = self
            .get_community(handle.clone())
            .expect(&format!("Community with handle `{}` does not exist", handle));

        self.communities.remove(&community.handle);
    }

    pub fn set_featured_communities(&mut self, handles: Vec<CommunityHandle>) {
        assert!(
            self.has_moderator(env::predecessor_account_id()),
            "Only moderators can add featured communities"
        );

        // Check if every handle corresponds to an existing community
        for handle in &handles {
            if !self.communities.get(&handle).is_some() {
                panic!("Community '{}' does not exist.", handle);
            }
        }

        // Replace the existing featured communities with the new ones
        self.featured_communities =
            handles.into_iter().map(|handle| FeaturedCommunity { handle }).collect();
    }

    pub fn get_featured_communities(&self) -> Vec<Community> {
        self.featured_communities
            .iter()
            .filter_map(|fc| self.get_community(fc.handle.clone()))
            .collect()
    }

    pub fn has_moderator(&self, account_id: AccountId) -> bool {
        let moderators = self.access_control.members_list.get_moderators();
        moderators.contains(&Member::Account(account_id))
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use std::collections::HashSet;
    use std::convert::TryInto;

    // use crate::access_control::members::{
    //     ActionType, Member, MemberMetadata, VersionedMemberMetadata,
    // };
    // use crate::access_control::rules::Rule;
    use crate::community::CommunityAddOn;
    use crate::post::PostBody;
    use near_sdk::test_utils::{get_created_receipts, VMContextBuilder};
    use near_sdk::{testing_env, MockedBlockchain, VMContext};
    use regex::Regex;

    use super::Contract;

    fn get_context(is_view: bool) -> VMContext {
        get_context_with_signer(is_view, "bob.near".to_string())
    }

    fn get_context_with_signer(is_view: bool, signer: String) -> VMContext {
        VMContextBuilder::new()
            .signer_account_id(signer.clone().try_into().unwrap())
            .current_account_id(signer.try_into().unwrap())
            .is_view(is_view)
            .build()
    }

    fn get_context_with_current(is_view: bool, signer: String) -> VMContext {
        VMContextBuilder::new()
            .current_account_id(signer.try_into().unwrap())
            .is_view(is_view)
            .build()
    }

    #[test]
    pub fn test_add_post_with_mention() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Contract::new();

        let body: PostBody = near_sdk::serde_json::from_str(r#"
        {
            "name": "another post",
            "description": "Hello to @petersalomonsen.near and @psalomo.near. This is an idea with mentions.",
            "post_type": "Idea",
            "idea_version": "V1"
        }"#).unwrap();
        contract.add_post(None, body, HashSet::new());
        let receipts = get_created_receipts();
        assert_eq!(2, receipts.len());
        let receipt = receipts.get(1).unwrap();
        let receipt_str = format!("{:?}", receipt);
        let re = Regex::new(r#"method_name: (\[[^\]]*\]), args: (\[[^\]]*\])"#).unwrap();

        // Extract the method_name and args values
        for cap in re.captures_iter(&receipt_str) {
            let method_name = &cap[1];

            let args = &cap[2];

            let method_name = method_name
                .trim_start_matches('[')
                .trim_end_matches(']')
                .split(", ")
                .map(|s| s.parse().unwrap())
                .collect::<Vec<u8>>();
            let method_name =
                String::from_utf8(method_name).expect("Failed to convert method_name to String");

            assert_eq!("set", method_name);

            let args = args
                .trim_start_matches('[')
                .trim_end_matches(']')
                .split(", ")
                .map(|s| s.parse().unwrap())
                .collect::<Vec<u8>>();
            let args = String::from_utf8(args).expect("Failed to convert args to String");

            assert_eq!("{\"data\":{\"bob.near\":{\"index\":{\"notify\":\"[{\\\"key\\\":\\\"petersalomonsen.near\\\",\\\"value\\\":{\\\"type\\\":\\\"devgovgigs/mention\\\",\\\"post\\\":0}},{\\\"key\\\":\\\"psalomo.near.\\\",\\\"value\\\":{\\\"type\\\":\\\"devgovgigs/mention\\\",\\\"post\\\":0}}]\"}}}}", args);
        }
    }

    #[test]
    pub fn test_create_new_addon() {
        let context = get_context_with_current(false, "bob.near".to_string());
        testing_env!(context);

        let mut contract = Contract::new();
        let input = CommunityAddOn {
            id: "CommunityAddOnId".to_owned(),
            title: "GitHub AddOn".to_owned(),
            description: "Current status of NEARCORE repo".to_owned(),
            viewer: "custom-viewer-widget".to_owned(),
            configurator: "github-configurator".to_owned(),
            icon: "bi bi-github".to_owned(),
        };
        contract.create_new_addon(input.to_owned());

        let addon = contract.get_addon("CommunityAddOnId".to_owned());

        assert_eq!(addon, Some(input))
    }

    #[test]
    pub fn test_get_available_addons() {
        let context = get_context_with_current(false, "bob.near".to_string());
        testing_env!(context);
        let mut contract = Contract::new();
        let input = CommunityAddOn {
            id: "CommunityAddOnId".to_owned(),
            title: "GitHub AddOn".to_owned(),
            description: "Current status of NEARCORE repo".to_owned(),
            viewer: "custom-viewer-widget".to_owned(),
            configurator: "github-configurator".to_owned(),
            icon: "bi bi-github".to_owned(),
        };
        contract.create_new_addon(input.to_owned());

        let addons = contract.get_available_addons();

        assert_eq!(addons, vec![input])
    }

    #[test]
    pub fn test_get_addon() {
        let context = get_context_with_current(false, "bob.near".to_string());
        testing_env!(context);
        let mut contract = Contract::new();
        let input = CommunityAddOn {
            id: "CommunityAddOnId".to_owned(),
            title: "GitHub AddOn".to_owned(),
            description: "Current status of NEARCORE repo".to_owned(),
            viewer: "custom-viewer-widget".to_owned(),
            configurator: "github-configurator".to_owned(),
            icon: "bi bi-github".to_owned(),
        };
        contract.create_new_addon(input.to_owned());

        let addon = contract.get_addon("CommunityAddOnId".to_owned());

        assert_eq!(addon, Some(input))
    }

    #[test]
    pub fn test_delete_addon() {
        let context = get_context_with_current(false, "bob.near".to_string());
        testing_env!(context);
        let mut contract = Contract::new();
        let input = CommunityAddOn {
            id: "CommunityAddOnId".to_owned(),
            title: "GitHub AddOn".to_owned(),
            description: "Current status of NEARCORE repo".to_owned(),
            viewer: "custom-viewer-widget".to_owned(),
            configurator: "github-configurator".to_owned(),
            icon: "bi bi-github".to_owned(),
        };
        contract.create_new_addon(input.to_owned());

        contract.delete_addon("CommunityAddOnId".to_owned());

        let addons = contract.get_available_addons();

        assert_eq!(addons, vec![])
    }

    #[test]
    pub fn test_edit_addon() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Contract::new();
        let input = CommunityAddOn {
            id: "CommunityAddOnId".to_owned(),
            title: "GitHub AddOn".to_owned(),
            description: "Current status of NEARCORE repo".to_owned(),
            viewer: "custom-viewer-widget".to_owned(),
            configurator: "github-configurator".to_owned(),
            icon: "bi bi-github".to_owned(),
        };
        contract.create_new_addon(input.to_owned());

        contract.edit_addon(CommunityAddOn { title: "Telegram AddOn".to_owned(), ..input });

        let addons = contract.get_available_addons();

        assert_eq!(addons[0].title, "Telegram AddOn".to_owned());
    }
}
