use std::collections::HashSet;

use crate::{
    get_subscribers, rfp::get_subscribers as get_rfp_subscribers, PostId, Proposal, ProposalId, RFP,
};
use devhub_common::social_db_contract;
use near_sdk::serde_json::json;
use near_sdk::{env, AccountId, Promise};

pub fn get_text_mentions(text: &str) -> Vec<String> {
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

    mentions
}

pub fn notify_accounts(
    notifier: AccountId,
    accounts: Vec<String>,
    notify_value: serde_json::Value,
) -> Promise {
    if !accounts.is_empty() {
        let mut notify_values = Vec::new();

        for account in accounts {
            notify_values.push(json!({
                "key": account,
                "value": notify_value,
            }));
        }

        social_db_contract()
            .with_static_gas(env::prepaid_gas().saturating_div(4))
            .with_attached_deposit(env::attached_deposit())
            .set(json!({
                notifier : {
                    "index": {
                        "notify": json!(notify_values).to_string()
                    }
                }
            }))
    } else {
        Promise::new(env::current_account_id())
    }
}

pub fn notify_proposal_subscribers(proposal: &Proposal) -> Promise {
    let accounts = get_subscribers(&proposal.snapshot.body.clone().latest_version());

    notify_accounts(
        env::current_account_id(),
        accounts,
        json!({
            "type": "proposal/mention",
            "proposal": proposal.id,
            "widgetAccountId": env::current_account_id(),
            "notifier": env::predecessor_account_id(),
        }),
    )
}

pub fn notify_rfp_subscribers(rfp: &RFP, additional_accounts: HashSet<AccountId>) -> Promise {
    let accounts = [
        get_rfp_subscribers(&rfp.snapshot.body.clone().latest_version()),
        additional_accounts.iter().map(|x| x.to_string()).collect::<Vec<_>>(),
    ]
    .concat();

    notify_accounts(
        env::current_account_id(),
        accounts,
        json!({
            "type": "rfp/mention",
            "rfp": rfp.id,
            "widgetAccountId": env::current_account_id(),
            "notifier": env::current_account_id(),
        }),
    )
}

pub fn notify_mentions(text: &str, post_id: PostId) -> Promise {
    let mentions = get_text_mentions(text);

    notify_accounts(
        env::predecessor_account_id(),
        mentions,
        json!({
            "type": "devgovgigs/mention",
            "post": post_id,
        }),
    )
}

pub fn notify_like(post_id: PostId, post_author: AccountId) -> Promise {
    notify(env::predecessor_account_id(), post_author, notify_value(post_id, "like"))
}

pub fn notify_reply(post_id: PostId, post_author: AccountId) -> Promise {
    notify(env::predecessor_account_id(), post_author, notify_value(post_id, "reply"))
}

pub fn notify_edit(post_id: PostId, post_author: AccountId) -> Promise {
    notify(env::predecessor_account_id(), post_author, notify_value(post_id, "edit"))
}

pub fn notify_edit_proposal(proposal_id: ProposalId, post_author: AccountId) -> Promise {
    notify(
        env::current_account_id(),
        post_author,
        json!({
            "type": "proposal/edit",
            "proposal": proposal_id,
            "widgetAccountId": env::current_account_id(),
            "notifier": env::predecessor_account_id(),
        }),
    )
}

fn notify_value(post_id: PostId, action: &str) -> serde_json::Value {
    json!({
        "type": format!("devgovgigs/{}", action),
        "post": post_id,
    })
}

fn notify(notifier: AccountId, post_author: AccountId, notify_value: serde_json::Value) -> Promise {
    social_db_contract()
        .with_static_gas(env::prepaid_gas().saturating_div(4))
        .with_attached_deposit(env::attached_deposit())
        .set(json!({
            notifier : {
                "index": {
                    "notify": json!({
                        "key": post_author,
                        "value": notify_value,
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

        if let near_sdk::mock::MockAction::FunctionCallWeight { method_name, args, .. } =
            &receipts[0].actions[0]
        {
            assert_eq!(method_name, b"set");
            assert_eq!(args, b"{\"data\":{\"bob.near\":{\"index\":{\"notify\":\"[{\\\"key\\\":\\\"a.near\\\",\\\"value\\\":{\\\"type\\\":\\\"devgovgigs/mention\\\",\\\"post\\\":2}},{\\\"key\\\":\\\"bcdefg.near\\\",\\\"value\\\":{\\\"type\\\":\\\"devgovgigs/mention\\\",\\\"post\\\":2}}]\"}}}}");
        } else {
            assert!(false, "Expected a function call ...")
        }
    }

    #[test]
    pub fn test_no_mentions() {
        let context = get_context(false);
        testing_env!(context);
        let text = "Not mentioning anyone";
        notify_mentions(text, 2);
        assert_eq!(1, get_created_receipts().len());
        assert_eq!(0, get_created_receipts()[0].actions.len());
    }
}
