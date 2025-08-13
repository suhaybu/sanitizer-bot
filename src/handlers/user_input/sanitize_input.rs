use tracing::debug;

use super::{external_api::QuickVidsAPI, parse_url::ParsedURL};

// Responses
const TWITTER_TEMPLATE: &str = "[@{0} via X (Twitter)](https://fxtwitter.com{1})";
const INSTAGRAM_API_TEMPLATE: &str = "[@{0} {1} via Instagram]({2})";
const INSTAGRAM_TEMPLATE: &str = "[{0} via Instagram](https://g.ddinstagram.com/{1}{2})";
const TIKTOK_TEMPLATE: &str = "[@{0} via TikTok]({1})";

pub async fn sanitize_input(user_input: &str) -> Option<String> {
    debug!("Attempting to parse URL: {}", user_input);
    let parsed_url = ParsedURL::new(user_input)?;
    debug!("URL parsed as: {:?}", parsed_url);
    let api_client = QuickVidsAPI::new();

    let format_post_type = |post_type: &str| -> &'static str {
        match post_type {
            "p" => "Post",
            "reel" => "Reel",
            _ => "",
        }
    };

    match parsed_url {
        ParsedURL::Twitter { username, data, .. } => Some(
            TWITTER_TEMPLATE
                .replace("{0}", username.as_ref())
                .replace("{1}", data.as_ref()),
        ),

        ParsedURL::Instagram {
            post_type,
            data,
            url,
        } => match api_client.get_response(&url).await {
            Some(api_response) => Some(
                INSTAGRAM_API_TEMPLATE
                    .replace("{0}", &api_response.username.unwrap_or_default())
                    .replace("{1}", format_post_type(post_type.as_ref()))
                    .replace("{2}", &api_response.url),
            ),
            None => Some(
                INSTAGRAM_TEMPLATE
                    .replace("{0}", format_post_type(post_type.as_ref()))
                    .replace("{1}", post_type.as_ref())
                    .replace("{2}", data.as_ref()),
            ),
        },

        ParsedURL::Tiktok { url } => match api_client.get_response(url.as_ref()).await {
            Some(api_response) => Some(
                TIKTOK_TEMPLATE
                    .replace("{0}", &api_response.username.unwrap_or_default())
                    .replace("{1}", &api_response.url),
            ),
            None => None,
        },
    }
}
