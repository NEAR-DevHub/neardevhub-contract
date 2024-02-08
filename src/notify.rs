use crate::social_db::social_db_contract;
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

    if mentions.len() > 0 {
        let mut notify_values = Vec::new();

        for mention in mentions {
            notify_values.push(json!({
                "key": mention,
                "value": {
                    "type": "devgovgigs/mention",
                    "post": post_id,
                }
            }));
        }

        social_db_contract()
            .with_static_gas(env::prepaid_gas().saturating_div(4))
            .with_attached_deposit(env::attached_deposit())
            .set(json!({
            env::predecessor_account_id() : {
                "index": {
                    "notify": json!(notify_values).to_string()
                } }
            }));
    }
}

pub fn notify_like(post_id: PostId, post_author: AccountId) -> Promise {
    notify(post_id, post_author, "like")
}

pub fn notify_reply(post_id: PostId, post_author: AccountId) -> Promise {
    notify(post_id, post_author, "reply")
}

pub fn notify_edit(post_id: PostId, post_author: AccountId) -> Promise {
    notify(post_id, post_author, "edit")
}

fn notify(post_id: PostId, post_author: AccountId, action: &str) -> Promise {
    social_db_contract()
        .with_static_gas(env::prepaid_gas().saturating_div(4))
        .with_attached_deposit(env::attached_deposit())
        .set(json!({
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
        }))
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::notify_mentions;

    use near_sdk::test_utils::{get_created_receipts, VMContextBuilder};
    use near_sdk::{testing_env, VMContext};

    use regex::Regex;

    fn get_context(is_view: bool) -> VMContext {
        VMContextBuilder::new()
            .signer_account_id("bob.near".parse().unwrap())
            .is_view(is_view)
            .build()
    }

    #[test]
    pub fn test_notify_mentions() {
        let context = get_context(false);
        testing_env!(context);
        let text = "Mentioning @a.near and @bcdefg.near";
        notify_mentions(text, 2);
        let receipts = get_created_receipts();
        assert_eq!(1, receipts.len());

        let receipt = receipts.get(0).unwrap();
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

            assert_eq!("{\"data\":{\"bob.near\":{\"index\":{\"notify\":\"[{\\\"key\\\":\\\"a.near\\\",\\\"value\\\":{\\\"type\\\":\\\"devgovgigs/mention\\\",\\\"post\\\":2}},{\\\"key\\\":\\\"bcdefg.near\\\",\\\"value\\\":{\\\"type\\\":\\\"devgovgigs/mention\\\",\\\"post\\\":2}}]\"}}}}", args);
        }
    }

    #[test]
    pub fn test_no_mentions() {
        let context = get_context(false);
        testing_env!(context);
        let text = "Not mentioning anyone";
        notify_mentions(text, 2);
        assert_eq!(0, get_created_receipts().len());
    }
}
