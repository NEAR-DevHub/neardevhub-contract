pub mod access_control;
pub mod community;
pub mod debug;
pub mod migrations;
mod notify;
pub mod post;
pub mod proposal;
mod repost;
mod social_db;
pub mod stats;
pub mod str_serializers;

use crate::access_control::members::ActionType;
use crate::access_control::members::Member;
use crate::access_control::AccessControl;
use crate::social_db::{social_db_contract, SetReturnType};
use community::*;
use near_sdk::schemars::JsonSchema;
use post::*;
use proposal::*;

use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, Vector};
use near_sdk::require;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde_json::{json, Value};
use near_sdk::Promise;
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault};
use serde_json::Number;

use std::collections::HashSet;
use std::convert::TryInto;

type PostId = u64;
type IdeaId = u64;
type AttestationId = u64;
type SolutionId = u64;
type SponsorshipId = u64;
type CommentId = u64;

/// An imaginary top post representing the landing page.
const ROOT_POST_ID: u64 = u64::MAX;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
#[borsh(crate = "near_sdk::borsh")]
pub struct Contract {
    pub posts: Vector<VersionedPost>,
    pub post_to_parent: LookupMap<PostId, PostId>,
    pub post_to_children: LookupMap<PostId, Vec<PostId>>,
    pub label_to_posts: UnorderedMap<String, HashSet<PostId>>,
    pub access_control: AccessControl,
    pub authors: UnorderedMap<AccountId, HashSet<PostId>>,
    pub proposals: Vector<VersionedProposal>,
    pub label_to_proposals: UnorderedMap<String, HashSet<ProposalId>>,
    pub author_proposals: UnorderedMap<AccountId, HashSet<ProposalId>>,
    pub proposal_categories: Vec<String>,
    pub communities: UnorderedMap<CommunityHandle, Community>,
    pub featured_communities: Vec<FeaturedCommunity>,
    pub available_addons: UnorderedMap<AddOnId, AddOn>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        migrations::state_version_write(&migrations::StateVersion::V10);

