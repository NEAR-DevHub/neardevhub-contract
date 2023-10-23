mod test_env;

use crate::test_env::*;
use serde_json::json;

#[tokio::test]
async fn test_solution_posts() -> anyhow::Result<()> {
    let deposit_amount = near_units::parse_near!("0.1");

    let contract = init_contracts().await?;

    // Call self upgrade with current branch code
    // compile the current code
    let wasm = near_workspaces::compile_project("./").await?;

    let mut contract_upgrade_result =
        contract.call("unsafe_self_upgrade").args(wasm).max_gas().transact().await?;

    while contract_upgrade_result.json::<String>()? == "needs-migration" {
        contract_upgrade_result =
            contract.call("unsafe_migrate").args_json(json!({})).max_gas().transact().await?;
    }

    let add_solution_post = contract
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
																				"sponsorship_token": "NEAR",
																				"solution_version": "V2"
												}
				}))
				.deposit(deposit_amount)
				.max_gas()
				.transact()
				.await?;

    println!("add_solution_post outcome: {:#?}", add_solution_post);
    assert!(add_solution_post.is_success());

    let get_solution_post: serde_json::Value = contract
        .call("get_post")
        .args_json(json!({
                                        "post_id" : 1
        }))
        .view()
        .await?
        .json()?;

    insta::assert_json_snapshot!(get_solution_post, {".snapshot.timestamp" => "[timestamp]"});

    Ok(())
}
