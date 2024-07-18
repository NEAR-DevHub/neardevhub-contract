use near_sdk::{
    base64::{
        alphabet,
        engine::{self, general_purpose},
        Engine,
    },
    env,
};
use serde_json::json;

use crate::{
    web4::types::{Web4Request, Web4Response},
    Contract, Proposal,
};

pub const BASE64_ENGINE: engine::GeneralPurpose =
    engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::PAD);

pub fn web4_get(contract: &Contract, request: Web4Request) -> Web4Response {
    let current_account_id = env::current_account_id().to_string();
    let path_parts: Vec<&str> = request.path.split('/').collect();

    // A valid path provided by a legit web4 gateway always has '/', so there
    // are always [0] and [1] elements, and [0] is always empty.
    let page = path_parts[1];

    let metadata_preload_url = format!(
        "/web4/contract/social.near/get?keys.json=%5B%22{}/widget/app/metadata/**%22%5D",
        &current_account_id
    );

    let mut app_name = String::from("near/dev/hub");
    let mut title = String::new();
    let mut description = String::from("The decentralized home base for NEAR builders");

    let Some(preloads) = request.preloads else {
        return Web4Response::PreloadUrls { preload_urls: [metadata_preload_url.clone()].to_vec() };
    };

    if let Some(Web4Response::Body { content_type: _, body }) = preloads.get(&metadata_preload_url) {
        if let Ok(body_value) = serde_json::from_slice::<serde_json::Value>(
            &BASE64_ENGINE.decode(body).unwrap()
        ) {
            if let Some(app_name_str) = body_value[&current_account_id]
                ["widget"]["app"]["metadata"]["name"]
                .as_str()
            {
                app_name = app_name_str.to_string();
            }

            if let Some(description_str) = body_value
                [&current_account_id]["widget"]["app"]["metadata"]
                ["description"]
                .as_str()
            {
                description = description_str.to_string();
            }
        }
    }

    let mut image = format!(
        "https://i.near.social/magic/large/https://near.social/magic/img/account/{}",
        &current_account_id
    );
    let redirect_path;
    let initial_props_json;

    match (page, path_parts.get(2)) {
        ("community", Some(handle)) => {
            if let Some(community) = contract.get_community(handle.to_string()) {
                title = format!(" - Community - {}", community.name);
                description = community.description;
                image = community.logo_url;
            } else {
                title = format!(" - Community - {}", handle);
            }
            redirect_path =
                format!("{}/widget/app?page={}&handle={}", &current_account_id, page, handle);
            initial_props_json = json!({"page": page, "handle": handle});
        }
        ("proposal", Some(id)) => {
            if let Ok(id) = id.parse::<u32>() {
                if let Some(versioned_proposal) = contract.proposals.get(id.into()) {
                    let proposal_body =
                        Proposal::from(versioned_proposal).snapshot.body.latest_version();
                    title = format!(" - Proposal #{} - {}", id, proposal_body.name);
                    description = proposal_body.summary;
                } else {
                    title = format!(" - Proposal #{}", id);
                }
            } else {
                title = " - Proposals".to_string();
            }
            redirect_path =
                format!("{}/widget/app?page={}&id={}", &current_account_id, page, id);
            initial_props_json = json!({"page": page, "id": id});
        }
        _ => {
            redirect_path = format!("{}/widget/app", &current_account_id);
            initial_props_json = json!({"page": page});
        }
    }

    let app_name = html_escape::encode_text(&app_name).to_string();
    let title = html_escape::encode_text(&title).to_string();
    let description = html_escape::encode_text(&description).to_string();

    let body = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>{title}</title>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width,initial-scale=1">
    <meta property="og:url" content="{url}" />
    <meta property="og:type" content="website" />
    <meta property="og:title" content="{app_name}{title}" />
    <meta property="og:description" content="{description}" />
    <meta property="og:image" content="{image}" />

    <meta name="twitter:card" content="summary_large_image">
    <meta name="twitter:title" content="{app_name}{title}">
    <meta name="twitter:description" content="{description}">
    <meta name="twitter:image" content="{image}">
    <script src="https://ipfs.web4.near.page/ipfs/bafybeic6aeztkdlthx5uwehltxmn5i6owm47b7b2jxbbpwmydv2mwxdfca/main.794b6347ae264789bc61.bundle.js"></script>
    <script src="https://ipfs.web4.near.page/ipfs/bafybeic6aeztkdlthx5uwehltxmn5i6owm47b7b2jxbbpwmydv2mwxdfca/runtime.25b143da327a5371509f.bundle.js"></script>
    <style>
        @media screen and (max-width: 600px) {{
            .gatewaylinks .nav-link {{
                padding-top: 0px!important;
                padding-bottom: 0px!important;
                margin: 0px;
            }}
            .gatewaylinks img {{
                height: 30px;
            }}
        }}
    </style>
</head>
<body>
<nav class="navbar navbar-expand-sm navbar-light bg-dark" style="display: flex; flex-wrap: nowrap; padding-left: 5px; padding-right: 5px; height: 73px; border-bottom: rgb(0, 236, 151) solid 5px;">
    <a class="navbar-brand" href="/"><img src="https://i.near.social/magic/large/https://near.social/magic/img/account/{current_account_id}" style="height: 68px" /></a>
    <p class="nav-text" style="flex-grow: 1"></p>
    <p class="nav-text text-light" style="margin-top: 1rem; margin-right: 1rem">Choose your gateway</p>
    <div class="navbar-nav gatewaylinks">
        <a href="https://near.org/{redirect_path}" class="nav-link">
            <img src="https://ipfs.web4.near.page/ipfs/bafybeia2ptgyoz7b6oxu3k57jmiras2pgigmw7a3cp6osjog67rndmf36y/nearorg.svg" />
        </a>
        <a href="https://near.social/{redirect_path}" class="nav-link">
            <img src="https://ipfs.web4.near.page/ipfs/bafybeia2ptgyoz7b6oxu3k57jmiras2pgigmw7a3cp6osjog67rndmf36y/nearsocial.svg" />
        </a>
    </div>
</nav>
    <near-social-viewer src="{current_account_id}/widget/app" initialProps='{initial_props_json}'></near-social-viewer>
</body>
</html>"#,
        url = redirect_path
    );

    Web4Response::Body {
        content_type: "text/html; charset=UTF-8".to_owned(),
        body: BASE64_ENGINE.encode(body),
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use std::collections::HashSet;

    use super::web4_get;
    use crate::{
        web4::{handler::BASE64_ENGINE, types::Web4Response},
        CommunityInputs, Contract, Proposal, ProposalBodyV0, ProposalSnapshot,
        VersionedProposalBody,
    };
    use near_sdk::{
        base64::Engine, serde_json::json, test_utils::VMContextBuilder, testing_env, NearToken,
    };

    const PRELOAD_URL: &str = "/web4/contract/social.near/get?keys.json=%5B%22not-only-devhub.near/widget/app/metadata/**%22%5D";

    fn create_preload_result(title: String, description: String) -> serde_json::Value {
        let body_string = serde_json::json!({"not-only-devhub.near":{"widget":{"app":{"metadata":{
            "description":description,
            "image":{"ipfs_cid":"bafkreido4srg4aj7l7yg2tz22nbu3ytdidjczdvottfr5ek6gqorwg6v74"},
            "name":title,
            "tags": {"devhub":"","communities":"","developer-governance":"","app":""}}}}}})
        .to_string();

        let body_base64 = BASE64_ENGINE.encode(body_string);
        return serde_json::json!({
                String::from(PRELOAD_URL): {
                    "contentType": "application/json",
                    "body": body_base64
                }
        });
    }

    fn view_test_env() {
        let contract: String = "not-only-devhub.near".to_string();
        let context =
            VMContextBuilder::new().current_account_id(contract.try_into().unwrap()).build();

        testing_env!(context);
    }

    #[test]
    pub fn test_preload_url_response() {
        view_test_env();
        let contract = Contract::new();

        let response_before_preload = web4_get(
            &contract,
            serde_json::from_value(serde_json::json!({
                "path": "/"
            }))
            .unwrap(),
        );
        match response_before_preload {
            Web4Response::PreloadUrls { preload_urls } => {
                assert_eq!(PRELOAD_URL, preload_urls.get(0).unwrap())
            }
            _ => {
                panic!("Should return Web4Response::PreloadUrls");
            }
        }
    }

    #[test]
    pub fn test_response_with_preload_content() {
        view_test_env();
        let contract = Contract::new();

        let response = web4_get(
            &contract,
            serde_json::from_value(serde_json::json!({
                "path": "/",
                "preloads": create_preload_result(String::from("NotOnlyDevHub"),String::from("A description of any devhub portal instance, not just devhub itself")),
            }))
            .unwrap(),
        );
        match response {
            Web4Response::Body { content_type, body } => {
                assert_eq!("text/html; charset=UTF-8", content_type);

                let body_string = String::from_utf8(BASE64_ENGINE.decode(body).unwrap()).unwrap();

                assert!(body_string.contains(
                    "<meta property=\"og:description\" content=\"A description of any devhub portal instance, not just devhub itself\" />"
                ));
                assert!(body_string
                    .contains("<meta property=\"og:title\" content=\"NotOnlyDevHub\" />"));
            }
            _ => {
                panic!("Should return Web4Response::Body");
            }
        }
    }

    #[test]
    pub fn test_response_with_empty_preload_content() {
        view_test_env();
        let contract = Contract::new();

        let response = web4_get(
            &contract,
            serde_json::from_value(serde_json::json!({
                "path": "/",
                "preloads": {
                        String::from(PRELOAD_URL): {
                            "contentType": "application/json",
                            "body": ""
                        }
                },
            }))
            .unwrap(),
        );
        match response {
            Web4Response::Body { content_type, body } => {
                assert_eq!("text/html; charset=UTF-8", content_type);

                let body_string = String::from_utf8(BASE64_ENGINE.decode(body).unwrap()).unwrap();

                assert!(body_string.contains(
                    "<meta property=\"og:description\" content=\"The decentralized home base for NEAR builders\" />"
                ));
                assert!(body_string
                    .contains("<meta property=\"og:title\" content=\"near/dev/hub\" />"));
            }
            _ => {
                panic!("Should return Web4Response::Body");
            }
        }
    }

    #[test]
    pub fn test_logo() {
        view_test_env();
        let contract = Contract::new();
        let response = web4_get(
            &contract,
            serde_json::from_value(serde_json::json!({
                "path": "/proposal/1",
                "preloads": create_preload_result(String::from("title"), String::from("description")),
            }))
            .unwrap(),
        );
        match response {
            Web4Response::Body { content_type, body } => {
                assert_eq!("text/html; charset=UTF-8", content_type);

                let body_string = String::from_utf8(BASE64_ENGINE.decode(body).unwrap()).unwrap();
                assert!(body_string.contains("<a class=\"navbar-brand\" href=\"/\"><img src=\"https://i.near.social/magic/large/https://near.social/magic/img/account/not-only-devhub.near\" style=\"height: 68px\" /></a>"));
                assert!(body_string.contains("<meta property=\"og:image\" content=\"https://i.near.social/magic/large/https://near.social/magic/img/account/not-only-devhub.near\" />"));
                assert!(body_string.contains("<meta name=\"twitter:image\" content=\"https://i.near.social/magic/large/https://near.social/magic/img/account/not-only-devhub.near\">"));
                let expected_initial_props_string =
                    json!({"page": "proposal", "id": "1"}).to_string();
                assert!(body_string.contains(&expected_initial_props_string));
            }
            _ => {
                panic!("Should return Web4Response::Body");
            }
        }
    }

    #[test]
    pub fn test_community_path() {
        let signer = "bob.near".to_string();
        let contract: String = "not-only-devhub.near".to_string();

        let context = VMContextBuilder::new()
            .signer_account_id(signer.clone().try_into().unwrap())
            .current_account_id(contract.try_into().unwrap())
            .attached_deposit(NearToken::from_near(4))
            .build();

        testing_env!(context);
        let mut contract = Contract::new();

        contract.create_community(CommunityInputs {
            handle: String::from("webassemblymusic"),
            name: String::from("WebAssembly Music"), 
            description: String::from("Music stored forever in the NEAR blockchain"),tag: String::from("wasm"),
            logo_url: String::from("https://ipfs.near.social/ipfs/bafybeiesrsf4fpdmlfgcnxpuxiqlgw2lk3bietdt25mvumrjk5yhf2c54e"),
            banner_url: String::from("https://ipfs.near.social/ipfs/bafybeihsid3qgrb2dd4adsd4kuwe3pondtjr3u27ru6e2mbvabvm4rocru"),
            bio_markdown: Some(String::from("Music stored forever in the NEAR blockchain"))
        });

        let response = web4_get(
            &contract,
            serde_json::from_value(serde_json::json!({
                "path": "/community/webassemblymusic",
                "preloads": create_preload_result(String::from("title"), String::from("description")),
            }))
            .unwrap(),
        );
        match response {
            Web4Response::Body { content_type, body } => {
                assert_eq!("text/html; charset=UTF-8", content_type);

                let body_string = String::from_utf8(BASE64_ENGINE.decode(body).unwrap()).unwrap();

                assert!(body_string.contains("<meta property=\"og:description\" content=\"Music stored forever in the NEAR blockchain\" />"));
                assert!(body_string
                    .contains("<meta name=\"twitter:title\" content=\"title - Community - WebAssembly Music\">"));
                assert!(body_string.contains("https://near.social/not-only-devhub.near/widget/app?page=community&handle=webassemblymusic"));
                let expected_initial_props_string =
                    json!({"page": "community", "handle": "webassemblymusic"}).to_string();
                assert!(body_string.contains(&expected_initial_props_string));
            }
            _ => {
                panic!("Should return Web4Response::Body");
            }
        }
    }

    #[test]
    pub fn test_web4_unknown_path() {
        view_test_env();
        let contract = Contract::new();
        for unknown_path in &["/", "/unknown", "/unknown/path"] {
            let response = web4_get(
                &contract,
                serde_json::from_value(serde_json::json!({
                    "path": unknown_path,
                    "preloads": create_preload_result(String::from("near/dev/hub"), String::from("The decentralized home base for NEAR builders")),
                }))
                .unwrap(),
            );
            match response {
                Web4Response::Body { content_type, body } => {
                    assert_eq!("text/html; charset=UTF-8", content_type);

                    let body_string =
                        String::from_utf8(BASE64_ENGINE.decode(body).unwrap()).unwrap();

                    assert!(body_string.contains("<meta name=\"twitter:description\" content=\"The decentralized home base for NEAR builders\">"));
                    assert!(body_string
                        .contains("<meta name=\"twitter:title\" content=\"near/dev/hub\">"));
                    assert!(
                        body_string.contains("https://near.social/not-only-devhub.near/widget/app")
                    );
                }
                _ => {
                    panic!("Should return Web4Response::Body for '{}' path", unknown_path);
                }
            }
        }
    }

    #[test]
    pub fn test_web4_unknown_community() {
        view_test_env();
        let contract = Contract::new();
        let response = web4_get(
            &contract,
            serde_json::from_value(serde_json::json!({
                "path": "/community/blablablablabla",
                "preloads": create_preload_result(String::from("near/dev/hub"), String::from("The decentralized home base for NEAR builders")),
            }))
            .unwrap(),
        );
        match response {
            Web4Response::Body { content_type, body } => {
                assert_eq!("text/html; charset=UTF-8", content_type);

                let body_string = String::from_utf8(BASE64_ENGINE.decode(body).unwrap()).unwrap();

                assert!(body_string.contains("<meta name=\"twitter:description\" content=\"The decentralized home base for NEAR builders\">"));
                assert!(
                    body_string.contains("<meta name=\"twitter:title\" content=\"near/dev/hub - Community - blablablablabla\">")
                );
                assert!(body_string.contains("https://near.social/not-only-devhub.near/widget/app"));
                assert!(body_string.contains("https://near.org/not-only-devhub.near/widget/app"));
                let expected_initial_props_string =
                    json!({"page": "community", "handle": "blablablablabla"}).to_string();
                assert!(body_string.contains(&expected_initial_props_string));
            }
            _ => {
                panic!("Should return Web4Response::Body");
            }
        }
    }

    #[test]
    pub fn test_web4_community_missing_handle() {
        view_test_env();
        let contract = Contract::new();
        let response = web4_get(
            &contract,
            serde_json::from_value(serde_json::json!({
                "path": "/community",
                "preloads": create_preload_result(String::from("near/dev/hub"), String::from("The decentralized home base for NEAR builders")),
            }))
            .unwrap(),
        );
        match response {
            Web4Response::Body { content_type, body } => {
                assert_eq!("text/html; charset=UTF-8", content_type);

                let body_string = String::from_utf8(BASE64_ENGINE.decode(body).unwrap()).unwrap();

                assert!(body_string.contains("<meta name=\"twitter:description\" content=\"The decentralized home base for NEAR builders\">"));
                assert!(
                    body_string.contains("<meta name=\"twitter:title\" content=\"near/dev/hub\">")
                );
                assert!(body_string.contains("https://near.social/not-only-devhub.near/widget/app"));
                assert!(body_string.contains("https://near.org/not-only-devhub.near/widget/app"));
                let expected_initial_props_string = json!({"page": "community"}).to_string();
                assert!(body_string.contains(&expected_initial_props_string));
            }
            _ => {
                panic!("Should return Web4Response::Body");
            }
        }
    }

    #[test]
    pub fn test_proposal_path() {
        let signer = "bob.near".to_string();
        let contract = "not-only-devhub.near".to_string();
        let context = VMContextBuilder::new()
            .signer_account_id(signer.clone().try_into().unwrap())
            .current_account_id(contract.try_into().unwrap())
            .build();

        testing_env!(context);
        let mut contract = Contract::new();

        let proposal_body: ProposalBodyV0 = near_sdk::serde_json::from_value(json!({
            "proposal_body_version": "V0",
            "name": "The best proposal ever",
            "description": "You should just understand why this is the best proposal",
            "category": "Marketing",
            "summary": "It is obvious why this proposal is so great",
            "linked_proposals": [1, 3],
            "requested_sponsorship_usd_amount": "1000000000",
            "requested_sponsorship_paid_in_currency": "USDT",
            "receiver_account": "polyprogrammist.near",
            "supervisor": "frol.near",
            "requested_sponsor": "neardevdao.near",
            "payouts": [],
            "timeline": {"status": "DRAFT"}
        }))
        .unwrap();
        let proposal = Proposal {
            id: 0,
            author_id: "bob.near".parse().unwrap(),
            social_db_post_block_height: 0u64,
            snapshot: ProposalSnapshot {
                editor_id: "bob.near".parse().unwrap(),
                timestamp: 0,
                labels: HashSet::new(),
                body: VersionedProposalBody::V0(proposal_body),
            },
            snapshot_history: vec![],
        };

        contract.proposals.push(&proposal.clone().into());

        let response = web4_get(
            &contract,
            serde_json::from_value(serde_json::json!({
                "path": "/proposal/0",
                "preloads": create_preload_result(String::from("near/dev/hub"), String::from("The decentralized home base for NEAR builders")),
            }))
            .unwrap(),
        );
        match response {
            Web4Response::Body { content_type, body } => {
                assert_eq!("text/html; charset=UTF-8", content_type);

                let body_string = String::from_utf8(BASE64_ENGINE.decode(body).unwrap()).unwrap();

                assert!(body_string.contains("<meta property=\"og:description\" content=\"It is obvious why this proposal is so great\" />"));
                assert!(body_string
                    .contains("<meta name=\"twitter:title\" content=\"near/dev/hub - Proposal #0 - The best proposal ever\">"));
                assert!(body_string.contains(
                    "https://near.social/not-only-devhub.near/widget/app?page=proposal&id=0"
                ));
                assert!(body_string.contains(
                    "https://near.org/not-only-devhub.near/widget/app?page=proposal&id=0"
                ));
                let expected_initial_props_string =
                    json!({"page": "proposal", "id": "0"}).to_string();
                assert!(body_string.contains(&expected_initial_props_string));
            }
            _ => {
                panic!("Should return Web4Response::Body");
            }
        }
    }

    #[test]
    pub fn test_proposal_with_html_tag_in_summary() {
        let signer = "bob.near".to_string();
        let contract = "not-only-devhub.near".to_string();
        let context = VMContextBuilder::new()
            .signer_account_id(signer.clone().try_into().unwrap())
            .current_account_id(contract.try_into().unwrap())
            .build();

        testing_env!(context);
        let mut contract = Contract::new();

        let proposal_body: ProposalBodyV0 = near_sdk::serde_json::from_value(json!({
            "proposal_body_version": "V0",
            "name": "The best proposal ever",
            "description": "You should just understand why this is the best proposal",
            "category": "Marketing",
            "summary": "It is obvious why this <script>alert('hello');</script> proposal is so great",
            "linked_proposals": [1, 3],
            "requested_sponsorship_usd_amount": "1000000000",
            "requested_sponsorship_paid_in_currency": "USDT",
            "receiver_account": "polyprogrammist.near",
            "supervisor": "frol.near",
            "requested_sponsor": "neardevdao.near",
            "payouts": [],
            "timeline": {"status": "DRAFT"}
        }))
        .unwrap();
        let proposal = Proposal {
            id: 0,
            author_id: "bob.near".parse().unwrap(),
            social_db_post_block_height: 0u64,
            snapshot: ProposalSnapshot {
                editor_id: "bob.near".parse().unwrap(),
                timestamp: 0,
                labels: HashSet::new(),
                body: VersionedProposalBody::V0(proposal_body),
            },
            snapshot_history: vec![],
        };

        contract.proposals.push(&proposal.clone().into());

        let response = web4_get(
            &contract,
            serde_json::from_value(serde_json::json!({
                "path": "/proposal/0",
                "preloads": create_preload_result(String::from("near/dev/hub"), String::from("The decentralized home base for NEAR builders")),
            }))
            .unwrap(),
        );
        match response {
            Web4Response::Body { content_type, body } => {
                assert_eq!("text/html; charset=UTF-8", content_type);

                let body_string = String::from_utf8(BASE64_ENGINE.decode(body).unwrap()).unwrap();

                assert!(body_string.contains("<meta property=\"og:description\" content=\"It is obvious why this &lt;script&gt;alert('hello');&lt;/script&gt; proposal is so great\" />"));
                assert!(body_string
                    .contains("<meta name=\"twitter:title\" content=\"near/dev/hub - Proposal #0 - The best proposal ever\">"));
                assert!(body_string.contains(
                    "https://near.social/not-only-devhub.near/widget/app?page=proposal&id=0"
                ));
                assert!(body_string.contains(
                    "https://near.org/not-only-devhub.near/widget/app?page=proposal&id=0"
                ));
                let expected_initial_props_string =
                    json!({"page": "proposal", "id": "0"}).to_string();
                assert!(body_string.contains(&expected_initial_props_string));
            }
            _ => {
                panic!("Should return Web4Response::Body");
            }
        }
    }

    #[test]
    pub fn test_proposal_path_unknown() {
        view_test_env();
        let contract = Contract::new();
        let response = web4_get(
            &contract,
            serde_json::from_value(serde_json::json!({
                "path": "/proposal/1",
                "preloads": create_preload_result(String::from("near/dev/hub"), String::from("The decentralized home base for NEAR builders")),
            }))
            .unwrap(),
        );
        match response {
            Web4Response::Body { content_type, body } => {
                assert_eq!("text/html; charset=UTF-8", content_type);

                let body_string = String::from_utf8(BASE64_ENGINE.decode(body).unwrap()).unwrap();

                assert!(body_string.contains("<meta name=\"twitter:description\" content=\"The decentralized home base for NEAR builders\">"));
                assert!(
                    body_string.contains("<meta name=\"twitter:title\" content=\"near/dev/hub - Proposal #1\">")
                );
                assert!(body_string.contains("https://near.social/not-only-devhub.near/widget/app"));
                let expected_initial_props_string =
                    json!({"page": "proposal", "id": "1"}).to_string();
                assert!(body_string.contains(&expected_initial_props_string));
            }
            _ => {
                panic!("Should return Web4Response::Body");
            }
        }
    }

    #[test]
    pub fn test_proposal_path_incomplete() {
        view_test_env();
        let contract = Contract::new();
        let response = web4_get(
            &contract,
            serde_json::from_value(serde_json::json!({
                "path": "/proposal",
                "preloads": create_preload_result(String::from("near/dev/hub"), String::from("The decentralized home base for NEAR builders")),
            }))
            .unwrap(),
        );
        match response {
            Web4Response::Body { content_type, body } => {
                assert_eq!("text/html; charset=UTF-8", content_type);

                let body_string = String::from_utf8(BASE64_ENGINE.decode(body).unwrap()).unwrap();

                assert!(body_string.contains("<meta name=\"twitter:description\" content=\"The decentralized home base for NEAR builders\">"));
                assert!(
                    body_string.contains("<meta name=\"twitter:title\" content=\"near/dev/hub\">")
                );
                assert!(body_string.contains("https://near.social/not-only-devhub.near/widget/app"));
                let expected_initial_props_string = json!({"page": "proposal"}).to_string();
                assert!(body_string.contains(&expected_initial_props_string));
            }
            _ => {
                panic!("Should return Web4Response::Body");
            }
        }
    }
}
