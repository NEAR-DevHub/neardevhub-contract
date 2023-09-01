use near_units::parse_near;
use serde_json::json;
use workspaces::AccountId;

const DEVHUB_CONTRACT: &str = "devgovgigs.near";
const NEAR_SOCIAL: &str = "social.near";

#[tokio::test]
async fn test_deploy_contract_self_upgrade() -> anyhow::Result<()> {
    //Test Flow
    // 1.Deploy devhub and near social contract on sandbox
    // 2. Add all kinds of posts and add a community.
    // 3. Upgrade the contract.
    // 4. Get all the posts and community and check if migration was succesfull.

    // Initialize the devhub and near social contract on chain,
    //contract is devhub contract instance.
    let contract = init_contracts().await?;

    let deposit_amount = near_units::parse_near!("0.1");

    //Add Posts
    let idea_post = contract
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

    assert!(idea_post.is_success());

    let submission_post = contract
        .call("add_post")
        .args_json(json!({
            "parent_id": null,
            "labels": [],
            "body": {
            "name": "Solution Test",
            "description": "###### Requested amount: 100 NEAR\n###### Requested sponsor: @neardevgov.near\nThis is a test submission. ",
            "post_type": "Submission",
            "submission_version": "V1"
        }
        }))
        .deposit(deposit_amount)
        .max_gas()
        .transact()
        .await?;

    assert!(submission_post.is_success());

    let comment = contract
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

    assert!(comment.is_success());

    let attestation = contract
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

    assert!(attestation.is_success());

    let sponsorship = contract
        .call("add_post")
        .args_json(json!({
            "parent_id": 1,
            "labels": [],
            "body": {
            "name": "Contributor fellowship",
            "description": "Funding approved",
            "amount": "1000",
            "sponsorship_token": "Near",
            "supervisor": "john.near",
            "sponsorship_version": "V1",
            "post_type": "Sponsorship"
            }
        }))
        .deposit(deposit_amount)
        .max_gas()
        .transact()
        .await?;

    assert!(sponsorship.is_success());

    //Add a community
    let community = contract
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

    println!("{}", format!("{:?}", community));

    assert!(community.is_success());

    //Call self upgrade with current branch code
    //compile the current code
    let wasm = workspaces::compile_project("./").await?;

    let self_upgrade = contract.call("unsafe_self_upgrade").args(wasm).max_gas().transact().await?;

    assert!(self_upgrade.is_success());

    //needs migration or not
    if format!("{:?}", self_upgrade).contains("needs-migration") {
        let migrate =
            contract.call("unsafe_migrate").args_json(json!({})).max_gas().transact().await?;
        assert!(format!("{:?}", migrate).contains("Migration done."));
    } else {
        assert!(format!("{:?}", self_upgrade).contains("Migration done."));
    }

    let get_idea: serde_json::Value = contract
        .call("get_post")
        .args_json(json!({
            "post_id" : 0
        }))
        .view()
        .await?
        .json()?;

    assert!(get_idea.to_string().contains("This is a test idea."));

    let get_comment: serde_json::Value = contract
        .call("get_posts")
        .args_json(json!({
            "parent_id" : 0
        }))
        .view()
        .await?
        .json()?;

    assert!(get_comment.to_string().contains("This is test Comment."));

    let get_submission: serde_json::Value = contract
        .call("get_post")
        .args_json(json!({
            "post_id" : 1
        }))
        .view()
        .await?
        .json()?;

    assert!(get_submission.to_string().contains("This is a test submission"));

    let get_attestation_sponsorship: serde_json::Value = contract
        .call("get_posts")
        .args_json(json!({
            "parent_id" : 1
        }))
        .view()
        .await?
        .json()?;

    assert!(get_attestation_sponsorship.to_string().contains("Attestation"));
    assert!(get_attestation_sponsorship.to_string().contains("Sponsorship"));

    let get_community: serde_json::Value = contract
        .call("get_community")
        .args_json(json!({
            "handle" : "gotham"
        }))
        .view()
        .await?
        .json()?;

    assert!(get_community.to_string().contains("This is a test community."));

    Ok(())
}

async fn init_contracts() -> anyhow::Result<workspaces::Contract> {
    let worker = workspaces::sandbox().await?;
    let mainnet = workspaces::mainnet_archival().await?;

    // NEAR social deployment
    let near_social_id: AccountId = NEAR_SOCIAL.parse()?;
    let near_social = worker
        .import_contract(&near_social_id, &mainnet)
        .initial_balance(parse_near!("10000 N"))
        .transact()
        .await?;
    near_social.call("new").transact().await?.into_result()?;

    // Devhub contract deployment
    let contract_id: AccountId = DEVHUB_CONTRACT.parse()?;
    let contract = worker
        .import_contract(&contract_id, &mainnet)
        .initial_balance(parse_near!("1000 N"))
        .transact()
        .await?;
    let outcome = contract.call("new").args_json(json!({})).transact().await?;
    assert!(outcome.is_success());
    assert!(format!("{:?}", outcome).contains("Migrated to version:"));

    Ok(contract)
}
