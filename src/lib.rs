pub mod access_control;
pub mod community;
pub mod debug;
pub mod migrations;
mod notify;
pub mod post;
pub mod project;
mod repost;
mod social_db;
pub mod stats;
pub mod str_serializers;

use crate::access_control::members::ActionType;
use crate::access_control::members::Member;
use crate::access_control::AccessControl;
use community::CommunityFeatureFlags;
use community::CommunityInputs;
use community::{Community, CommunityHandle, CommunityMetadata, FeaturedCommunity, WikiPage};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, Vector};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault};
use post::*;
use project::Project;
use project::ProjectId;
use project::ProjectInputs;
use project::ProjectMetadata;
use project::ProjectPermissions;
use project::ProjectView;
use project::ProjectViewConfig;
use project::ProjectViewId;
use project::ProjectViewInputs;
use project::ProjectViewMetadata;

use std::collections::HashSet;
use std::convert::identity;

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
    pub last_project_id: usize,
    pub projects: UnorderedMap<ProjectId, Project>,
    pub project_views: UnorderedMap<ProjectViewId, ProjectView>,
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
            last_project_id: 0,
            projects: UnorderedMap::new(StorageKey::Projects),
            project_views: UnorderedMap::new(StorageKey::ProjectViews),
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

    pub fn is_allowed_to_moderate(&self) -> bool {
        let moderators = self.access_control.members_list.get_moderators();
        env::predecessor_account_id() == env::current_account_id()
            || moderators.contains(&Member::Account(env::current_account_id()))
    }

    pub fn is_allowed_to_edit(&self, post_id: PostId, editor: Option<AccountId>) -> bool {
        near_sdk::log!("is_allowed_to_edit");
        let post: Post = self
            .posts
            .get(post_id)
            .unwrap_or_else(|| panic!("Post id {} not found", post_id))
            .into();
        let editor = match editor {
            None => env::predecessor_account_id(),
            Some(e) => e,
        };
        // First check for simple cases.
        if editor == env::current_account_id() || editor == post.author_id {
            return true;
        }

        // Then check for complex case.
        self.access_control
            .members_list
            .check_permissions(editor, post.snapshot.labels.iter().cloned().collect())
            .contains(&ActionType::EditPost)
    }

    pub fn is_allowed_to_use_labels(&self, editor: Option<AccountId>, labels: Vec<String>) -> bool {
        let editor = match editor {
            None => env::predecessor_account_id(),
            Some(e) => e,
        };
        // First check for simple cases.
        if editor == env::current_account_id() {
            return true;
        }
        let restricted_labels = self.access_control.rules_list.find_restricted(labels.clone());
        if restricted_labels.is_empty() {
            return true;
        }
        self.access_control
            .members_list
            .check_permissions(editor, labels)
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
    pub fn add_community(&mut self, handle: CommunityHandle, mut community: CommunityInputs) {
        if self.communities.get(&handle).is_some() {
            panic!("Community already exists");
        }

        let mut new_community = Community {
            handle: handle.clone(),
            admins: community.admins,
            name: community.name,
            description: community.description,
            bio_markdown: community.bio_markdown,
            logo_url: community.logo_url,
            banner_url: community.banner_url,
            tag: community.tag,
            github_handle: community.github_handle,
            telegram_handle: community.telegram_handle,
            twitter_handle: community.twitter_handle,
            website_url: community.website_url,
            github: community.github,
            wiki1: community.wiki1,
            wiki2: community.wiki2,
            project_ids: HashSet::new(),

            feature_flags: CommunityFeatureFlags {
                github_integration: true,
                projects: true,
                sponsorship: true,
                wiki: true,
            },
        };

        new_community.validate();
        new_community.set_default_admin();
        self.communities.insert(&new_community.handle, &new_community);
    }

    fn get_editable_community(&self, handle: &CommunityHandle) -> Option<Community> {
        let caller_account_id = env::predecessor_account_id();
        let community = self.communities.get(&handle).expect("Community does not exist");

        if community.admins.contains(&caller_account_id) || self.has_moderator(caller_account_id) {
            return Some(community);
        } else {
            return Option::None;
        };
    }

    #[allow(unused_mut)]
    pub fn edit_community(&mut self, handle: CommunityHandle, mut community: Community) {
        let target_community = self
            .get_editable_community(&handle)
            .expect("Only community admins and hub moderators can configure communities");

        // Prevent direct manipulations on relations
        community.project_ids = target_community.project_ids;
        community.validate();
        community.set_default_admin();

        if handle == community.handle {
            self.communities.insert(&handle, &community);
        } else {
            if self.communities.get(&community.handle).is_some() {
                panic!("Community handle '{}' is already taken", community.handle);
            }
            self.communities.remove(&handle);
            self.communities.insert(&community.handle, &community);
        }
    }

    pub fn edit_community_github(&mut self, handle: CommunityHandle, github: Option<String>) {
        let mut community = self
            .get_editable_community(&handle)
            .expect("Only community admins and hub moderators can configure GitHub integrations");

        community.github = github;
        self.communities.insert(&handle, &community);
    }

    pub fn edit_community_wiki1(&mut self, handle: CommunityHandle, wiki1: Option<WikiPage>) {
        let mut community = self
            .get_editable_community(&handle)
            .expect("Only community admins and hub moderators can edit wiki");

        community.wiki1 = wiki1;
        self.communities.insert(&handle, &community);
    }

    pub fn edit_community_wiki2(&mut self, handle: CommunityHandle, wiki2: Option<WikiPage>) {
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

        community.project_ids.iter().for_each(|project_id| {
            let maybe_project = self.get_project(*project_id);

            if maybe_project.is_none() {
                return;
            };

            let project = maybe_project.unwrap();

            if project.metadata.owner_community_handles.len() == 1
                && project.metadata.owner_community_handles.contains(&community.handle)
            {
                self.delete_project(project.metadata.id)
            }
        });

        self.communities.remove(&community.handle);
    }

    pub fn get_all_communities(&self) -> Vec<CommunityMetadata> {
        near_sdk::log!("get_all_communities");
        self.communities
            .iter()
            .map(|(handle, community)| CommunityMetadata {
                handle,
                name: community.name,
                description: community.description,
                logo_url: community.logo_url,
                banner_url: community.banner_url,
            })
            .collect()
    }

    pub fn get_community(&self, handle: CommunityHandle) -> Option<Community> {
        self.communities.get(&handle)
    }

    pub fn get_community_metadata(&self, handle: CommunityHandle) -> Option<CommunityMetadata> {
        self.communities.get(&handle).map(|community| CommunityMetadata {
            handle: community.handle,
            name: community.name,
            description: community.description,
            logo_url: community.logo_url,
            banner_url: community.banner_url,
        })
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

    fn has_moderator(&self, account_id: AccountId) -> bool {
        let moderators = self.access_control.members_list.get_moderators();
        moderators.contains(&Member::Account(account_id))
    }

    fn has_community_admin_in(
        &self,
        account_id: AccountId,
        community_handles: &HashSet<CommunityHandle>,
    ) -> bool {
        community_handles
            .iter()
            .map(|handle| self.get_community(handle.to_owned()))
            .filter_map(identity)
            .map(|community| community.admins.contains(&account_id))
            .any(identity)
    }

    pub fn create_project(
        &mut self,
        author_community_handle: CommunityHandle,
        metadata: ProjectInputs,
    ) {
        let mut author_community = self
            .get_editable_community(&author_community_handle)
            .expect("Only community admins and hub moderators can create projects");

        let mut new_project = Project {
            metadata: ProjectMetadata {
                id: self.last_project_id + 1,
                name: metadata.name,
                description: metadata.description,
                tag: metadata.tag,
                owner_community_handles: HashSet::new(),
            },

            view_ids: HashSet::new(),
        };

        new_project.metadata.owner_community_handles.insert(author_community_handle);
        new_project.validate();
        author_community.project_ids.insert(new_project.metadata.id);
        self.projects.insert(&new_project.metadata.id, &new_project);
        self.communities.insert(&author_community.handle, &author_community);
        self.last_project_id = new_project.metadata.id
    }

    pub fn get_project(&self, id: ProjectId) -> Option<Project> {
        self.projects.get(&id)
    }

    pub fn get_account_project_permissions(
        &self,
        account_id: AccountId,
        project_id: ProjectId,
    ) -> ProjectPermissions {
        let project = self
            .get_project(project_id)
            .expect(&format!("Project with id `{}` does not exist", project_id));

        ProjectPermissions {
            can_configure: self.has_community_admin_in(
                account_id.clone(),
                &project.metadata.owner_community_handles,
            ) || self.has_moderator(account_id),
        }
    }

    pub fn update_project_metadata(&mut self, metadata: ProjectMetadata) {
        let mut project = self
            .get_project(metadata.id)
            .expect(&format!("Project with id `{}` does not exist", metadata.id));

        if !self
            .get_account_project_permissions(env::predecessor_account_id(), project.metadata.id)
            .can_configure
        {
            panic!("Only community admins and hub moderators can configure projects");
        }

        project.metadata = metadata;
        project.validate();
        self.projects.insert(&project.metadata.id, &project);
    }

    pub fn delete_project(&mut self, id: ProjectId) {
        let project =
            self.get_project(id).expect(&format!("Project with id `{}` does not exist", id));

        if &project.metadata.owner_community_handles.len() > &1 {
            panic!("Only projects owned by a single community can be deleted");
        }

        let mut owner_community = self
            .get_editable_community(
                &project.metadata.owner_community_handles.into_iter().next().unwrap(),
            )
            .expect("Only community admins and hub moderators can delete projects");

        project.view_ids.iter().for_each(|view_id| {
            self.project_views.remove(view_id);
        });

        self.projects.remove(&id);
        owner_community.project_ids.remove(&id);
        self.communities.insert(&owner_community.handle, &owner_community);
    }

    pub fn get_all_projects_metadata(&self) -> Vec<ProjectMetadata> {
        self.projects.iter().map(|(_, project)| project.metadata).collect()
    }

    pub fn get_community_projects_metadata(
        &self,
        community_handle: CommunityHandle,
    ) -> Vec<ProjectMetadata> {
        self.get_community(community_handle)
            .map(|community| {
                community
                    .project_ids
                    .iter()
                    .filter_map(|id| self.get_project(*id))
                    .map(|project| project.metadata)
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn create_project_view(&mut self, view: ProjectViewInputs) {
        let mut project = self
            .get_project(view.metadata.project_id)
            .expect(&format!("Project with id `{}` does not exist", view.metadata.project_id));

        if !self
            .get_account_project_permissions(env::predecessor_account_id(), project.metadata.id)
            .can_configure
        {
            panic!("Only community admins and hub moderators can create projects");
        }

        let new_project_view = ProjectView {
            config: view.config,

            metadata: ProjectViewMetadata {
                id: format!("{:X}", env::block_timestamp() + self.project_views.len()),
                project_id: project.metadata.id,
                kind: view.metadata.kind,
                title: view.metadata.title,
                description: view.metadata.description,
            },
        };

        project.view_ids.insert(new_project_view.metadata.id.clone());
        self.project_views.insert(&new_project_view.metadata.id, &new_project_view);
        self.projects.insert(&project.metadata.id, &project);
    }

    pub fn get_project_view(&self, id: ProjectViewId) -> Option<ProjectView> {
        self.project_views.get(&id)
    }

    pub fn get_project_view_config(&self, id: ProjectViewId) -> Option<ProjectViewConfig> {
        self.project_views.get(&id).map(|view| view.config)
    }

    pub fn get_project_views_metadata(&self, project_id: ProjectId) -> Vec<ProjectViewMetadata> {
        let project = self
            .get_project(project_id)
            .expect(&format!("Project with id `{}` does not exist", project_id));

        project
            .view_ids
            .iter()
            .filter_map(|id| self.get_project_view(id.to_owned()))
            .map(|view| view.metadata)
            .collect()
    }

    pub fn update_project_view(&mut self, view: ProjectView) {
        let project = self
            .get_project(view.metadata.project_id)
            .expect(&format!("Project with id `{}` does not exist", view.metadata.project_id));

        if !self
            .get_account_project_permissions(env::predecessor_account_id(), project.metadata.id)
            .can_configure
        {
            panic!("Only community admins and hub moderators can update project views");
        }

        let mut project_view = if let Some(project_view) =
            self.get_project_view(view.metadata.id.clone())
        {
            if !project.view_ids.contains(&project_view.metadata.id) {
                panic!(
                    "Project view with id `{view_id}` does not correspond to the project with id `{project_id}`",
                    view_id = view.metadata.id,
										project_id = project.metadata.id
                );
            }

            project_view
        } else {
            panic!("Project view with id `{}` does not exist", view.metadata.id);
        };

        project_view.config = view.config.clone();
        project_view.metadata = view.metadata.clone();
        self.project_views.insert(&view.metadata.id, &project_view);
    }

    pub fn delete_project_view(&mut self, id: ProjectViewId) {
        let project_view = self
            .get_project_view(id.clone())
            .expect(&format!("Project view with id `{}` does not exist", id));

        let mut project = self.get_project(project_view.metadata.project_id).expect(&format!(
            "Project with id `{}` does not exist",
            project_view.metadata.project_id
        ));

        if !self
            .get_account_project_permissions(env::predecessor_account_id(), project.metadata.id)
            .can_configure
        {
            panic!("Only community admins and hub moderators can delete project views");
        }

        self.project_views.remove(&project_view.metadata.id);
        project.view_ids.remove(&project_view.metadata.id);
        self.projects.insert(&project.metadata.id, &project);
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use std::collections::HashSet;
    use std::convert::TryInto;

    use crate::post::PostBody;
    use near_sdk::test_utils::{get_created_receipts, VMContextBuilder};
    use near_sdk::{testing_env, MockedBlockchain, VMContext};
    use regex::Regex;

    use super::Contract;

    fn get_context(is_view: bool) -> VMContext {
        VMContextBuilder::new()
            .signer_account_id("bob.near".try_into().unwrap())
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
}
