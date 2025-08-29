// Regex logic
use regex::{Regex, RegexSet, RegexSetBuilder};
use std::borrow::Cow;
use std::sync::LazyLock;
use tracing;

const TIKTOK_URL_PATTERN: &str = r"(?i)https?://(?P<subdomain>(?:\w{1,3}\.)?)(?P<domain>tiktok\.com)(?P<data>/\S*)";
const INSTAGRAM_URL_PATTERN: &str =
    r"(?i)https?://(?:www\.)?instagram\.com/(?P<type>reels?|p)(?P<data>/[^/\s?]+)";
const TWITTER_URL_PATTERN: &str =
    r"(?i)https?://(www\.)?(twitter|x)\.com/(?P<username>\w+)(?P<data>/status/[^?\s]*)";

const URL_PATTERNS: &[&str] = &[
    TIKTOK_URL_PATTERN,
    INSTAGRAM_URL_PATTERN,
    TWITTER_URL_PATTERN,
];

type RegexResult<T> = Result<T, regex::Error>;

static PATTERNS: LazyLock<RegexSet> = LazyLock::new(|| {
    // Take note of the order the Patterns are loaded, that will be essenstial for the get_match fn
    build_regex_set().unwrap_or_else(|err| {
        tracing::error!("Failed to build RegexSet: {}", err);
        panic!("RegexSet initialization failed")
    })
});

fn build_regex_set() -> RegexResult<RegexSet> {
    RegexSetBuilder::new(URL_PATTERNS)
        .case_insensitive(true)
        .multi_line(true)
        .build()
}

#[derive(Debug, PartialEq)]
pub enum ParsedURL<'a> {
    /// Captures for TikTok URLs
    ///
    /// Example URL: "https://vm.tiktok.com/ZGdah868J/"
    ///
    /// Captures:
    ///   - subdomain: "vm." (optional, can be empty)
    ///   - domain: "tiktok.com"
    ///   - data: "/ZGdah868J/"
    Tiktok { subdomain: Cow<'a, str>, domain: Cow<'a, str>, data: Cow<'a, str> },

    /// Captures for Instagram URLs
    ///
    /// Example URL: "https://www.instagram.com/p/C9uiuh4KTlR/"
    ///
    /// Captures:
    ///   - url: "https://www.instagram.com/p/C9uiuh4KTlR"
    ///   - post_type: "p"      (can be "p"|"reel")
    ///   - data: "/C9uiuh4KTlR"
    Instagram {
        url: Cow<'a, str>,
        post_type: Cow<'a, str>,
        data: Cow<'a, str>,
    },

    /// Captures for Twitter/X URLs
    ///
    /// Example URL: "https://www.twitter.com/rit_chill/status/1756388311445221859"
    ///
    /// Captures:
    ///   - url: "https://www.twitter.com/rit_chill/status/1756388311445221859"
    ///   - username: "rit_chill"
    ///   - data: "/status/1756388311445221859"
    Twitter {
        url: Cow<'a, str>,
        username: Cow<'a, str>,
        data: Cow<'a, str>,
    },
}

