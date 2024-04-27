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

    let _set_categories = contract
        .call("set_allowed_categories")
        .args_json(json!({"new_categories": ["Marketing", "Events"]}))
        .max_gas()
        .deposit(NearToken::from_near(1))
        .transact()
        .await?;

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

    let get_rfp_with_new_category: serde_json::Value = contract
        .call("get_rfp")
        .args_json(json!({
            "rfp_id" : 0
        }))
        .view()
        .await?
        .json()?;

    assert_eq!(get_rfp_with_new_category["snapshot"]["category"], "Events");

    let _add_second_rfp = contract
        .call("add_rfp")
        .args_json(json!({
            "body": {
                "rfp_body_version": "V0",
                "name": "Another RFP",
                "description": "another description",
                "category": "Events",
                "summary": "sum",
                "timeline": {"status": "ACCEPTING_SUBMISSIONS"},
                "submission_deadline": "1707821848175250170"
            },
            "labels": ["test3"],
        }))
        .max_gas()
        .deposit(deposit_amount)
        .transact()
        .await?;

    let get_rfps =
        contract.call("get_rfps").args_json(json!({})).view().await?.json::<Value>()?;

    let rfps_array = get_rfps.as_array().unwrap();

    assert_eq!(rfps_array.len(), 2);
    assert_eq!(rfps_array.get(1).unwrap()["snapshot"]["name"], "Another RFP");

    let get_rfp_ids =
        contract.call("get_all_rfp_ids").args_json(json!({})).view().await?.json::<Value>()?;

    let rfp_ids = get_rfp_ids
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.clone().as_u64().unwrap())
        .collect::<Vec<_>>();

    let expected_ids: Vec<u64> = [0u64, 1u64].to_vec();

    assert_eq!(rfp_ids, expected_ids);

    let second_account = worker
        .root_account()?
        .create_subaccount("second")
        .initial_balance(NearToken::from_near(20))
        .transact()
        .await?
        .into_result()?;

    let _second_author_add_rfp = second_account
        .call(contract.id(), "add_rfp")
        .args_json(json!({
            "body": {
                "rfp_body_version": "V0",
                "name": "Another Author",
                "description": "another description",
                "category": "Events",
                "summary": "sum",
                "timeline": {"status": "ACCEPTING_SUBMISSIONS"},
                "submission_deadline": "1707821848175250170"
            },
            "labels": ["test2", "test3"],
        }))
        .max_gas()
        .deposit(NearToken::from_near(1))
        .transact()
        .await?;    

    assert!(_second_author_add_rfp.is_failure());

    let get_rfps_by_label = contract
        .call("get_rfps_by_label")
        .args_json(json!({
            "label": "test2"
        }))
        .view()
        .await?
        .json::<Value>()?;

    let rfp_ids_by_label = get_rfps_by_label
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_u64().unwrap())
        .collect::<Vec<_>>();

    let expected_ids: Vec<u64> = [0u64].to_vec();
    assert_eq!(rfp_ids_by_label, expected_ids);

    let get_all_rfp_labels = contract
        .call("get_all_rfp_labels")
        .args_json(json!({}))
        .view()
        .await?
        .json::<Value>()?;

    let rfp_labels = get_all_rfp_labels
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap())
        .collect::<Vec<_>>();

    let expected_labels: Vec<&str> = ["test1", "test2", "test3"].to_vec();
    assert_eq!(rfp_labels, expected_labels);

    let is_allowed_to_edit_rfp_false = contract
        .call("is_allowed_to_write_rfps")
        .args_json(json!({
            "rfp_id": 0,
            "editor": "second.test.near"
        }))
        .view()
        .await?
        .json::<Value>()?;

    assert!(!is_allowed_to_edit_rfp_false.as_bool().unwrap());

    let is_allowed_to_edit_rfp_true = contract
        .call("is_allowed_to_write_rfps")
        .args_json(json!({
            "rfp_id": 0,
            "editor": "devhub.near"
        }))
        .view()
        .await?
        .json::<Value>()?;

    assert!(is_allowed_to_edit_rfp_true.as_bool().unwrap());

    let get_all_allowed_rfp_labels = contract
        .call("get_all_allowed_rfp_labels")
        .args_json(json!({
            "editor": "devhub.near"
        }))
        .view()
        .await?
        .json::<Value>()?;

    let allowed_rfp_labels = get_all_allowed_rfp_labels
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap())
        .collect::<Vec<_>>();

    let expected_labels: Vec<&str> = ["test1", "test2", "test3"].to_vec();
    assert_eq!(allowed_rfp_labels, expected_labels);

    let _edit_rfp_timeline_evaluation = contract
        .call("edit_rfp_timeline")
        .args_json(json!({
            "id": 0,
            "timeline": {"status": "EVALUATION" }
        }))
        .max_gas()
        .deposit(deposit_amount)
        .transact()
        .await?;

    println!("{:?}", _edit_rfp_timeline_evaluation);

    assert!(_edit_rfp_timeline_evaluation.is_success());

    let _edit_rfp_timeline_proposal_selected = contract
        .call("edit_rfp_timeline")
        .args_json(json!({
            "id": 0,
            "timeline": {"status": "PROPOSAL_SELECTED" }
        }))
        .max_gas()
        .deposit(deposit_amount)
        .transact()
        .await?;

    assert!(_edit_rfp_timeline_proposal_selected.is_success());

    let _edit_rfp_timeline_cancelled = contract
        .call("edit_rfp_timeline")
        .args_json(json!({
            "id": 0,
            "timeline": {"status": "CANCELLED" }
        }))
        .max_gas()
        .deposit(deposit_amount)
        .transact()
        .await?;

    assert!(_edit_rfp_timeline_cancelled.is_success());

    let _add_rfp_incorrect_category = contract
        .call("add_rfp")
        .args_json(json!({
            "body": {
                "rfp_body_version": "V0",
                "name": "Some RFP",
                "description": "some description",
                "category": "NotExistingCategory",
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

    assert!(_edit_rfp_timeline_cancelled.is_success());

    Ok(())
}