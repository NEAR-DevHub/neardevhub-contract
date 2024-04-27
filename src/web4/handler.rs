use near_sdk::base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine,
};

use crate::{
    web4::types::{Web4Request, Web4Response},
    Contract,
};

pub const BASE64_ENGINE: engine::GeneralPurpose =
    engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);

pub fn web4_get(contract: &Contract, request: Web4Request) -> Web4Response {
    let path_parts: Vec<&str> = request.path.split('/').collect();

    let gateway = "https://near.social";

    let page = path_parts[1];
    let mut title: String = String::from("near/dev/hub");
    let mut description: String = String::from("The decentralized home base for NEAR builders");
    let mut image: String = String::from(
        "https://i.near.social/magic/large/https://near.social/magic/img/account/devhub.near",
    );
    let mut redirect_url: String = String::from("https://near.social/devhub.near/widget/app");

    match page {
        "community" => {
            let handle = path_parts[2];
            let community = contract.get_community(handle.to_string()).unwrap();
            redirect_url =
                format!("{}/devhub.near/widget/app?page={}&handle={}", gateway, page, handle);
            title = community.name;
            description = community.description;
            image = community.logo_url;
        }
        _ => {}
    }

    let body = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>{title}</title>
    <meta property="og:url" content="{url}" />
    <meta property="og:type" content="website" />
    <meta property="og:title" content="{title}" />
    <meta property="og:description" content="{description}" />
    <meta property="og:image" content="{image}" />

    <meta name="twitter:card" content="summary_large_image">
    <meta name="twitter:title" content="{title}">
    <meta name="twitter:description" content="{description}">
    <meta name="twitter:image" content="{image}">
</head>
<body>
    <h1>{title}</h1>
    <p>{description}</p>
    <a href="{url}">Visit our page</a>
</body>
</html>"#,
        url = redirect_url
    );

    Web4Response::Body {
        content_type: "text/html; charset=UTF-8".to_owned(),
        body: BASE64_ENGINE.encode(body),
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::web4_get;
    use crate::{
        web4::{handler::BASE64_ENGINE, types::Web4Response},
        CommunityInputs, Contract,
    };
    use near_sdk::{
        base64::Engine, test_utils::VMContextBuilder, testing_env, NearToken, VMContext,
    };

    #[test]
    pub fn test_web4() {
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
                println!("Body: {:?}", body_string);
                assert!(body_string.contains("<meta property=\"og:description\" content=\"Music stored forever in the NEAR blockchain\" />"));
                assert!(body_string
                    .contains("<meta name=\"twitter:title\" content=\"WebAssembly Music\">"));
                assert!(body_string.contains("https://near.social/devhub.near/widget/app?page=community&handle=webassemblymusic"));
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
                println!("Body: {:?}", body_string);
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
}
