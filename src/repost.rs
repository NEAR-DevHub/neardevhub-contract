use crate::social_db::{ext_social_db, SOCIAL_DB};
use near_sdk::serde_json::json;
use near_sdk::{env, Gas, Promise};
use crate::post::{Post, PostBody};

const GAS_FOR_REPOST: Gas = 10_000_000_000_000;

pub fn repost(post: Post) -> Promise {
    let post_link = format!("https://near.social/#/devgovgigs.near/widget/Post?id={}", post.id);
    let title = match post.snapshot.body.clone() {
        PostBody::Idea(idea) => format!("## Idea: {}\n", idea.latest_version().name),
        PostBody::Submission(submission) => format!("## Solution: {}\n", submission.latest_version().name),
        PostBody::Attestation(attestation) => format!("## Attestation: {}\n", attestation.latest_version().name),
        PostBody::Sponsorship(sponsorship) => format!("## Sponsorship: {}\n", sponsorship.latest_version().name),
        _ => Default::default()
    };

    let desc = match post.snapshot.body.clone() {
        PostBody::Comment(comment) => comment.latest_version().description,
        PostBody::Idea(idea) =>  idea.latest_version().description,
        PostBody::Submission(submission) => submission.latest_version().description,
        PostBody::Attestation(attestation) => attestation.latest_version().description,
        PostBody::Sponsorship(sponsorship) => sponsorship.latest_version().description
    };

    let text = format!("@{author} [Posted on Developer DAO Board]({post_link})\n{title}{desc}", author=post.author_id, post_link=post_link, title=title, desc=desc);

    let main_value = json!({
        "type": "md",
        "text": text
    }).to_string();

    ext_social_db::set(
        json!({
            "data": {
                env::current_account_id(): {
                    "post": {
                        "main": main_value,
                    },
                    "index": {
                        "post": "{\"key\":\"main\",\"value\":{\"type\":\"md\"}}",
                    }
                }
            }
        }),
        &SOCIAL_DB,
        env::attached_deposit(),
        env::prepaid_gas() - GAS_FOR_REPOST,
    )
}
