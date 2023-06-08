use crate::social_db::{ext_social_db, SOCIAL_DB};
use crate::PostId;
use near_sdk::serde_json::json;
use near_sdk::{env, AccountId, Promise};

pub fn notify_mentions(text: &str, post_id: PostId) {
    let mut mentions = Vec::new();
    let mut mention = String::new();
    let mut recording = false;

    for ch in text.chars() {
        if recording {
            if ch.is_alphanumeric() || ch == '.' {
                mention.push(ch);
            } else {
                if !mention.is_empty() {
                    mentions.push(mention.clone());
                    mention.clear();
                }
                recording = false;
            }
        }

        if ch == '@' {
            recording = true;
        }
    }

    // Push the last mention if it wasn't pushed yet
    if recording && !mention.is_empty() {
        mentions.push(mention);
    }

    for mention in &mentions {
        near_sdk::log!(mention);
        notify_mention(post_id, mention.to_string());
    }
}

pub fn notify_like(post_id: PostId, post_author: AccountId) -> Promise {
    notify(post_id, post_author, "like")
}

pub fn notify_mention(post_id: PostId, mentioned_account: AccountId) -> Promise {
    notify(post_id, mentioned_account, "mention")
}

pub fn notify_reply(post_id: PostId, post_author: AccountId) -> Promise {
    notify(post_id, post_author, "reply")
}

pub fn notify_edit(post_id: PostId, post_author: AccountId) -> Promise {
    notify(post_id, post_author, "edit")
}

fn notify(post_id: PostId, post_author: AccountId, action: &str) -> Promise {
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
        env::prepaid_gas() / 4,
    )
}
