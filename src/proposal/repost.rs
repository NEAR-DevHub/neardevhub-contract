use near_sdk::serde_json::json;
use near_sdk::{env, AccountId, Promise};

use crate::{get_proposal_description, social_db::social_db_contract, Proposal};

fn repost_internal(proposal: Proposal, contract_address: AccountId) -> near_sdk::serde_json::Value {
    let proposal_link = format!("/devhub.near/widget/app?page=proposal&id={}", proposal.id);
    let title = proposal.snapshot.body.clone().latest_version().name;

    let desc = get_proposal_description(proposal.snapshot.body.clone());

    let text = format!(
        "@{author} [Created the proposal on DevHub]({post_link})\n{title}{desc}",
        author = proposal.author_id,
        post_link = proposal_link,
        title = title,
        desc = desc
    );

    let main_value = json!({
        "type": "md",
        "text": text
    });

    json!({
        contract_address: {
            "post": {
                "main": main_value.to_string(),
            },
            "index": {
                "post": "{\"key\":\"main\",\"value\":{\"type\":\"md\"}}",
            }
        }
    })
}

pub fn publish_to_socialdb_feed(callback: Promise, proposal: Proposal) -> Promise {
    social_db_contract()
        .with_static_gas(env::prepaid_gas().saturating_div(3))
        .with_attached_deposit(env::attached_deposit())
        .set(repost_internal(proposal, env::current_account_id()))
        .then(callback)
}
