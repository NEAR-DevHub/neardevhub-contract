use crate::social_db::{ext_social_db, SOCIAL_DB};
use crate::PostId;
use near_sdk::serde_json::json;
use near_sdk::{env, AccountId, Gas, Promise};

const GAS_FOR_LIKE: Gas = 10_000_000_000_000;
const GAS_FOR_REPLY: Gas = 10_000_000_000_000;
const GAS_FOR_EDIT: Gas = 10_000_000_000_000;

pub fn notify_like(post_id: PostId, post_author: AccountId) -> Promise {
    notify(post_id, post_author, "like", GAS_FOR_LIKE)
}

pub fn notify_reply(post_id: PostId, post_author: AccountId) -> Promise {
    notify(post_id, post_author, "reply", GAS_FOR_REPLY)
}

pub fn notify_edit(post_id: PostId, post_author: AccountId) -> Promise {
    notify(post_id, post_author, "edit", GAS_FOR_EDIT)
}

fn notify(post_id: PostId, post_author: AccountId, action: &str, action_gas: Gas) -> Promise {
    ext_social_db::set(
        json!({
            env::predecessor_account_id() : {
                "index": {
                    "notify": json!({
                        "key": post_author,
                        "value": {
                            "type": format!("devgovgigs/{}", action),
                            "post": post_id,
                        },
                    }).to_string()
                }
            }
        }),
        &SOCIAL_DB,
        env::attached_deposit(),
        env::prepaid_gas() - action_gas,
    )
}
