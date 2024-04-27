mod test_env;

use near_sdk::NearToken;
use serde_json::Value;
use {crate::test_env::*, serde_json::json};

#[tokio::test]
async fn test_rfp() -> anyhow::Result<()> {
    // Initialize the devhub and near social contract on chain,
    // contract is devhub contract instance.
    let (contract, worker, near_social) = init_contracts_from_res().await?;

    let deposit_amount = NearToken::from_near(2);

    let _add_rfp = contract
        .call("add_rfp")
        .args_json(json!({
            "body": {
                "rfp_body_version": "V0",
                "name": "Some RFP",
                "description": "some description",
                "category": "Marketing",
                "summary": "sum",
                "timeline": {"status": "ACCEPTING_SUBMISSIONS"},
                "submission_deadline": "1707821848175250170"
            },
            "labels": ["test1", "test2"],
        }))
        .max_gas()
        .deposit(deposit_amount)
        .transact()
        .await?;

    let get_rfp: serde_json::Value = contract
        .call("get_rfp")
        .args_json(json!({
            "rfp_id" : 0
        }))
        .view()
        .await?
        .json()?;

    assert_eq!(get_rfp["snapshot"]["category"], "Marketing");

    let social_db_post_block_height: u64 =
        get_rfp["social_db_post_block_height"].as_str().unwrap().parse::<u64>()?;
    assert!(social_db_post_block_height > 0);

    let _edit_rfp_category = contract
        .call("edit_rfp")
        .args_json(json!({
            "id": 0,
            "body": {
                "rfp_body_version": "V0",
                "name": "Some RFP",
                "description": "some description",
                "category": "Events",
                "summary": "sum",
                "timeline": {"status": "ACCEPTING_SUBMISSIONS"},
                "submission_deadline": "1707821848175250170"
            },
            "labels": ["test1", "test2"],
        }))
        .max_gas()
        .deposit(deposit_amount)
        .transact()
        .await?;

    println!("edit rfp category: {:?}", _edit_rfp_category);

    let get_rfp_with_new_category: serde_json::Value = contract
        .call("get_rfp")
        .args_json(json!({
            "rfp_id" : 0
        }))
        .view()
        .await?
        .json()?;

    assert_eq!(get_rfp_with_new_category["snapshot"]["category"], "Events");


    Ok(())
}