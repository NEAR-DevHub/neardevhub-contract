mod test_env;

use near_sdk::NearToken;
use {crate::test_env::*, serde_json::json};

#[tokio::test]
async fn test_community_addon() -> anyhow::Result<()> {
    // Initialize the devhub and near social contract on chain,
    // contract is devhub contract instance.
    let (contract, _, _) = init_contracts_from_res().await?;

    let deposit_amount = NearToken::from_near(4);

    // Add a community
    let _ = contract
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
        .deposit(deposit_amount)
        .transact()
        .await?;

    // Create add-on
    let _ = contract
        .call("create_addon")
        .args_json(json!({"addon": {
            "id": "CommunityAddOnId",
            "title": "GitHub AddOn",
            "description": "Current status of NEARCORE repo",
            "view_widget": "custom-viewer-widget",
            "configurator_widget": "github-configurator",
            "icon": "bi bi-github",
        }}))
        .max_gas()
        .transact()
        .await?;

    let _ = contract
        .call("set_community_addons")
        .args_json(json!({
            "handle": "gotham",
            "addons": [{
                "id": "unique",
                "addon_id": "CommunityAddOnId",
                "display_name": "GitHub",
                "enabled": true,
                "parameters": ""
            }]
        }))
        .max_gas()
        .transact()
        .await?;

    let get_community: serde_json::Value = contract
        .call("get_community")
        .args_json(json!({
            "handle" : "gotham"
        }))
        .view()
        .await?
        .json()?;

    assert_eq!(get_community["addons"][0]["display_name"].as_str(), Some("GitHub"));

    Ok(())
}

#[tokio::test]
async fn test_update_community() -> anyhow::Result<()> {
    // Initialize the devhub and near social contract on chain,
    // contract is devhub contract instance.
    let (contract, _, _) = init_contracts_from_res().await?;

    let deposit_amount = NearToken::from_near(4);

    // Add a community
    let _ = contract
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
        .deposit(deposit_amount)
        .transact()
        .await?;

    let _ = contract
        .call("update_community")
        .args_json(json!({
            "handle": "gotham",
            "community": {
                "admins": [],
                "handle": "gotham",
                "name": "Gotham2",
                "tag": "other",
                "description": "This is a test community.",
                "bio_markdown": "This is a sample text about your community.\nYou can change it on the community configuration page.",
                "logo_url": "https://ipfs.near.social/ipfs/bafkreibysr2mkwhb4j36h2t7mqwhynqdy4vzjfygfkfg65kuspd2bawauu",
                "banner_url": "https://ipfs.near.social/ipfs/bafkreic4xgorjt6ha5z4s5e3hscjqrowe5ahd7hlfc5p4hb6kdfp6prgy4",
                "addons": []
            }
        }))
        .max_gas()
        .transact()
        .await?;

    let get_community: serde_json::Value = contract
        .call("get_community")
        .args_json(json!({
            "handle" : "gotham"
        }))
        .view()
        .await?
        .json()?;

    assert_eq!(get_community["tag"].as_str(), Some("other"));
    assert_eq!(get_community["name"].as_str(), Some("Gotham2"));

    Ok(())
}

