use near_sdk::base64::{alphabet, engine::{self, general_purpose}, Engine};

use crate::web4::types::{Web4Request, Web4Response};

const BASE64_ENGINE: engine::GeneralPurpose =
    engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);

pub fn web4_get(#[allow(unused_variables)] request: Web4Request) -> Web4Response {
    let path_parts: Vec<&str> = request.path.split('/').collect();

    let page = path_parts[1]; // Assuming path format is "/community/webassemblymusic"
    let handle = path_parts[2];


    let redirect_url = format!("https://near.social/devhub.near/widget/app?page={}&handle={}", page, handle);

    let body = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Your Website Title</title>
    <meta property="og:url" content="{url}" />
    <meta property="og:type" content="website" />
    <meta property="og:title" content="Your Title Here" />
    <meta property="og:description" content="Your description here" />
    <meta property="og:image" content="https://example.com/image.jpg" />

    <!-- Additional tags for Twitter -->
    <meta name="twitter:card" content="summary_large_image">
    <meta name="twitter:title" content="Your Title Here">
    <meta name="twitter:description" content="Your description here">
    <meta name="twitter:image" content="https://example.com/image.jpg">
</head>
<body>
    <h1>Hello, check out this link:</h1>
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
    use crate::web4::types::Web4Response;
    use super::web4_get;

    #[test]
    pub fn test_web4() {
        let response = web4_get(serde_json::from_value(serde_json::json!({
            "path": "/community/webassemblymusic"
        })).unwrap());
        match response {
            Web4Response::Body { content_type, body } => {
                println!("Content Type: {}", content_type);
                println!("Body: {}", body);
            },
            Web4Response::BodyUrl { body_url } => {
                println!("Body is located at URL: {}", body_url);
            },
            Web4Response::PreloadUrls { preload_urls } => {
                println!("Preloaded URLs: {:?}", preload_urls);
            },
        }
    }
}