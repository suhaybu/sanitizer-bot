use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

const API_URL: &str = "https://api.quickvids.app/v2/quickvids/shorturl";
static API_TOKEN: LazyLock<String> =
    LazyLock::new(|| std::env::var("QUICKVIDS_TOKEN").expect("QUICKVIDS_TOKEN is not set"));

#[derive(Serialize)]
struct APIRequest<'a> {
    input_text: &'a str,
    detailed: bool,
}

#[derive(Deserialize)]
struct Author {
    username: String,
}

#[derive(Deserialize)]
struct VideoDetails {
    author: Author,
}

#[derive(Deserialize)]
struct APIResponse {
    quickvids_url: String,
    details: Option<VideoDetails>,
}

#[derive(Debug)]
pub struct FormattedResponse {
    pub username: Option<String>,
    pub url: String,
}

pub struct QuickVidsAPI {
    client: Client,
}

impl QuickVidsAPI {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    async fn make_request(&self, url: &str, detailed: bool) -> Option<APIResponse> {
        let request = APIRequest {
            input_text: url,
            detailed,
        };

        let response = self
            .client
            .post(API_URL)
            .bearer_auth(&*API_TOKEN)
            .json(&request)
            .send()
            .await
            .ok()?;

        if response.status().is_success() {
            response.json().await.ok()
        } else {
            None
        }
    }

    pub async fn get_response(&self, url: &str) -> Option<FormattedResponse> {
        // Try detailed request first
        if let Some(api_response) = self.make_request(url, true).await {
            return Some(FormattedResponse {
                username: api_response.details.map(|details| details.author.username),
                url: api_response.quickvids_url,
            });
        }

        // Fallback to simple request
        self.make_request(url, false)
            .await
            .map(|api_response| FormattedResponse {
                username: None,
                url: api_response.quickvids_url,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_platform_response() {
        let client = QuickVidsAPI::new();
        let tiktok_url = "https://vt.tiktok.com/ZSYXeWygm/";
        if let Some(response) = client.get_response(tiktok_url).await {
            assert!(response.url.contains("TikTok"));
        }
    }
}
