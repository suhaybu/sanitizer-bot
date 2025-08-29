use tracing::debug;

use super::parse_url::ParsedURL;

// Responses
const TWITTER_TEMPLATE: &str = "[@{0} via X (Twitter)](https://fxtwitter.com{1})";
const INSTAGRAM_TEMPLATE: &str = "[{0} via Instagram](https://kkinstagram.com/{1}{2})";
const TIKTOK_TEMPLATE: &str = "[Post via TikTok]({0})";

pub async fn sanitize_input(user_input: &str) -> Option<String> {
    debug!("Attempting to parse URL: {}", user_input);
    let parsed_url = ParsedURL::new(user_input)?;
    debug!("URL parsed as: {:?}", parsed_url);

    let format_post_type = |post_type: &str| -> &'static str {
        match post_type {
            "p" => "Post",
            "reel" | "reels" => "Reel",
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
            url: _,
        } => Some(
            INSTAGRAM_TEMPLATE
                .replace("{0}", format_post_type(post_type.as_ref()))
                .replace("{1}", post_type.as_ref())
                .replace("{2}", data.as_ref()),
        ),

        ParsedURL::Tiktok { subdomain, domain, data } => {
            let kktiktok_url = format!("https://{}{}{}", subdomain.as_ref(), domain.as_ref().replace("tiktok", "kktiktok"), data.as_ref());
            Some(TIKTOK_TEMPLATE.replace("{0}", &kktiktok_url))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tiktok_link_substitution() {
        let result = sanitize_input("https://vm.tiktok.com/ZGdah868J/").await;
        assert_eq!(result, Some("[Post via TikTok](https://vm.kktiktok.com/ZGdah868J/)".to_string()));
    }

    #[tokio::test]
    async fn test_tiktok_link_substitution_no_trailing_slash() {
        let result = sanitize_input("https://vm.tiktok.com/ZGdah868J").await;
        assert_eq!(result, Some("[Post via TikTok](https://vm.kktiktok.com/ZGdah868J)".to_string()));
    }

    #[tokio::test]
    async fn test_instagram_post_substitution() {
        let result = sanitize_input("https://www.instagram.com/p/C9uiuh4KTlR/").await;
        assert_eq!(result, Some("[Post via Instagram](https://kkinstagram.com/p/C9uiuh4KTlR)".to_string()));
    }

    #[tokio::test]
    async fn test_instagram_reel_substitution() {
        let result = sanitize_input("https://www.instagram.com/reel/C6lmbgLLflh/").await;
        assert_eq!(result, Some("[Reel via Instagram](https://kkinstagram.com/reel/C6lmbgLLflh)".to_string()));
    }

    #[tokio::test]
    async fn test_instagram_reels_substitution() {
        let result = sanitize_input("https://www.instagram.com/reels/C6lmbgLLflh/").await;
        assert_eq!(result, Some("[Reel via Instagram](https://kkinstagram.com/reels/C6lmbgLLflh)".to_string()));
    }

    #[tokio::test]
    async fn test_tiktok_full_url_substitution() {
        let result = sanitize_input("https://www.tiktok.com/@misahere/video/7444680304293399850").await;
        assert_eq!(result, Some("[Post via TikTok](https://www.kktiktok.com/@misahere/video/7444680304293399850)".to_string()));
    }

    #[tokio::test]
    async fn test_twitter_unchanged() {
        let result = sanitize_input("https://x.com/loltyler1/status/179560257244486sf33").await;
        assert_eq!(result, Some("[@loltyler1 via X (Twitter)](https://fxtwitter.com/status/179560257244486sf33)".to_string()));
    }
}
