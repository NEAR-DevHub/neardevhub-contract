use near_sdk::serde_json::json;
use near_sdk::{env, AccountId, Promise};

use devhub_common::social_db_contract;

use crate::Proposal;

pub fn proposal_repost_text(proposal: Proposal) -> String {
    let proposal_link = format!("/devhub.near/widget/app?page=proposal&id={}", proposal.id);

    let title = proposal.snapshot.body.clone().latest_version().name;
    let summary = proposal.snapshot.body.clone().latest_version().summary;
    let category = proposal.snapshot.body.clone().latest_version().category;

    let text = format!(
        "We have just received a new *{category}* proposal.\n\n———\n\n**By**: @{author}\n\n**Title**: “{title}“\n\n**Summary**:\n\n{summary}\n\n———\n\nRead the full proposal and share your feedback on [DevHub]({proposal_link})",
        author = proposal.author_id,
        proposal_link = proposal_link,
        title = title,
        summary = summary,
        category = category
    );

    text
}

fn repost_internal(text: String, contract_address: AccountId) -> near_sdk::serde_json::Value {
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

pub fn publish_to_socialdb_feed(callback: Promise, text: String) -> Promise {
    social_db_contract()
        .with_static_gas(env::prepaid_gas().saturating_div(3))
        .with_attached_deposit(env::attached_deposit())
        .set(repost_internal(text, env::current_account_id()))
        .then(callback)
}
