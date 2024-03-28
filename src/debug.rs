use crate::*;
use near_sdk::near;
use near_sdk::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Stats {
    pub num_posts: u64,
}

#[near]
impl Contract {
    pub fn get_post_to_parent(&self) -> Vec<(PostId, PostId)> {
        let mut res = vec![];
        for child_id in 0..self.posts.len() {
            if let Some(parent_id) = self.post_to_parent.get(&child_id) {
                res.push((child_id, parent_id));
            }
        }
        res
    }

    pub fn get_parent_to_children(&self) -> Vec<(PostId, Vec<PostId>)> {
        let mut res = vec![];
        for parent_id in 0..self.posts.len() {
            if let Some(children_ids) = self.post_to_children.get(&parent_id) {
                res.push((parent_id, children_ids));
            }
        }
        res
    }
}
