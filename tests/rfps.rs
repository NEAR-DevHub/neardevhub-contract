mod test_env;

use near_sdk::NearToken;
use serde_json::Value;
use {crate::test_env::*, serde_json::json};

#[tokio::test]
async fn test_rfp() -> anyhow::Result<()> {
    // Initialize the devhub and near social contract on chain,
    // contract is devhub contract instance.
    let (contract, worker, _) = init_contracts_from_res().await?;

    let deposit_amount = NearToken::from_near(2);

    let _set_global_labels = contract
        .call("set_global_labels")
        .args_json(json!({
            "labels": [
                {
                    "value": "test1",
                    "title": "test1 description",
                    "color": [255, 0, 0]
                },
                {
                    "value": "test2",
                    "title": "test2 description",
                    "color": [0, 255, 0]
                },
                {
                    "value": "test3",
                    "title": "test3 description",
                    "color": [0, 0, 255]
                }
            ]
        }))
        .max_gas()
        .deposit(deposit_amount)
        .transact()
        .await?;

    let _add_rfp = contract
        .call("add_rfp")
        .args_json(json!({
            "body": {
                "rfp_body_version": "V0",
                "name": "Some RFP",
                "description": "some description",
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

    assert_eq!(get_rfp["snapshot"]["summary"], "sum");

    let social_db_post_block_height: u64 =
        get_rfp["social_db_post_block_height"].as_str().unwrap().parse::<u64>()?;
    assert!(social_db_post_block_height > 0);

    let _edit_rfp = contract
        .call("edit_rfp")
        .args_json(json!({
            "id": 0,
            "body": {
                "rfp_body_version": "V0",
                "name": "Some RFP",
                "description": "another description",
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

    let get_edited_rfp: serde_json::Value = contract
        .call("get_rfp")
        .args_json(json!({
            "rfp_id" : 0
        }))
        .view()
        .await?
        .json()?;

    assert_eq!(get_edited_rfp["snapshot"]["description"], "another description");

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

    let get_rfps = contract.call("get_rfps").args_json(json!({})).view().await?.json::<Value>()?;

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

    let _add_rfp_incorrect_label = contract
        .call("add_rfp")
        .args_json(json!({
            "body": {
                "rfp_body_version": "V0",
                "name": "Some RFP",
                "description": "some description",
                "summary": "sum",
                "timeline": {"status": "ACCEPTING_SUBMISSIONS"},
                "submission_deadline": "1707821848175250170"
            },
            "labels": ["test4"],
        }))
        .max_gas()
        .deposit(deposit_amount)
        .transact()
        .await?;

    assert!(_add_rfp_incorrect_label.is_failure());

    let _add_rfp_proposal = second_account
        .call(contract.id(), "add_proposal")
        .args_json(json!({
            "body": {
                "proposal_body_version": "V0",
                "name": "RFP-proposal",
                "description": "some description",
                "category": "Marketing",
                "summary": "sum",
                "linked_proposals": [1, 3],
                "requested_sponsorship_usd_amount": "1000000000",
                "requested_sponsorship_paid_in_currency": "USDT",
                "receiver_account": "polyprogrammist.near",
                "supervisor": "frol.near",
                "requested_sponsor": "neardevdao.near",
                "timeline": {"status": "DRAFT"},
            },
            "labels": ["test1"],
        }))
        .max_gas()
        .deposit(deposit_amount)
        .transact()
        .await?;

    let _edit_proposal_linked_rfp_incorrect = contract
        .call("edit_proposal_linked_rfp")
        .args_json(json!({
            "id": 0,
            "rfp_id": 2,
        }))
        .max_gas()
        .deposit(deposit_amount)
        .transact()
        .await?;

    assert!(_edit_proposal_linked_rfp_incorrect.is_failure());

    let _edit_proposal_linked_rfp = second_account
        .call(contract.id(), "edit_proposal_linked_rfp")
        .args_json(json!({
            "id": 0,
            "rfp_id": 0,
        }))
        .max_gas()
        .deposit(deposit_amount)
        .transact()
        .await?;

    let get_proposal: serde_json::Value = contract
        .call("get_proposal")
        .args_json(json!({
            "proposal_id" : 0
        }))
        .view()
        .await?
        .json()?;

    let proposal_labels = get_proposal["snapshot"]["labels"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap())
        .collect::<Vec<_>>();

    let expected_labels: Vec<&str> = ["test1", "test2"].to_vec();
    assert_eq!(proposal_labels, expected_labels);

    assert_eq!(get_proposal["snapshot"]["linked_rfp"], 0);

    let _edit_rfp_labels = contract
        .call("edit_rfp")
        .args_json(json!({
            "id": 0,
            "body": {
                "rfp_body_version": "V0",
                "name": "Some RFP",
                "description": "some description",
                "summary": "sum",
                "timeline": {"status": "ACCEPTING_SUBMISSIONS"},
                "submission_deadline": "1707821848175250170"
            },
            "labels": ["test2", "test3"],
        }))
        .max_gas()
        .deposit(deposit_amount)
        .transact()
        .await?;

    let get_proposal: serde_json::Value = contract
        .call("get_proposal")
        .args_json(json!({
            "proposal_id" : 0
        }))
        .view()
        .await?
        .json()?;

    let proposal_labels = get_proposal["snapshot"]["labels"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap())
        .collect::<Vec<_>>();

    let expected_labels: Vec<&str> = ["test3", "test2"].to_vec();
    assert_eq!(proposal_labels, expected_labels);

    let edit_proposal = contract
        .call("edit_proposal")
        .args_json(json!({
            "id": 0,
            "body": {
                "proposal_body_version": "V1",
                "name": "RFP-proposal",
                "description": "some description",
                "category": "Marketing",
                "summary": "sum",
                "linked_proposals": [1, 3],
                "requested_sponsorship_usd_amount": "1000000000",
                "requested_sponsorship_paid_in_currency": "USDT",
                "receiver_account": "polyprogrammist.near",
                "supervisor": "frol.near",
                "requested_sponsor": "neardevdao.near",
                "timeline": {"status": "DRAFT"},
                "linked_rfp": 0,
            },
            "labels": [],
        }))
        .max_gas()
        .deposit(deposit_amount)
        .transact()
        .await?;

    println!("edit_proposal: {:?}", edit_proposal);

    assert!(edit_proposal.is_success());

    let get_proposal: serde_json::Value = contract
        .call("get_proposal")
        .args_json(json!({
            "proposal_id" : 0
        }))
        .view()
        .await?
        .json()?;

    let proposal_labels = get_proposal["snapshot"]["labels"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap())
        .collect::<Vec<_>>();

    let expected_labels: Vec<&str> = ["test3", "test2"].to_vec();
    assert_eq!(proposal_labels, expected_labels);

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

    assert!(_edit_rfp_timeline_proposal_selected.is_failure());

    let _approve_proposal = contract
        .call("edit_proposal_timeline")
        .args_json(json!({
            "id": 0,
            "timeline": {"status": "APPROVED", "sponsor_requested_review": true, "reviewer_completed_attestation": false }
        }))
        .max_gas()
        .deposit(deposit_amount)
        .transact()
        .await?;

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

    let _edit_rfp_timeline_accepting_submissions = contract
        .call("edit_rfp_timeline")
        .args_json(json!({
            "id": 0,
            "timeline": {"status": "ACCEPTING_SUBMISSIONS" }
        }))
        .max_gas()
        .deposit(deposit_amount)
        .transact()
        .await?;

    let _edit_proposal_unlink_rfp = contract
        .call("edit_proposal_linked_rfp")
        .args_json(json!({
            "id": 0,
            "rfp_id": 1,
        }))
        .max_gas()
        .deposit(deposit_amount)
        .transact()
        .await?;

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

    let _edit_proposal_linked_rfp_incorrect_unlink = second_account
        .call(contract.id(), "edit_proposal_linked_rfp")
        .args_json(json!({
            "id": 0,
            "rfp_id": 1,
        }))
        .max_gas()
        .deposit(deposit_amount)
        .transact()
        .await?;

    assert!(_edit_proposal_linked_rfp_incorrect_unlink.is_failure());

    let get_global_labels =
        contract.call("get_global_labels").args_json(json!({})).view().await?.json::<Value>()?;

    let labels_array = get_global_labels.as_array().unwrap();

    assert_eq!(labels_array.len(), 3);

    let _cancel_rfp = contract
        .call("cancel_rfp")
        .args_json(json!({
            "id": 0,
            "proposals_to_cancel": [0],
            "proposals_to_unlink": [],
        }))
        .max_gas()
        .deposit(deposit_amount)
        .transact()
        .await?;

    println!("_cancel_rfp: {:?}", _cancel_rfp);

    assert!(_cancel_rfp.is_success());

    Ok(())
}
