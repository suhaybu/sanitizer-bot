// This file contains the logic for making the necessary external API calls
use reqwest::{Client, ClientBuilder};
use serde::{Deserialize, Serialize};
use std::{sync::LazyLock, time::Duration};
use tracing::{debug, error};

const API_URL: &str = "https://api.quickvids.app/v2/quickvids/shorturl";
static API_TOKEN: LazyLock<String> =
    LazyLock::new(|| std::env::var("QUICKVIDS_TOKEN").expect("QUICKVIDS_TOKEN is not set"));

static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    ClientBuilder::new()
        .timeout(Duration::from_secs(3)) // 2 second total timeout
        .connect_timeout(Duration::from_secs(2)) // 2 second connect timeout
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

#[derive(Deserialize, Debug)]
struct Author {
    username: String,
}

#[derive(Deserialize, Debug)]
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

    // async fn make_request(&self, url: &str, detailed: bool) -> Option<APIResponse> {
    //     // This function makes the API request with crazy 0 variables
    //     CLIENT
    //         .post(API_URL)
    //         .bearer_auth(&*API_TOKEN)
    //         .json(&APIRequest {
    //             input_text: url,
    //             detailed,
    //         })
    //         .send() // Sends the HTTP request, gets a Result<Response, Error>
    //         .await
    //         .ok()? // Unwraps Result -> Option
    //         .error_for_status() // Checks if there is an error, gets a Result
    //         .ok()? // Unwraps Result -> Option
    //         .json() // Deserializes the JSON response from API
    //         .await
    //         .ok() // Converts the Result into Option
    // }

    async fn make_request(&self, url: &str, detailed: bool) -> Option<APIResponse> {
        debug!(
            "Making API request to {} with detailed={}",
            API_URL, detailed
        );
        debug!("Request payload: input_text={}, detailed={}", url, detailed);

        // Send the request
        let response = match CLIENT
            .post(API_URL)
            .bearer_auth(&*API_TOKEN)
            .json(&APIRequest {
                input_text: url,
                detailed,
            })
            .send()
            .await
        {
            Ok(resp) => {
                debug!("HTTP request successful, status: {}", resp.status());
                resp
            }
            Err(e) => {
                error!("Failed to send HTTP request: {}", e);
                return None;
            }
        };

        // Check for HTTP errors
        let response = match response.error_for_status() {
            Ok(resp) => {
                debug!("HTTP status check passed");
                resp
            }
            Err(e) => {
                error!("HTTP error status: {}", e);
                return None;
            }
        };

        // Parse JSON response
        match response.json::<APIResponse>().await {
            Ok(api_response) => {
                debug!("Successfully parsed JSON response");
                debug!("Response URL: {}", api_response.quickvids_url);
                debug!("Response details: {:?}", api_response.details);
                Some(api_response)
            }
            Err(e) => {
                error!("Failed to parse JSON response: {}", e);
                None
            }
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
