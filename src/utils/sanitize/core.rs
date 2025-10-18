use std::sync::LazyLock;

use anyhow::Context;
use regex::{Regex, RegexSet, RegexSetBuilder};

// Regex's for capturing urls
const INSTAGRAM_URL_PATTERN: &str =
    r"(?i)https?://(?:www\.)?instagram\.com/(?P<type>reels?|p)(?P<data>/[^/\s?]+)";
const TIKTOK_URL_PATTERN: &str =
    r"(?i)https?://(?P<subdomain>(?:\w{1,3}\.)?)(?P<domain>tiktok\.com)(?P<data>/\S*)";
const TWITTER_URL_PATTERN: &str =
    r"(?i)https?://(www\.)?(twitter|x)\.com/(?P<username>\w+)(?P<data>/status/[^?\s]*)";

const URL_PATTERNS: &[&str] = &[
    INSTAGRAM_URL_PATTERN,
    TIKTOK_URL_PATTERN,
    TWITTER_URL_PATTERN,
];

static INDIVIDUAL_REGEXES: LazyLock<[Regex; 3]> = LazyLock::new(|| {
    [
        Regex::new(INSTAGRAM_URL_PATTERN).expect("Valid Instagram regex"),
        Regex::new(TIKTOK_URL_PATTERN).expect("Valid TikTok regex"),
        Regex::new(TWITTER_URL_PATTERN).expect("Valid Twitter regex"),
    ]
});

#[derive(Debug, Clone, Copy)]
pub enum Platform {
    Instagram = 0,
    TikTok = 1,
    Twitter = 2,
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
        || input.contains("tiktok.com")
        || input.contains("twitter.com")
        || input.contains("x.com")
}

impl Platform {
    // pub const ALL: [Self; 3] = [Self::Instagram, Self::TikTok, Self::Twitter];

    pub fn try_detect(input: &str) -> Option<Self> {
        let regex_set = REGEX_SET.as_ref().ok()?;
        let matches = regex_set.matches(input);

        matches.iter().next().and_then(|idx| Self::from_index(idx))
    }

    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Instagram => "Instagram",
            Self::TikTok => "TikTok",
            Self::Twitter => "Twitter",
        }
    }

    const fn from_index(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(Self::Instagram),
            1 => Some(Self::TikTok),
            2 => Some(Self::Twitter),
            _ => None,
        }
    }

    const fn replacement_domain(&self) -> &'static str {
        match self {
            Self::Instagram => "kkinstagram.com",
            Self::TikTok => "kktiktok.com",
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

    pub fn capture_url(mut self) -> Option<Self> {
        let regex = &INDIVIDUAL_REGEXES[self.platform as usize];
        let captures = regex.captures(&self.user_input)?;

        match self.platform {
            Platform::Instagram => {
                let post_type = captures.name("type")?.as_str();
                let data = captures.name("data")?.as_str();
                let clean_url = format!(
                    "https://{}/{}{}",
                    self.platform.replacement_domain(),
                    post_type,
                    data
                );

                self.post_type = Some(post_type.to_string());
                self.clean_url = Some(clean_url);
            }
            Platform::TikTok => {
                let subdomain = captures.name("subdomain").map(|m| m.as_str()).unwrap_or("");
                let data = captures.name("data")?.as_str();
                let clean_url = format!(
                    "https://{}{}{}",
                    subdomain,
                    self.platform.replacement_domain(),
                    data
                );

                self.clean_url = Some(clean_url);
            }
            Platform::Twitter => {
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
            Platform::TikTok => Some(format!(
                "[Post via {}]({})",
                self.platform.display_name(),
                clean_url
            )),

            Platform::Twitter => match self.username {
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
        let regex = &INDIVIDUAL_REGEXES[self.platform as usize];
        regex.find(&self.user_input).map(|m| m.as_str().to_string())
    }
}
