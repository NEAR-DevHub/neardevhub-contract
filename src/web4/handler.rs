use near_sdk::base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine,
};
use serde_json::json;

use crate::{
    web4::types::{Web4Request, Web4Response},
    Contract, Proposal,
};

pub const BASE64_ENGINE: engine::GeneralPurpose =
    engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);

pub fn web4_get(contract: &Contract, request: Web4Request) -> Web4Response {
    let path_parts: Vec<&str> = request.path.split('/').collect();

    let page = path_parts[1];
    let mut title: String = String::from("near/dev/hub");
    let mut description: String = String::from("The decentralized home base for NEAR builders");
    let mut image: String = String::from(
        "https://i.near.social/magic/large/https://near.social/magic/img/account/devhub.near",
    );
    let mut redirect_path: String = String::from("devhub.near/widget/app");

    let mut initial_props_json = json!({"page": page}).to_string();

    if path_parts.len() > 1 {
        match page {
            "community" => {
                let handle = path_parts[2];
                let community_option = contract.get_community(handle.to_string());
                if community_option.is_some() {
                    let community = community_option.unwrap();
                    title = html_escape::encode_text(community.name.as_str()).to_string();
                    description =
                        html_escape::encode_text(community.description.as_str()).to_string();
                    image = community.logo_url;
                }
                redirect_path = format!("devhub.near/widget/app?page={}&handle={}", page, handle);
                initial_props_json = json!({"page": page, "handle": handle}).to_string();
            }
            "proposal" => {
                let id_string = path_parts[2];
                if let Ok(id) = id_string.parse::<u32>() {
                    if let Some(proposal) = contract.proposals.get(id.into()) {
                        let proposal_body = proposal.snapshot.body.latest_version();
                        title = html_escape::encode_text(proposal_body.name.as_str()).to_string();
                        description =
                            html_escape::encode_text(proposal_body.summary.as_str()).to_string();
                    }
                }
                redirect_path = format!("devhub.near/widget/app?page={}&id={}", page, id_string);
                initial_props_json = json!({"page": page, "id": id_string}).to_string();
            }
            _ => {}
        }
    }

    let body = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>{title}</title>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width,initial-scale=1">
    <meta property="og:url" content="{url}" />
    <meta property="og:type" content="website" />
    <meta property="og:title" content="{title}" />
    <meta property="og:description" content="{description}" />
    <meta property="og:image" content="{image}" />

    <meta name="twitter:card" content="summary_large_image">
    <meta name="twitter:title" content="{title}">
    <meta name="twitter:description" content="{description}">
    <meta name="twitter:image" content="{image}">
    <script src="https://ipfs.web4.near.page/ipfs/bafybeic6aeztkdlthx5uwehltxmn5i6owm47b7b2jxbbpwmydv2mwxdfca/main.794b6347ae264789bc61.bundle.js"></script>
    <script src="https://ipfs.web4.near.page/ipfs/bafybeic6aeztkdlthx5uwehltxmn5i6owm47b7b2jxbbpwmydv2mwxdfca/runtime.25b143da327a5371509f.bundle.js"></script>
</head>
<body>
    <div class="container">
        <nav class="navbar navbar-expand-lg navbar-light bg-light">
            <div class="navbar-brand"><img src="https://i.near.social/magic/large/https://near.social/magic/img/account/devhub.near" style="width: 64px" /></div>
            <ul class="navbar-nav mr-auto">
                <li class="nav-item"><a href="https://near.org/{redirect_path}" class="nav-link">near.org</a></li>
                <li class="nav-item"><a href="https://near.social/{redirect_path}" class="nav-link">near.social</a></li>
            </ul>
        </nav>
    </div>
    <near-social-viewer src="devhub.near/widget/app" initialProps='{initial_props_json}'></near-social-viewer>
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

    #[test]
    pub fn test_community_path() {
        let signer = "bob.near".to_string();
        let context = VMContextBuilder::new()
            .signer_account_id(signer.clone().try_into().unwrap())
            .current_account_id(signer.try_into().unwrap())
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
                "path": "/community/webassemblymusic"
            }))
            .unwrap(),
        );
        match response {
            Web4Response::Body { content_type, body } => {
                assert_eq!("text/html; charset=UTF-8", content_type);

                let body_string = String::from_utf8(BASE64_ENGINE.decode(body).unwrap()).unwrap();

                assert!(body_string.contains("<meta property=\"og:description\" content=\"Music stored forever in the NEAR blockchain\" />"));
                assert!(body_string
                    .contains("<meta name=\"twitter:title\" content=\"WebAssembly Music\">"));
                assert!(body_string.contains("https://near.social/devhub.near/widget/app?page=community&handle=webassemblymusic"));
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
        let contract = Contract::new();
        let response = web4_get(
            &contract,
            serde_json::from_value(serde_json::json!({
                "path": "/unknown/path"
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
                assert!(body_string.contains("https://near.social/devhub.near/widget/app"));
            }
            _ => {
                panic!("Should return Web4Response::Body");
            }
        }
    }

    #[test]
    pub fn test_web4_unknown_community() {
        let contract = Contract::new();
        let response = web4_get(
            &contract,
            serde_json::from_value(serde_json::json!({
                "path": "/community/blablablablabla"
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
                assert!(body_string.contains("https://near.social/devhub.near/widget/app"));
                assert!(body_string.contains("https://near.org/devhub.near/widget/app"));
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
    pub fn test_proposal_path() {
        let signer = "bob.near".to_string();
        let context = VMContextBuilder::new()
            .signer_account_id(signer.clone().try_into().unwrap())
            .current_account_id(signer.try_into().unwrap())
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
                "path": "/proposal/0"
            }))
            .unwrap(),
        );
        match response {
            Web4Response::Body { content_type, body } => {
                assert_eq!("text/html; charset=UTF-8", content_type);

                let body_string = String::from_utf8(BASE64_ENGINE.decode(body).unwrap()).unwrap();

                assert!(body_string.contains("<meta property=\"og:description\" content=\"It is obvious why this proposal is so great\" />"));
                assert!(body_string
                    .contains("<meta name=\"twitter:title\" content=\"The best proposal ever\">"));
                assert!(body_string
                    .contains("https://near.social/devhub.near/widget/app?page=proposal&id=0"));
                assert!(body_string
                    .contains("https://near.org/devhub.near/widget/app?page=proposal&id=0"));
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
        let contract = Contract::new();
        let response = web4_get(
            &contract,
            serde_json::from_value(serde_json::json!({
                "path": "/proposal/1"
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
                assert!(body_string.contains("https://near.social/devhub.near/widget/app"));
                let expected_initial_props_string =
                    json!({"page": "proposal", "id": "1"}).to_string();
                assert!(body_string.contains(&expected_initial_props_string));
            }
            _ => {
                panic!("Should return Web4Response::Body");
            }
        }
    }
}
