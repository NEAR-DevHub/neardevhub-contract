mod test_env;

use {crate::test_env::*, serde_json::json};

#[tokio::test]
async fn test_deploy_contract_self_upgrade() -> anyhow::Result<()> {
    // Test Flow:
    // 1. Deploy devhub and near social contract on sandbox
    // 2. Add all kinds of posts and add a community.
    // 3. Upgrade the contract.
    // 4. Get all the posts and community and check if migration was successful.

    // Initialize the devhub and near social contract on chain,
    // contract is devhub contract instance.
    let contract = init_contracts().await?;

    let deposit_amount = near_units::parse_near!("0.1");

    // Add Posts
    let add_idea_post = contract
        .call("add_post")
        .args_json(json!({
            "parent_id": null,
            "labels": [],
            "body": {
                "name": "This is a test idea.",
                "description": "This is a test description.",
                "post_type": "Idea",
                "idea_version": "V1"
            }
        }))
        .deposit(deposit_amount)
        .transact()
        .await?;

    assert!(add_idea_post.is_success());

    let add_solution_v2_post = contract
        .call("add_post")
        .args_json(json!({
            "parent_id": null,
            "labels": [],
            "body": {
                "name": "Solution Test",
                "description": "This is a test solution post.",
                "post_type": "Solution",
                "requested_sponsor": "neardevgov.near",
                "requested_sponsorship_amount": "1000",
                "requested_sponsorship_token": "NEAR",
                "solution_version": "V2"
            }
        }))
        .deposit(deposit_amount)
        .max_gas()
        .transact()
        .await?;

    assert!(add_solution_v2_post.is_success());

    let add_comment_post = contract
        .call("add_post")
        .args_json(json!({
            "parent_id": 0,
            "labels": [],
            "body": {
                "description": "This is test Comment.",
                "comment_version": "V2",
                "post_type": "Comment"
            }
        }))
        .deposit(deposit_amount)
        .max_gas()
        .transact()
        .await?;

    assert!(add_comment_post.is_success());

    let add_attestation_post = contract
        .call("add_post")
        .args_json(json!({
            "parent_id": 1,
            "labels": [],
            "body": {
                "name": "Attestation",
                "description": "Description",
                "attestation_version": "V1",
                "post_type": "Attestation"
            }
        }))
        .deposit(deposit_amount)
        .max_gas()
        .transact()
        .await?;

    assert!(add_attestation_post.is_success());

    let add_sponsorship_post_with_near = contract
        .call("add_post")
        .args_json(json!({
            "parent_id": 1,
            "labels": [],
            "body": {
                "name": "Contributor fellowship",
                "description": "Funding approved",
                "amount": "1000",
                "sponsorship_token": "NEAR",
                "supervisor": "john.near",
                "sponsorship_version": "V1",
                "post_type": "Sponsorship"
            }
        }))
        .deposit(deposit_amount)
        .max_gas()
        .transact()
        .await?;

    assert!(add_sponsorship_post_with_near.is_success());

    let add_sponsorship_post_with_usd = contract
        .call("add_post")
        .args_json(json!({
            "parent_id": 1,
            "labels": [],
            "body": {
                "name": "Contributor fellowship",
                "description": "Funding approved",
                "amount": "1000",
                "sponsorship_token": "USD",
                "supervisor": "john.near",
                "sponsorship_version": "V1",
                "post_type": "Sponsorship"
            }
        }))
        .deposit(deposit_amount)
        .max_gas()
        .transact()
        .await?;

    assert!(add_sponsorship_post_with_usd.is_success());

    let add_sponsorship_post_with_nep141 = contract
        .call("add_post")
        .args_json(json!({
            "parent_id": 1,
            "labels": [],
            "body": {
                "name": "Contributor fellowship",
                "description": "Funding approved",
                "amount": "1000",
                "sponsorship_token": {
                    "NEP141": {
                        "address": "usdt.tether-token.near"
                    }
                },
                "supervisor": "john.near",
                "sponsorship_version": "V1",
                "post_type": "Sponsorship"
            }
        }))
        .deposit(deposit_amount)
        .max_gas()
        .transact()
        .await?;

    assert!(add_sponsorship_post_with_nep141.is_success());

    // Add a community
    let create_community = contract
        .call("create_community")
        .args_json(json!({
            "inputs": {
                "handle": "gotham",
                "name": "Gotham",
                "tag": "some",
                "description": "This is a test community.",
                "bio_markdown": "This is a sample text about your community.\nYou can change it on the community configuration page.",
                "logo_url": "https://ipfs.near.social/ipfs/bafkreibysr2mkwhb4j36h2t7mqwhynqdy4vzjfygfkfg65kuspd2bawauu",
                "banner_url": "https://ipfs.near.social/ipfs/bafkreic4xgorjt6ha5z4s5e3hscjqrowe5ahd7hlfc5p4hb6kdfp6prgy4"
            }
        }))
        .max_gas()
        .transact()
        .await?;

    assert!(create_community.is_success());

    // Call self upgrade with current branch code
    // compile the current code
    let wasm = near_workspaces::compile_project("./").await?;

    let mut contract_upgrade_result =
        contract.call("unsafe_self_upgrade").args(wasm).max_gas().transact().await?;

    while contract_upgrade_result.json::<String>()? == "needs-migration" {
        contract_upgrade_result =
            contract.call("unsafe_migrate").args_json(json!({})).max_gas().transact().await?;
    }

    let get_idea_post: serde_json::Value = contract
        .call("get_post")
        .args_json(json!({
            "post_id" : 0
        }))
        .view()
        .await?
        .json()?;

    insta::assert_json_snapshot!(get_idea_post, {".snapshot.timestamp" => "[timestamp]"});

    let get_solution_v2_post: serde_json::Value = contract
        .call("get_post")
        .args_json(json!({
            "post_id" : 1
        }))
        .view()
        .await?
        .json()?;

    insta::assert_json_snapshot!(get_solution_v2_post, {".snapshot.timestamp" => "[timestamp]"});

    let get_comment_posts: serde_json::Value = contract
        .call("get_posts")
        .args_json(json!({
            "parent_id" : 0
        }))
        .view()
        .await?
        .json()?;

    insta::assert_json_snapshot!(get_comment_posts, {"[].snapshot.timestamp" => "[timestamp]"});

    let get_attestation_sponsorship_posts: serde_json::Value = contract
        .call("get_posts")
        .args_json(json!({
            "parent_id" : 1
        }))
        .view()
        .await?
        .json()?;

    insta::assert_json_snapshot!(get_attestation_sponsorship_posts, {"[].snapshot.timestamp" => "[timestamp]"});

    let get_sponsorship_post_with_near: serde_json::Value = contract
        .call("get_post")
        .args_json(json!({
            "post_id" : 4
        }))
        .view()
        .await?
        .json()?;

    insta::assert_json_snapshot!(get_sponsorship_post_with_near, {".snapshot.timestamp" => "[timestamp]"});

    let get_sponsorship_post_with_usd: serde_json::Value = contract
        .call("get_post")
        .args_json(json!({
            "post_id" : 5
        }))
        .view()
        .await?
        .json()?;

    insta::assert_json_snapshot!(get_sponsorship_post_with_usd, {".snapshot.timestamp" => "[timestamp]"});

    let get_sponsorship_post_with_nep141: serde_json::Value = contract
        .call("get_post")
        .args_json(json!({
            "post_id" : 6
        }))
        .view()
        .await?
        .json()?;

    insta::assert_json_snapshot!(get_sponsorship_post_with_nep141, {".snapshot.timestamp" => "[timestamp]"});

    let get_community: serde_json::Value = contract
        .call("get_community")
        .args_json(json!({
            "handle" : "gotham"
        }))
        .view()
        .await?
        .json()?;

    insta::assert_json_snapshot!(get_community);

    Ok(())
}
