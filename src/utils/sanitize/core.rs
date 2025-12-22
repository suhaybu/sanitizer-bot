use std::sync::LazyLock;

use anyhow::{Context, Ok};
use regex::{Regex, RegexSet, RegexSetBuilder};
use scraper::Selector;

// Regex's for capturing urls
const INSTAGRAM_URL_PATTERN: &str =
    r"(?i)https?://(?:www\.)?instagram\.com/(?P<type>reels?|p)(?P<data>/[^/\s?)\]`]+)";
const REDDIT_URL_PATTERN: &str = r"(?i)https?://(?P<subdomain>(?:www\.|old\.)?)reddit\.com/(?P<subreddit>r/[^/]+)(?P<data>/[^?\s)\]`]*)?";
const TIKTOK_URL_PATTERN: &str =
    r"(?i)https?://(?P<subdomain>(?:\w{1,3}\.)?)(?P<domain>tiktok\.com)(?P<data>/[^?\s)\]`]*)";
const TWITCH_URL_PATTERN: &str = r"(?i)https?://(www\.)?(twitch\.tv/(?P<username>\w+)/clip/|clips\.twitch\.tv/)(?P<data>[^?\s)\]`]+)";
const TWITTER_URL_PATTERN: &str =
    r"(?i)https?://(www\.)?(twitter|x)\.com/(?P<username>\w+)(?P<data>/status/[^?\s)\]`]*)";

const URL_PATTERNS: &[&str] = &[
    INSTAGRAM_URL_PATTERN,
    REDDIT_URL_PATTERN,
    TIKTOK_URL_PATTERN,
    TWITCH_URL_PATTERN,
    TWITTER_URL_PATTERN,
];

static INDIVIDUAL_REGEXES: LazyLock<[Regex; 5]> = LazyLock::new(|| {
    [
        Regex::new(INSTAGRAM_URL_PATTERN).expect("Valid Instagram regex"),
        Regex::new(REDDIT_URL_PATTERN).expect("Valid Reddit regex"),
        Regex::new(TIKTOK_URL_PATTERN).expect("Valid TikTok regex"),
        Regex::new(TWITCH_URL_PATTERN).expect("Valid Twitch regex"),
        Regex::new(TWITTER_URL_PATTERN).expect("Valid Twitter regex"),
    ]
});

#[derive(Debug, Clone, Copy)]
pub enum Platform {
    Instagram = 0,
    Reddit = 1,
    TikTok = 2,
    Twitch = 3,
    Twitter = 4,
}

#[derive(Debug, Clone)]
pub struct UrlProcessor {
    platform: Platform,
    user_input: String, // Gets set in caller
    clean_url: Option<String>,
    username: Option<String>,
    post_type: Option<String>,
}

static REGEX_SET: LazyLock<anyhow::Result<RegexSet>> = LazyLock::new(|| {
    RegexSetBuilder::new(URL_PATTERNS)
        .case_insensitive(true)
        .multi_line(true)
        .build()
        .context("CRITICAL: Failed to build RegexSet")
});

// A simple & fast pre-check to see if a url is present.
pub fn contains_url(input: &str) -> bool {
    let input = input.to_lowercase();
    input.contains("instagram.com")
        || input.contains("reddit.com")
        || input.contains("tiktok.com")
        || input.contains("twitch.tv")
        || input.contains("twitter.com")
        || input.contains("x.com")
}

impl Platform {
    pub fn try_detect(input: &str) -> Option<Self> {
        let regex_set = REGEX_SET.as_ref().ok()?;
        let matches = regex_set.matches(input);
        tracing::debug!("Trying to detect a match in the url.");

        matches.iter().next().and_then(|idx| Self::from_index(idx))
    }

    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Instagram => "Instagram",
            Self::Reddit => "Reddit",
            Self::TikTok => "TikTok",
            Self::Twitch => "Twitch",
            Self::Twitter => "Twitter",
        }
    }

    const fn from_index(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(Self::Instagram),
            1 => Some(Self::Reddit),
            2 => Some(Self::TikTok),
            3 => Some(Self::Twitch),
            4 => Some(Self::Twitter),
            _ => None,
        }
    }

    const fn replacement_domain(&self) -> &'static str {
        match self {
            Self::Instagram => "kkinstagram.com",
            Self::Reddit => "rxddit.com",
            Self::TikTok => "kktiktok.com",
            Self::Twitch => "fxtwitch.seria.moe",
            Self::Twitter => "fxtwitter.com",
        }
    }
}

impl UrlProcessor {
    pub fn try_new(input: &str) -> Option<Self> {
        let platform = Platform::try_detect(input)?;
        Some(Self {
            platform,
            user_input: input.to_string(),
            clean_url: None,
            username: None,
            post_type: None,
        })
    }

