mod test_env;

use {crate::test_env::*, serde_json::json};

#[tokio::test]
async fn test_community_addon() -> anyhow::Result<()> {
    // Initialize the devhub and near social contract on chain,
    // contract is devhub contract instance.
    let contract = init_contracts().await?;

    let deposit_amount = near_units::parse_near!("0.1");

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

    // Create add-on
    contract
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

    contract
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
    let contract = init_contracts().await?;

    let deposit_amount = near_units::parse_near!("0.1");

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
    
    let update_community = contract
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
                "telegram_handle": [],
                "features": {
                    "telegram": true,
                    "github": true,
                    "board": true,
                    "wiki": true
                },
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