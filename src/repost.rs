use crate::post::{get_post_description, Post, PostBody};
use devhub_common::social_db_contract;
use near_sdk::serde_json::json;
use near_sdk::{env, AccountId, Promise};

fn repost_internal(post: Post, contract_address: AccountId) -> near_sdk::serde_json::Value {
    let post_link = format!("/devhub.near/widget/app?page=post&id={}", post.id);
    let title = match post.snapshot.body.clone() {
        PostBody::Idea(idea) => format!("## Idea: {}\n", idea.latest_version().name),
        PostBody::Solution(solution) => {
            format!("## Solution: {}\n", solution.latest_version().name)
        }
        PostBody::Attestation(attestation) => {
            format!("## Attestation: {}\n", attestation.latest_version().name)
        }
        PostBody::Sponsorship(sponsorship) => {
            format!("## Sponsorship: {}\n", sponsorship.latest_version().name)
        }
        _ => Default::default(),
    };

    let desc = get_post_description(post.clone());

    let text = format!(
        "@{author} [Posted on DevHub]({post_link})\n{title}{desc}",
        author = post.author_id,
        post_link = post_link,
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

pub fn repost(post: Post) -> Promise {
    social_db_contract()
        .with_static_gas(env::prepaid_gas().saturating_div(4))
        .with_attached_deposit(env::attached_deposit())
        .set(repost_internal(post, env::current_account_id()))
}

#[cfg(test)]
mod tests {
    use crate::post::{IdeaV1, Post, PostBody, PostSnapshot, VersionedIdea};
    use crate::repost::repost_internal;
    use near_sdk::serde_json::json;

    #[test]
    pub fn check_formatting() {
        let post = Post {
            id: 0,
            author_id: "neardevgov.near".parse().unwrap(),
            likes: Default::default(),
            snapshot: PostSnapshot {
                editor_id: "neardevgov.near".parse().unwrap(),
                timestamp: 0,
                labels: Default::default(),
                body: PostBody::Idea(VersionedIdea::V1(IdeaV1 { name: "A call for Zero Knowledge Work Group members!".to_string(), description: "We are excited to create a more formal Zero Knowledge Work Group (WG) to oversee official decisions on Zero Knowledge proposals. We’re looking for 3-7 experts to participate. Reply to the post if you’re interested in becoming a work group member.".to_string() })),
            },
            snapshot_history: vec![],
        };

        let call_args = repost_internal(post, "devhub.near".parse().unwrap());
        let expected = json!({
            "devhub.near": {
                "post": {
                  "main": "{\"type\":\"md\",\"text\":\"@neardevgov.near [Posted on DevHub](/devhub.near/widget/app?page=post&id=0)\\n## Idea: A call for Zero Knowledge Work Group members!\\nWe are excited to create a more formal Zero Knowledge Work Group (WG) to oversee official decisions on Zero Knowledge proposals. We’re looking for 3-7 experts to participate. Reply to the post if you’re interested in becoming a work group member.\"}"
                },
                "index": {
                  "post": "{\"key\":\"main\",\"value\":{\"type\":\"md\"}}"
                }
              }
        });
        assert_eq!(call_args, expected);
    }
}