impl<'a> ParsedURL<'a> {
    // Creates a new ParsedURL (enum) if there is a match, else returns None
    pub fn new(user_input: &'a str) -> Option<Self> {
        let matches = PATTERNS.matches(&user_input);
        let match_index = matches.matched_any().then(|| matches.iter().next())??;

        let pattern = URL_PATTERNS.get(match_index)?;
        let re = Regex::new(pattern).ok()?;
        let captures = re.captures(&user_input)?;

        match match_index {
            0 => Some(ParsedURL::Tiktok {
                subdomain: captures.name("subdomain").map(|m| m.as_str()).unwrap_or("").into(),
                domain: captures.name("domain")?.as_str().into(),
                data: captures.name("data")?.as_str().into(),
            }),
            1 => Some(ParsedURL::Instagram {
                url: captures.get(0)?.as_str().into(),
                post_type: captures.name("type")?.as_str().into(),
                data: captures.name("data")?.as_str().into(),
            }),
            2 => Some(ParsedURL::Twitter {
                url: captures.get(0)?.as_str().into(),
                username: captures.name("username")?.as_str().into(),
                data: captures.name("data")?.as_str().into(),
            }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // This function takes the user input, if there is a match, it returns a ParsedURL enum
    fn parse_url(user_input: &str) -> Option<ParsedURL<'_>> {
        ParsedURL::new(&user_input)
    }

    #[test]
    fn test_tiktok_url() {
        let matches = parse_url("https://vm.tiktok.com/ZGdah868J/");
        assert_eq!(
            matches,
            Some(ParsedURL::Tiktok {
                subdomain: Cow::Borrowed("vm."),
                domain: Cow::Borrowed("tiktok.com"),
                data: Cow::Borrowed("/ZGdah868J/"),
            })
        );
    }

    #[test]
    fn test_tiktok_url_no_trailing_slash() {
        let matches = parse_url("https://vm.tiktok.com/ZGdah868J");
        assert_eq!(
            matches,
            Some(ParsedURL::Tiktok {
                subdomain: Cow::Borrowed("vm."),
                domain: Cow::Borrowed("tiktok.com"),
                data: Cow::Borrowed("/ZGdah868J"),
            })
        );
    }

    #[test]
    fn test_tiktok_full_url_format() {
        let matches = parse_url("https://www.tiktok.com/@testuser/video/1234567890");
        assert_eq!(
            matches,
            Some(ParsedURL::Tiktok {
                subdomain: Cow::Borrowed("www."),
                domain: Cow::Borrowed("tiktok.com"),
                data: Cow::Borrowed("/@testuser/video/1234567890"),
            })
        );
    }

    #[test]
    fn test_instagram_post_url() {
        let matches = parse_url("https://instagram.com/p/CMeJMFBs66n/");
        assert_eq!(
            matches,
            Some(ParsedURL::Instagram {
                url: Cow::Borrowed("https://instagram.com/p/CMeJMFBs66n"),
                post_type: Cow::Borrowed("p"),
                data: Cow::Borrowed("/CMeJMFBs66n"),
            })
        );
    }

    #[test]
    fn test_instagram_reel_url() {
        let matches = parse_url("https://www.instagram.com/reel/C6lmbgLLflh/");
        assert_eq!(
            matches,
            Some(ParsedURL::Instagram {
                url: Cow::Borrowed("https://www.instagram.com/reel/C6lmbgLLflh"),
                post_type: Cow::Borrowed("reel"),
                data: Cow::Borrowed("/C6lmbgLLflh"),
            })
        );
    }

    #[test]
    fn test_instagram_reels_url() {
        let matches = parse_url("https://www.instagram.com/reels/C6lmbgLLflh/");
        assert_eq!(
            matches,
            Some(ParsedURL::Instagram {
                url: Cow::Borrowed("https://www.instagram.com/reels/C6lmbgLLflh"),
                post_type: Cow::Borrowed("reels"),
                data: Cow::Borrowed("/C6lmbgLLflh"),
            })
        );
    }

    #[test]
    fn test_twitter_url() {
        let matches = parse_url("https://x.com/loltyler1/status/179560257244486sf33");
        assert_eq!(
            matches,
            Some(ParsedURL::Twitter {
                url: Cow::Borrowed("https://x.com/loltyler1/status/179560257244486sf33"),
                username: Cow::Borrowed("loltyler1"),
                data: Cow::Borrowed("/status/179560257244486sf33"),
            })
        );
    }

    #[test]
    fn test_twitter_with_www_url() {
        let matches = parse_url("http://www.twitter.com/rit_chill/status/1756388311445221859");
        assert_eq!(
            matches,
            Some(ParsedURL::Twitter {
                url: Cow::Borrowed("http://www.twitter.com/rit_chill/status/1756388311445221859"),
                username: Cow::Borrowed("rit_chill"),
                data: Cow::Borrowed("/status/1756388311445221859"),
            })
        );
    }
}
