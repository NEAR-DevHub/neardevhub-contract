use near_sdk::serde_json::json;
use near_sdk::{env, AccountId, Promise};

use crate::{social_db::social_db_contract, Proposal};

fn repost_internal(proposal: Proposal, contract_address: AccountId) -> near_sdk::serde_json::Value {
    let proposal_link = format!("/devhub.near/widget/app?page=proposal&id={}", proposal.id);
    let title = proposal.snapshot.body.clone().latest_version().name;

    let summary = proposal.snapshot.body.clone().latest_version().summary;

    let text = format!(
        "There is a new proposal on DevHub from @{author}: “{title}“\n> {summary}\n__Read the full proposal [here]({proposal_link})__",
        author = proposal.author_id,
        proposal_link = proposal_link,
        title = title,
        summary = summary
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
