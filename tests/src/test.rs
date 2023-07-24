// macro allowing us to convert human readable units to workspace units.

// macro allowing us to convert args into JSON bytes to be read by the contract.
use serde_json::json;

// Additional convenient imports that allows workspaces to function readily.

//Pointing to my devhub contract
const DEVHUB_WASM_FILEPATH: &str = "../res/devgovgigs.wasm";

//This includes launching the sandbox, loading your wasm file and deploying it to the sandbox environment.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let wasm = std::fs::read(DEVHUB_WASM_FILEPATH)?;
    let contract = worker.dev_deploy(&wasm).await?;

    let outcome = contract
    .call("new")
    .args_json(json!({}))
    .transact()  // note: we use the contract's keys here to sign the transaction
    .await?;

    // outcome contains data like logs, receipts and transaction outcomes.
    println!("Init outcome: {:#?}", outcome);

    //near call $contract add_post --accountId zxcvn.testnet --deposit 0.01 --args '{"parent_id":null,"body":{"post_type": "Idea","idea_version":"V1","name":"a'$i'","description":"aaa"},"labels":[]}'

    let self_outcome = contract
        .call("unsafe_self_upgrade")
        .args(wasm)
        //.deposit(deposit)
        .max_gas()
        //.gas(near_units::parse_gas!("300 Tgas") as u64)
        .transact()
        .await?;

    println!("Outcome: {:#?}", self_outcome);


    //let deposit = 10000000000000000000000;
    // let outcome = contract
    //     .call("add_community")
    //     .args_json(json!({
          
    //         "handle": "golden",
    //         "community": {
    //           "handle": "golden",
    //           "name": "Golden Retrievers ",
    //           "description": "A community of dogs",
    //           "tag": "dogs",
    //           "bio_markdown": "",
    //           "admins": [
    //             contract.id()
    //           ],
    //           "banner_url": "https://ipfs.near.social/ipfs/bafkreic4xgorjt6ha5z4s5e3hscjqrowe5ahd7hlfc5p4hb6kdfp6prgy4",
    //           "logo_url": "https://ipfs.near.social/ipfs/bafkreibysr2mkwhb4j36h2t7mqwhynqdy4vzjfygfkfg65kuspd2bawauu"
            
    //       }
            
    //     }))
    //     //.deposit(deposit)
    //     .transact()
    //     .await?;

    //println!("Add community outcome: {:#?}", outcome);


    // let community = contract
    //     .call("get_community")
    //     .args_json(json!({
          
    //         "handle": "golden",
    //     }))
    //     //.deposit(deposit)
    //     .transact()
    //     .await?;

    // println!("Get community outcome: {:#?}", community);

    
    Ok(())
}