#[tokio::test]
async fn test_announcement() -> anyhow::Result<()> {
    // Initialize the devhub and near social contract on chain,
    // contract is devhub contract instance.
    let (contract, worker, _) = init_contracts_from_res().await?;

    let deposit_amount = NearToken::from_near(4);

    // Add a community
    let _ = contract
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
        .deposit(deposit_amount)
        .transact()
        .await?;

    let community_account = "gotham.community.devhub.near".parse()?;

    // assert community account exists
    let _ = worker.view_account(&community_account).await?;

    // create announcement
    let create_announcement = contract
        .call("set_community_socialdb")
        .args_json(json!({
            "handle": "gotham",
            "data": {
                "post": {
                    "main": "{\"type\":\"md\",\"text\":\"what's up\"}"
                },
                "index": {
                    "post": "{\"key\":\"main\",\"value\":{\"type\":\"md\"}}"
                }
            }
        }))
        .max_gas()
        .transact()
        .await?;

    assert!(create_announcement.is_success());

    let near_social_account = "social.near".parse()?;
    let data: serde_json::Value = worker
        .view(&near_social_account, "get")
        .args_json(json!({"keys": ["gotham.community.devhub.near/**"]}))
        .await?
        .json()?;

    assert_eq!(
        data["gotham.community.devhub.near"]["post"]["main"].as_str(),
        Some("{\"type\":\"md\",\"text\":\"what's up\"}")
    );

    // update community, intend to change name and logo
    let _ = contract
    .call("update_community")
    .args_json(json!({
        "handle": "gotham",
        "community": {
            "admins": [],
            "handle": "gotham",
            "name": "Gotham2",
            "tag": "some",
            "description": "This is a test community.",
            "bio_markdown": "This is a sample text about your community.\nYou can change it on the community configuration page.",
            "logo_url": "https://example.com/image.png",
            "banner_url": "https://ipfs.near.social/ipfs/bafkreic4xgorjt6ha5z4s5e3hscjqrowe5ahd7hlfc5p4hb6kdfp6prgy4",
            "addons": []
        }
    }))
    .max_gas()
    .transact()
    .await?;

    let near_social_account = "social.near".parse()?;
    let data: serde_json::Value = worker
        .view(&near_social_account, "get")
        .args_json(json!({"keys": ["gotham.community.devhub.near/**"]}))
        .await?
        .json()?;

    assert_eq!(data["gotham.community.devhub.near"]["profile"]["name"].as_str(), Some("Gotham2"));
    assert_eq!(
        data["gotham.community.devhub.near"]["profile"]["image"]["url"].as_str(),
        Some("https://example.com/image.png")
    );

    Ok(())
}

#[tokio::test]
async fn test_discussions() -> anyhow::Result<()> {
    // Initialize the devhub and near social contract on chain,
    // contract is devhub contract instance.
    let (contract, worker, near_social) = init_contracts_from_res().await?;

    let deposit_amount = NearToken::from_near(6);

    // Add a community
    let _ = contract
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
        .deposit(deposit_amount)
        .transact()
        .await?;

    let community_account = "gotham.community.devhub.near".parse()?;
    let discussions_account = "discussions.gotham.community.devhub.near".parse()?;

    // assert community account exists
    let _ = worker.view_account(&community_account).await?;
    // assert discussions account exists
    let _ = worker.view_account(&discussions_account).await?;

    // grant write permission to discussions account from a user
    let user = worker.dev_create_account().await?;

    let grant_write_permission_result = user
        .call(near_social.id(), "grant_write_permission")
        .args_json(json!({
            "predecessor_id": contract.id(),
            "keys": ["bob.near/post/main"],
        }))
        .max_gas()
        .transact()
        .await?;

    // create discussion as user
    let create_discussion = user
        .call(contract.id(), "create_discussion")
        .args_json(json!({
            "handle": "gotham",
            "data": {
                "post": {
                    "main": "{\"type\":\"md\",\"text\":\"what's up\"}"
                },
                "index": {
                    "post": "{\"key\":\"main\",\"value\":{\"type\":\"md\"}}"
                }
            }
        }))
        .max_gas()
        .transact()
        .await?;

    assert!(create_discussion.is_success());

    let discussion_data: serde_json::Value = worker
        .view(&near_social.id(), "get")
        .args_json(json!({"keys": ["discussions.gotham.community.devhub.near/**"]}))
        .await?
        .json()?;

    assert_eq!(
        discussion_data["discussions.gotham.community.devhub.near"]["post"]["main"].as_str(),
        Some("{\"type\":\"md\",\"text\":\"what's up\"}")
    );

    Ok(())
}
