#![allow(dead_code)]
use regex::{Regex, RegexSet, RegexSetBuilder};
use std::sync::LazyLock;
use tracing;

const TIKTOK_URL_PATTERN: &str = r"(?i)https?://(?:\w{1,3}\.)?tiktok\.com/[^/]+/?\S*";
const INSTAGRAM_URL_PATTERN: &str =
    r"(?i)https?://(?:www\.)?instagram\.com/(?P<type>reel|p)(?P<data>/[^/\s?]+)";
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
pub enum ParsedURL {
    Tiktok {
        url: String,
    },
    Instagram {
        url: String,
        post_type: String,
        data: String,
    },
    Twitter {
        url: String,
        username: String,
        data: String,
    },
}

impl ParsedURL {
    // Creates a new ParsedURL (enum) if there is a match, else returns None
    pub fn new(user_input: String) -> Option<Self> {
        let matches = PATTERNS.matches(&user_input);
        let match_index = matches.matched_any().then(|| matches.iter().next())??;

        let pattern = URL_PATTERNS.get(match_index)?;
        let re = Regex::new(pattern).ok()?;
        let captures = re.captures(&user_input)?;

        let url = captures.get(0).unwrap().as_str().to_string(); // Stores base of url => url

        match match_index {
            0 => Some(ParsedURL::Tiktok { url }),
            1 => Some(ParsedURL::Instagram {
                url,
                post_type: captures.name("type").unwrap().as_str().to_string(),
                data: captures.name("data").unwrap().as_str().to_string(),
            }),
            2 => Some(ParsedURL::Twitter {
                url,
                username: captures.name("username").unwrap().as_str().to_string(),
                data: captures.name("data").unwrap().as_str().to_string(),
            }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // This function takes the user input, if there is a match, it returns a ParsedURL enum
    fn parse_url(user_input: String) -> Option<ParsedURL> {
        ParsedURL::new(user_input)
    }

    #[test]
    // TODO: The / in the end of the input URL mirrors the output URL.
    // 		 Need to standerdize output regardless of input
    fn test_tiktok_url() {
        let matches = parse_url("https://vt.tiktok.com/ZSYXeWygm/".to_string());
        assert_eq!(
            matches,
            Some(ParsedURL::Tiktok {
                url: ("https://vt.tiktok.com/ZSYXeWygm/".to_string())
            })
        );
    }

    #[test]
    fn test_instagram_post_url() {
        let matches = parse_url("https://instagram.com/p/CMeJMFBs66n/".to_string());
        assert_eq!(
            matches,
            Some(ParsedURL::Instagram {
                url: "https://instagram.com/p/CMeJMFBs66n".to_string(),
                post_type: "p".to_string(),
                data: "/CMeJMFBs66n".to_string(),
            }),
        );
    }

    #[test]
    fn test_instagram_reel_url() {
        let matches = parse_url("https://www.instagram.com/reel/C6lmbgLLflh/".to_string());
        assert_eq!(
            matches,
            Some(ParsedURL::Instagram {
                url: "https://www.instagram.com/reel/C6lmbgLLflh".to_string(),
                post_type: "reel".to_string(),
                data: "/C6lmbgLLflh".to_string(),
            }),
        );
    }

    #[test]
    fn test_twitter_url() {
        let matches = parse_url("https://x.com/loltyler1/status/179560257244486sf33".to_string());
        assert_eq!(
            matches,
            Some(ParsedURL::Twitter {
                url: "https://x.com/loltyler1/status/179560257244486sf33".to_string(),
                username: "loltyler1".to_string(),
                data: "/status/179560257244486sf33".to_string(),
            }),
        );
    }

    #[test]
    fn test_twitter_with_www_url() {
        let matches =
            parse_url("http://www.twitter.com/rit_chill/status/1756388311445221859".to_string());
        assert_eq!(
            matches,
            Some(ParsedURL::Twitter {
                url: "http://www.twitter.com/rit_chill/status/1756388311445221859".to_string(),
                username: "rit_chill".to_string(),
                data: "/status/1756388311445221859".to_string(),
            }),
        );
    }
}