    pub async fn capture_url(mut self) -> Option<Self> {
        let regex = &INDIVIDUAL_REGEXES[self.platform as usize];
        let captures = regex.captures(&self.user_input)?;

        match self.platform {
            Platform::Instagram => {
                tracing::debug!("Successfully matched the platform: Instagram");
                let post_type = captures.name("type")?.as_str();
                let data = captures.name("data")?.as_str();
                let clean_url = format!(
                    "https://www.{}/{}{}",
                    self.platform.replacement_domain(),
                    post_type,
                    data
                );

                self.post_type = Some(post_type.to_string());
                self.clean_url = Some(clean_url);
            }
            Platform::Reddit => {
                tracing::debug!("Successfully matched the platform: Reddit");
                let subdomain = captures
                    .name("subdomain")
                    .map(|m| m.as_str())
                    .unwrap_or("www.");
                let subreddit = captures.name("subreddit")?.as_str();
                let data = captures.name("data")?.as_str();

                let clean_url = format!("https://{}rxddit.com/{}{}", subdomain, subreddit, data);

                self.username = Some(subreddit.into());
                self.clean_url = Some(clean_url);
            }
            Platform::TikTok => {
                tracing::debug!("Successfully matched the platform: TikTok");
                let subdomain = captures.name("subdomain").map(|m| m.as_str()).unwrap_or("");
                let data = captures.name("data")?.as_str();
                let clean_url = format!(
                    "https://{}{}{}",
                    subdomain,
                    self.platform.replacement_domain(),
                    data
                );

                let username = match self.get_original_url() {
                    Some(original_url) => Self::get_author(original_url.as_str(), self.platform)
                        .await
                        .ok()
                        .flatten(),
                    None => None,
                };

                self.username = username;
                self.clean_url = Some(clean_url);
            }
            Platform::Twitch => {
                tracing::debug!("Successfully matched the platform: Twitch");
                let username = captures.name("username").map(|m| m.as_str().to_string());
                let data = captures.name("data")?.as_str();

                let clean_url = if let Some(ref user) = username {
                    // Format: twitch.tv/username/clip/data
                    format!(
                        "https://{}/{}/clip/{}",
                        self.platform.replacement_domain(),
                        user,
                        data
                    )
                } else {
                    // Format: clips.twitch.tv/clipid
                    format!(
                        "https://{}/clip/{}",
                        self.platform.replacement_domain(),
                        data
                    )
                };

                let username = match username {
                    Some(u) => Some(u),
                    None => Self::get_author(&clean_url, self.platform)
                        .await
                        .ok()
                        .flatten(),
                };

                self.clean_url = Some(clean_url);
                self.username = username;
            }
            Platform::Twitter => {
                tracing::debug!("Successfully matched the platform: Twitter");
                let username = captures.name("username")?.as_str();
                let data = captures.name("data")?.as_str();
                let clean_url = format!(
                    "https://{}/{}{}",
                    self.platform.replacement_domain(),
                    username,
                    data
                );

                self.clean_url = Some(clean_url);
                self.username = Some(username.to_string());
            }
        }

        Some(self)
    }

    pub fn format_output(self) -> Option<String> {
        tracing::debug!("Attempting to format the final output.");
        let clean_url = self.clean_url?;

        match self.platform {
            Platform::Instagram => {
                let post_type_display = self
                    .post_type
                    .as_ref()
                    .map(|pt| match pt.to_lowercase().as_str() {
                        "reels" | "reel" => "Reel",
                        "p" => "Post",
                        _ => "Post",
                    })
                    .unwrap_or("Post");

                Some(format!(
                    "[{} via {}]({})",
                    post_type_display,
                    self.platform.display_name(),
                    clean_url
                ))
            }
            Platform::Reddit => match self.username {
                Some(username) => Some(format!(
                    "[{} via {}]({})",
                    username,
                    self.platform.display_name(),
                    clean_url
                )),
                None => Some(format!(
                    "[Post via {}]({})",
                    self.platform.display_name(),
                    clean_url
                )),
            },
            Platform::TikTok | Platform::Twitch | Platform::Twitter => match self.username {
                Some(username) => Some(format!(
                    "[@{} via {}]({})",
                    username,
                    self.platform.display_name(),
                    clean_url
                )),
                None => Some(format!(
                    "[Post via {}]({})",
                    self.platform.display_name(),
                    clean_url
                )),
            },
        }
    }

    pub fn get_original_url(&self) -> Option<String> {
        tracing::debug!("Getting original url");
        let regex = &INDIVIDUAL_REGEXES[self.platform as usize];
        regex.find(&self.user_input).map(|m| m.as_str().to_string())
    }

    // Retrieve's the author name by attempting to curl the url and parse the output.
    async fn get_author(url: &str, platform: Platform) -> anyhow::Result<Option<String>> {
        tracing::debug!("Attempting to get author, building reqwest client");
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (compatible; Discordbot/2.0; +https://discordapp.com/)")
            .redirect(reqwest::redirect::Policy::none())
            .build()?;

        let author = match platform {
            Platform::TikTok => {
                let response = client.get(url).send().await?;
                let Some(location) = response.headers().get("location") else {
                    return Ok(None);
                };
                let location = location.to_str()?;

                let Some(start) = location.find("/@") else {
                    return Ok(None);
                };
                let Some(end) = location[start + 2..].find("/video/") else {
                    return Ok(None);
                };

                if end == 0 {
                    return Ok(None);
                }

                Some(location[start + 2..start + 2 + end].to_string())
            }
            // Twitter doesn't ever get ran, added for future use.
            Platform::Twitch | Platform::Twitter => {
                let html = client.get(url).send().await?.text().await?;
                let document = scraper::Html::parse_document(&html);
                let selector_property = match platform {
                    Platform::Twitch => "og:title",
                    Platform::Twitter => "twitter:creator",
                    _ => unreachable!(),
                };

                let selector = Selector::parse(&format!("meta[property='{}']", selector_property))
                    .expect("valid CSS selector");

                document
                    .select(&selector)
                    .next()
                    .and_then(|el| el.value().attr("content"))
                    .map(|content| match platform {
                        Platform::Twitch => {
                            content.split(" - ").next().unwrap_or(content).to_string()
                        }
                        _ => content.to_string(),
                    })
            }
            _ => None,
        };
        Ok(author)
    }
}
