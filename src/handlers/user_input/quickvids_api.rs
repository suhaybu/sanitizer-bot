use reqwest::{Client, ClientBuilder};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use std::time::Duration;

const API_URL: &str = "https://api.quickvids.app/v2/quickvids/shorturl";
static API_TOKEN: LazyLock<String> =
    LazyLock::new(|| std::env::var("QUICKVIDS_TOKEN").expect("QUICKVIDS_TOKEN is not set"));

static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    ClientBuilder::new()
        .timeout(Duration::from_secs(2)) // 2 second total timeout
        .connect_timeout(Duration::from_secs(1)) // 1 second connect timeout
        .pool_max_idle_per_host(1) // Single connection for infrequent use
        .use_rustls_tls()
        .build()
        .expect("Failed to create HTTP client")
});

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

pub struct QuickVidsAPI {}

impl QuickVidsAPI {
    pub fn new() -> Self {
        Self {}
    }

    async fn make_request(&self, url: &str, detailed: bool) -> Option<APIResponse> {
        // This function makes the API request with crazy 0 variables
        CLIENT
            .post(API_URL)
            .bearer_auth(&*API_TOKEN)
            .json(&APIRequest {
                input_text: url,
                detailed,
            })
            .send() // Sends the HTTP request, gets a Result<Response, Error>
            .await
            .ok()? // Unwraps Result -> Option
            .error_for_status() // Checks if there is an error, gets a Result
            .ok()? // Unwraps Result -> Option
            .json() // Deserializes the JSON response from API
            .await
            .ok() // Converts the Result into Option
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
