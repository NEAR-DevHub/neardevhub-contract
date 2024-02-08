use crate::*;
use near_sdk::near_bindgen;
use near_sdk::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
#[schemars(crate = "near_sdk::schemars")]
pub struct Stats {
    pub num_posts: u64,
}

#[near_bindgen]
impl Contract {
    pub fn get_stats(&self) -> Stats {
        Stats { num_posts: self.posts.len() }
    }
}