        let mut contract = Self {
            posts: Vector::new(StorageKey::Posts),
            post_to_parent: LookupMap::new(StorageKey::PostToParent),
            post_to_children: LookupMap::new(StorageKey::PostToChildren),
            label_to_posts: UnorderedMap::new(StorageKey::LabelToPostsV2),
            access_control: AccessControl::default(),
            authors: UnorderedMap::new(StorageKey::AuthorToAuthorPosts),
            proposals: Vector::new(StorageKey::Proposals),
            label_to_proposals: UnorderedMap::new(StorageKey::LabelToProposals),
            author_proposals: UnorderedMap::new(StorageKey::AuthorProposals),
            proposal_categories: default_categories(),
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
        near_sdk::log!("get_all_post_ids");
        (0..self.posts.len()).collect()
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

    pub fn get_proposals(&self) -> Vec<VersionedProposal> {
        near_sdk::log!("get_proposals");
        self.proposals.to_vec()
    }

    pub fn get_proposal(&self, proposal_id: ProposalId) -> VersionedProposal {
        near_sdk::log!("get_proposal");
        self.proposals
            .get(proposal_id.into())
            .unwrap_or_else(|| panic!("Proposal id {} not found", proposal_id))
    }

    pub fn get_all_proposal_ids(&self) -> Vec<ProposalId> {
        near_sdk::log!("get_all_proposal_ids");
        (0..self.proposals.len().try_into().unwrap()).collect()
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
        require!(
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

        let mut author_posts = self.authors.get(&author_id).unwrap_or_default();
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

    #[payable]
    pub fn add_proposal(&mut self, body: VersionedProposalBody, labels: HashSet<String>) -> Promise {
        near_sdk::log!("add_proposal");
        let id: ProposalId = self.proposals.len().try_into().unwrap();
        let author_id = env::predecessor_account_id();
        let editor_id = author_id.clone();

        let proposal_body = body.clone().latest_version();

        require!(
            self.is_allowed_to_use_labels(
                Some(editor_id.clone()),
                labels.iter().cloned().collect()
            ),
            "Cannot use these labels"
        );
        require!(
            proposal_body.payouts.is_empty(),
            "Can't add proposal with payouts at the beginning"
        );

        require!(self.proposal_categories.contains(&proposal_body.category), "Unknown category");

        require!(
            proposal_body.timeline.is_draft(),
            "Cannot create proposal which is not in a draft state"
        );

        for label in &labels {
            let mut other_proposals = self.label_to_proposals.get(label).unwrap_or_default();
            other_proposals.insert(id);
            self.label_to_proposals.insert(label, &other_proposals);
        }

        let mut author_proposals = self.author_proposals.get(&author_id).unwrap_or_default();
        author_proposals.insert(id);
        self.author_proposals.insert(&author_id, &author_proposals);

        let proposal = Proposal {
            id: id,
            author_id: author_id.clone(),
            social_db_post_block_height: 0u64,
            snapshot: ProposalSnapshot {
                editor_id,
                timestamp: env::block_timestamp(),
                labels,
                body: body.clone(),
            },
            snapshot_history: vec![],
        };

        let socialdb_promise = proposal::repost::publish_to_socialdb_feed(
            Self::ext(env::current_account_id())
                .with_static_gas(env::prepaid_gas().saturating_div(3))
                .set_block_height_callback(proposal.clone()),
            proposal.clone(),
        );

        notify::notify_proposal_subscribers(&proposal);

        socialdb_promise
    }

    #[private]
    pub fn set_block_height_callback(
        &mut self,
        #[allow(unused_mut)] mut proposal: Proposal,
        #[callback_unwrap] set_result: SetReturnType,
    ) -> BlockHeightCallbackRetValue {
        proposal.social_db_post_block_height = set_result.block_height.into();
        self.proposals.push(&proposal.clone().into());
        BlockHeightCallbackRetValue { proposal_id: proposal.id }
    }

    pub fn get_posts_by_author(&self, author: AccountId) -> Vec<PostId> {
        near_sdk::log!("get_posts_by_author");
        self.authors.get(&author).map(|posts| posts.into_iter().collect()).unwrap_or_default()
    }

    pub fn get_posts_by_label(&self, label: String) -> Vec<PostId> {
        near_sdk::log!("get_posts_by_label");
        let mut res: Vec<_> =
            self.label_to_posts.get(&label).unwrap_or_default().into_iter().collect();
        res.sort();
        res
    }

    pub fn get_proposals_by_author(&self, author: AccountId) -> Vec<ProposalId> {
        near_sdk::log!("get_proposals_by_author");
        self.author_proposals
            .get(&author)
            .map(|proposals| proposals.into_iter().collect())
            .unwrap_or_default()
    }

    pub fn get_proposals_by_label(&self, label: String) -> Vec<ProposalId> {
        near_sdk::log!("get_proposals_by_label");
        let mut res: Vec<_> =
            self.label_to_proposals.get(&label).unwrap_or_default().into_iter().collect();
        res.sort();
        res
    }

    pub fn get_all_labels(&self) -> Vec<String> {
        near_sdk::log!("get_all_labels");
        let mut res: Vec<_> = self.label_to_posts.keys().collect();
        res.sort();
        res
    }

    pub fn get_all_proposal_labels(&self) -> Vec<String> {
        near_sdk::log!("get_all_proposal_labels");
        let mut res: Vec<_> = self.label_to_proposals.keys().collect();
        res.sort();
        res
    }

    pub fn get_all_authors(&self) -> Vec<AccountId> {
        near_sdk::log!("get_all_authors");
        let mut res: Vec<_> = self.authors.keys().collect();
        res.sort();
        res
    }

    pub fn get_all_proposal_authors(&self) -> Vec<AccountId> {
        near_sdk::log!("get_all_proposal_authors");
        let mut res: Vec<_> = self.author_proposals.keys().collect();
        res.sort();
        res
    }

    pub fn is_allowed_to_edit_proposal(
        &self,
        proposal_id: ProposalId,
        editor: Option<AccountId>,
    ) -> bool {
        near_sdk::log!("is_allowed_to_edit_proposal");
        let proposal: Proposal = self
            .proposals
            .get(proposal_id.try_into().unwrap())
            .unwrap_or_else(|| panic!("Proposal id {} not found", proposal_id))
            .into();
        let editor = editor.unwrap_or_else(env::predecessor_account_id);
        // First check for simple cases.
        if editor == env::current_account_id() || editor == proposal.author_id {
            return true;
        }

        // Then check for complex case.
        self.access_control
            .members_list
            .check_permissions(editor, proposal.snapshot.labels.into_iter().collect::<Vec<_>>())
            .contains(&ActionType::EditPost)
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
            .check_permissions(editor, post.snapshot.labels.into_iter().collect::<Vec<_>>())
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
            .check_permissions(editor, labels)
            .contains(&ActionType::UseLabels)
    }

    fn filtered_labels<T>(
        &self,
        labels_to_t: &UnorderedMap<String, T>,
        editor: &AccountId,
    ) -> Vec<String>
    where
        T: near_sdk::borsh::BorshSerialize + near_sdk::borsh::BorshDeserialize,
    {
        let filtered: HashSet<String> = labels_to_t
            .keys()
            .filter(|label| {
                self.is_allowed_to_use_labels(Some(editor.clone()), vec![label.clone()])
            })
            .collect();
        let mut res: Vec<_> = filtered.into_iter().collect();
        res.sort();
        res
    }

    pub fn get_all_allowed_labels(&self, editor: AccountId) -> Vec<String> {
        near_sdk::log!("get_all_allowed_labels");
        self.filtered_labels(&self.label_to_posts, &editor)
    }

    pub fn get_all_allowed_proposal_labels(&self, editor: AccountId) -> Vec<String> {
        near_sdk::log!("get_all_allowed_proposal_labels");
        self.filtered_labels(&self.label_to_proposals, &editor)
    }

    #[payable]
    pub fn edit_post(&mut self, id: PostId, body: PostBody, labels: HashSet<String>) {
        near_sdk::log!("edit_post");
        require!(
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
        require!(
            self.is_allowed_to_use_labels(
                Some(editor_id.clone()),
                labels_to_remove.iter().cloned().collect()
            ),
            "Not allowed to remove these labels"
        );
        require!(
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

    #[payable]
    pub fn create_community(
        &mut self,
        #[allow(unused_mut)] mut inputs: CommunityInputs,
    ) -> Promise {
        require!(
            self.get_community(inputs.handle.to_owned()).is_none(),
            "Community already exists"
        );

        require!(
            env::attached_deposit() >= CREATE_COMMUNITY_BALANCE,
            "Require 4 NEAR to create community"
        );

        require!(env::prepaid_gas() >= CREATE_COMMUNITY_GAS, "Require at least 200 Tgas");

        let mut new_community = Community {
            admins: vec![],
            handle: inputs.handle.clone(),
            name: inputs.name,
            tag: inputs.tag,
            description: inputs.description,
            logo_url: inputs.logo_url,
            banner_url: inputs.banner_url,
            bio_markdown: inputs.bio_markdown,
            github_handle: None,
            telegram_handle: None,
            twitter_handle: None,
            website_url: None,
            addons: vec![],
        };

        new_community.validate();
        new_community.set_default_admin();
        self.communities.insert(&new_community.handle, &new_community);

        ext_devhub_community_factory::ext(get_devhub_community_factory())
            .with_unused_gas_weight(1)
            .with_attached_deposit(CREATE_COMMUNITY_BALANCE)
            .create_community_account(new_community.handle.clone())
    }

    #[payable]
    pub fn edit_proposal(
        &mut self,
        id: ProposalId,
        body: VersionedProposalBody,
        labels: HashSet<String>,
    ) {
        near_sdk::log!("edit_proposal");
        require!(
            self.is_allowed_to_edit_proposal(id, Option::None),
            "The account is not allowed to edit this proposal"
        );
        let editor_id = env::predecessor_account_id();
        let mut proposal: Proposal =
            self.proposals.get(id.into()).unwrap_or_else(|| panic!("Proposal id {} not found", id)).into();

        let proposal_body = body.clone().latest_version();

        require!(
            self.has_moderator(editor_id.clone())
                || editor_id.clone() == env::current_account_id()
                || (proposal.snapshot.body.clone().latest_version().timeline.is_draft())
                    && (proposal_body.timeline.is_empty_review()
                        || proposal_body.timeline.is_draft()),
            "This account is only allowed to change proposal status from DRAFT to REVIEW"
        );

        require!(self.proposal_categories.contains(&proposal_body.category), "Unknown category");

        let old_snapshot = proposal.snapshot.clone();
        let old_labels_set = old_snapshot.labels.clone();
        let new_labels = labels;
        let new_snapshot = ProposalSnapshot {
            editor_id: editor_id.clone(),
            timestamp: env::block_timestamp(),
            labels: new_labels.clone(),
            body,
        };
        proposal.snapshot = new_snapshot;
        proposal.snapshot_history.push(old_snapshot);
        let proposal_author = proposal.author_id.clone();
        self.proposals.replace(id.try_into().unwrap(), &proposal.into());

        // Update labels index.

        let new_labels_set = new_labels;
        let labels_to_remove = &old_labels_set - &new_labels_set;
        let labels_to_add = &new_labels_set - &old_labels_set;
        require!(
            self.is_allowed_to_use_labels(
                Some(editor_id.clone()),
                labels_to_remove.iter().cloned().collect()
            ),
            "Not allowed to remove these labels"
        );
        require!(
            self.is_allowed_to_use_labels(
                Some(editor_id.clone()),
                labels_to_add.iter().cloned().collect()
            ),
            "Not allowed to add these labels"
        );

        for label_to_remove in labels_to_remove {
            let mut proposals = self.label_to_proposals.get(&label_to_remove).unwrap();
            proposals.remove(&id);
            self.label_to_proposals.insert(&label_to_remove, &proposals);
        }

        for label_to_add in labels_to_add {
            let mut proposals = self.label_to_proposals.get(&label_to_add).unwrap_or_default();
            proposals.insert(id);
            self.label_to_proposals.insert(&label_to_add, &proposals);
        }

        notify::notify_edit_proposal(id, proposal_author);
    }

    pub fn get_allowed_categories(&self) -> Vec<String> {
        near_sdk::log!("get_allowed_categories");
        self.proposal_categories.clone()
    }

    #[payable]
    pub fn set_allowed_categories(&mut self, new_categories: Vec<String>) {
        let editor_id = env::predecessor_account_id();
        require!(
            self.has_moderator(editor_id.clone()) || editor_id.clone() == env::current_account_id(),
            "Only the admin and moderators can set categories"
        );
        self.proposal_categories = new_categories;
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

    pub fn get_addon(&self, id: AddOnId) -> Option<AddOn> {
        self.available_addons.get(&id)
    }

    pub fn get_all_addons(&self) -> Vec<AddOn> {
        self.available_addons.iter().map(|(_id, add_on)| add_on).collect()
    }

    // Only the contract admin and DevHub moderators
    pub fn create_addon(&mut self, addon: AddOn) {
        if !self.has_moderator(env::predecessor_account_id())
            && env::predecessor_account_id() != env::current_account_id()
        {
            panic!("Only the admin and moderators can create new add-ons");
        }
        if self.get_addon(addon.id.to_owned()).is_some() {
            panic!("Add-on with this id already exists");
        }
        addon.validate();
        self.available_addons.insert(&addon.id.clone(), &addon);
    }

    // ONLY FOR TESTING
    pub fn delete_addon(&mut self, id: AddOnId) {
        // Also delete from communities
        if !self.has_moderator(env::predecessor_account_id())
            && env::predecessor_account_id() != env::current_account_id()
        {
            panic!("Only the admin and moderators can delete add-ons");
        }
        let addon = self
            .get_addon(id.clone())
            .expect(&format!("Add-on with id `{}` does not exist", id))
            .clone();

        self.available_addons.remove(&addon.id);
    }

    pub fn update_addon(&mut self, addon: AddOn) {
        if !self.has_moderator(env::predecessor_account_id())
            && env::predecessor_account_id() != env::current_account_id()
        {
            panic!("Only the admin and moderators can edit add-ons");
        }
        self.available_addons.insert(&addon.id.clone(), &addon);
    }

    pub fn get_community_addons(&self, handle: CommunityHandle) -> Vec<CommunityAddOn> {
        let community = self
            .get_community(handle.clone())
            .expect(format!("Community not found with handle `{}`", handle).as_str());
        return community.addons;
    }

    pub fn set_community_addons(&mut self, handle: CommunityHandle, addons: Vec<CommunityAddOn>) {
        let mut community = self
            .get_community(handle.clone())
            .expect(format!("Community not found with handle `{}`", handle).as_str());
        community.addons = addons;
        self.update_community(handle, community);
    }

    // To add or update parameters set by the configurator widget
    pub fn set_community_addon(
        &mut self,
        handle: CommunityHandle,
        community_addon: CommunityAddOn,
    ) {
        let mut community = self
            .get_community(handle.clone())
            .expect(format!("Community not found with handle `{}`", handle).as_str());
        if let Some(existing_addon) =
            community.addons.iter_mut().find(|current| current.id == community_addon.id)
        {
            *existing_addon = community_addon;
        } else {
            community.addons.push(community_addon);
        }
        self.update_community(handle, community);
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

    pub fn update_community(
        &mut self,
        handle: CommunityHandle,
        #[allow(unused_mut)] mut community: Community,
    ) {
        let _ = self
            .get_editable_community(&handle)
            .expect("Only community admins and hub moderators can configure communities");

        community.validate();
        community.set_default_admin();

        require!(community.handle == handle, "Community handle cannot be changed");
        require!(env::prepaid_gas() >= UPDATE_COMMUNITY_GAS, "Require at least 30 Tgas");
        self.communities.insert(&handle, &community);
        social_db_contract().with_unused_gas_weight(1).set(json!({
            get_devhub_community_account(&community.handle)
            : {
                "profile": {
                    "name": community.name,
                    "image": {
                        "url": community.logo_url,
                    }
                }
            }
        }));
    }

    pub fn set_community_socialdb(&mut self, handle: CommunityHandle, data: Value) {
        let _ = self
            .get_editable_community(&handle)
            .expect("Only community admins and hub moderators can set community Social DB");

        require!(env::prepaid_gas() >= SET_COMMUNITY_SOCIALDB_GAS, "Require at least 30 Tgas");
        social_db_contract()
            .with_unused_gas_weight(1)
            .set(json!({ get_devhub_community_account(&handle): data }));
    }

    pub fn create_discussion(&mut self, handle: CommunityHandle, block_height: Number) -> Promise {
        require!(env::prepaid_gas() >= CREATE_DISCUSSION_GAS, "Require at least 30 Tgas");

        let post_initiator = env::predecessor_account_id();
        let repost = format!("[{{\"key\":\"main\",\"value\":{{\"type\":\"repost\",\"item\":{{\"type\":\"social\",\"path\":\"{}/post/main\",\"blockHeight\":{}}}}}}},{{\"key\":{{\"type\":\"social\",\"path\":\"{}/post/main\",\"blockHeight\":{}}},\"value\":{{\"type\":\"repost\"}}}}]", post_initiator, block_height, post_initiator, block_height);
        let notify = format!("{{\"key\":\"{}\",\"value\":{{\"type\":\"repost\",\"item\":{{\"type\":\"social\",\"path\":\"{}/post/main\",\"blockHeight\":{}}}}}}}", post_initiator, post_initiator, block_height);
        social_db_contract().with_unused_gas_weight(1).set(
            json!({ get_devhub_discussions_account(&handle): {
              "index": {
                "repost": repost,
                "notify": notify
              }
            } }),
        )
    }

    pub fn delete_community(&mut self, handle: CommunityHandle) {
        require!(
            self.has_moderator(env::predecessor_account_id()),
            "Only moderators can delete community"
        );

        let community = self
            .get_community(handle.clone())
            .expect(&format!("Community with handle `{}` does not exist", handle));

        self.communities.remove(&community.handle);

        require!(env::prepaid_gas() >= DELETE_COMMUNITY_GAS, "Require at least 30 Tgas");
        ext_devhub_community::ext(get_devhub_community_account(&community.handle).parse().unwrap())
            .with_unused_gas_weight(1)
            .destroy();
    }

    pub fn set_featured_communities(&mut self, handles: Vec<CommunityHandle>) {
        require!(
            self.has_moderator(env::predecessor_account_id()),
            "Only moderators can add featured communities"
        );

        // Check if every handle corresponds to an existing community
        for handle in &handles {
            require!(self.communities.get(&handle).is_some(), "Community does not exist.");
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

#[derive(Copy, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
#[schemars(crate = "near_sdk::schemars")]
pub struct BlockHeightCallbackRetValue {
    proposal_id: ProposalId,
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use crate::community::AddOn;
    use crate::{PostBody, ProposalBodyV0, VersionedProposalBody};
    use near_sdk::test_utils::{get_created_receipts, VMContextBuilder};
    use near_sdk::{testing_env, VMContext};
    use serde_json::json;
    use std::collections::HashSet;
    use std::convert::TryInto;

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

    #[allow(dead_code)]
    fn get_context_with_predecessor(is_view: bool, signer: String) -> VMContext {
        VMContextBuilder::new()
            .predecessor_account_id(signer.try_into().unwrap())
            .is_view(is_view)
            .build()
    }

    #[test]
    pub fn test_add_proposal() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Contract::new();

        let body: ProposalBodyV0 = near_sdk::serde_json::from_value(json!({
            "proposal_body_version": "V0",
            "name": "another post",
            "description": "Hello to @petersalomonsen.near and @psalomo.near. This is an idea with mentions.",            "category": "Marketing",
            "summary": "sum",
            "linked_proposals": [1, 3],
            "requested_sponsorship_amount": "1000000000",
            "requested_sponsorship_token": "USD",
            "receiver_account": "polyprogrammist.near",
            "supervisor": "frol.near",
            "requested_sponsor": "neardevdao.near",
            "payouts": [],
            "timeline": {"status": "DRAFT"}
        })).unwrap();
        contract.add_proposal(VersionedProposalBody::V0(body), HashSet::new());
        let receipts = get_created_receipts();
        assert_eq!(3, receipts.len());

        if let near_sdk::mock::MockAction::FunctionCallWeight { method_name, args, .. } =
            &receipts[0].actions[0]
        {
            assert_eq!(method_name, b"set");
            assert_eq!(args, b"{\"data\":{\"bob.near\":{\"index\":{\"notify\":\"[{\\\"key\\\":\\\"petersalomonsen.near\\\",\\\"value\\\":{\\\"type\\\":\\\"devgovgigs/mention\\\",\\\"proposal\\\":0}},{\\\"key\\\":\\\"psalomo.near.\\\",\\\"value\\\":{\\\"type\\\":\\\"devgovgigs/mention\\\",\\\"proposal\\\":0}},{\\\"key\\\":\\\"frol.near\\\",\\\"value\\\":{\\\"type\\\":\\\"devgovgigs/mention\\\",\\\"proposal\\\":0}},{\\\"key\\\":\\\"neardevdao.near\\\",\\\"value\\\":{\\\"type\\\":\\\"devgovgigs/mention\\\",\\\"proposal\\\":0}}]\"}}}}");
        } else {
            assert!(false, "Expected a function call ...")
        }
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

        // Extract the method_name and args values
        if let near_sdk::mock::MockAction::FunctionCallWeight { method_name, args, .. } =
            &receipts[1].actions[0]
        {
            assert_eq!(method_name, b"set");
            assert_eq!(args, b"{\"data\":{\"bob.near\":{\"index\":{\"notify\":\"[{\\\"key\\\":\\\"petersalomonsen.near\\\",\\\"value\\\":{\\\"type\\\":\\\"devgovgigs/mention\\\",\\\"post\\\":0}},{\\\"key\\\":\\\"psalomo.near.\\\",\\\"value\\\":{\\\"type\\\":\\\"devgovgigs/mention\\\",\\\"post\\\":0}}]\"}}}}");
        } else {
            assert!(false, "Expected a function call ...")
        }
    }

    #[test]
    pub fn test_create_addon() {
        let context = get_context_with_current(false, "bob.near".to_string());
        testing_env!(context);

        let mut contract = Contract::new();
        let input = fake_addon("CommunityAddOnId".to_string());
        contract.create_addon(input.to_owned());

        let addon = contract.get_addon("CommunityAddOnId".to_owned());

        assert_eq!(addon, Some(input))
    }

    pub fn fake_addon(id: String) -> AddOn {
        let input = AddOn {
            id: id.to_owned(),
            title: "GitHub AddOn".to_owned(),
            description: "Current status of NEARCORE repo".to_owned(),
            view_widget: "custom-viewer-widget".to_owned(),
            configurator_widget: "github-configurator".to_owned(),
            icon: "bi bi-github".to_owned(),
        };
        return input;
    }

    #[test]
    pub fn test_get_all_addons() {
        let context = get_context_with_current(false, "bob.near".to_string());
        testing_env!(context);
        let mut contract = Contract::new();
        let input = fake_addon("CommunityAddOnId".to_string());
        contract.create_addon(input.to_owned());

        let addons = contract.get_all_addons();

        assert_eq!(addons[0], input)
    }

    #[test]
    pub fn test_get_addon() {
        let context = get_context_with_current(false, "bob.near".to_string());
        testing_env!(context);
        let mut contract = Contract::new();
        let input = fake_addon("CommunityAddOnId".to_string());
        contract.create_addon(input.to_owned());

        let addon = contract.get_addon("CommunityAddOnId".to_owned());

        assert_eq!(addon, Some(input))
    }

    #[test]
    pub fn test_update_addon() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Contract::new();
        let input = fake_addon("test".to_owned());
        contract.create_addon(input.to_owned());

        contract.update_addon(AddOn { title: "Telegram AddOn".to_owned(), ..input });

        let addons = contract.get_all_addons();

        assert_eq!(addons[0].title, "Telegram AddOn".to_owned());
    }
}
