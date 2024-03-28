use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{near, NearSchema};

use crate::*;

#[derive(Serialize, Deserialize, NearSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct Stats {
    pub num_posts: u64,
}

#[near]
impl Contract {
    pub fn get_stats(&self) -> Stats {
        Stats { num_posts: self.posts.len() }
    }
}
