#![allow(dead_code)]
use regex::{Regex, RegexSet, RegexSetBuilder};
use std::sync::LazyLock;
use tracing;

const TIKTOK_PATTERN: &str = r"(?i)https?://(?:\w{1,3}\.)?tiktok\.com/[^/]+/?\S*";
const INSTAGRAM_PATTERN: &str =
    r"(?i)https?://(?:www\.)?instagram\.com/(?P<type>reel|p)(?P<data>/[^/\s?]+)";
const TWITTER_PATTERN: &str =
    r"(?i)https?://(www\.)?(twitter|x)\.com/(?P<username>\w+)(?P<data>/status/[^?\s]*)";

const URL_PATTERNS: &[&str] = &[TIKTOK_PATTERN, INSTAGRAM_PATTERN, TWITTER_PATTERN];

static PATTERNS: LazyLock<RegexSet> = LazyLock::new(|| {
    // Take note of the order the Patterns are loaded, that will be essenstial for the get_match fn
    RegexSetBuilder::new(URL_PATTERNS)
        .case_insensitive(true)
        .multi_line(true)
        .build()
        .unwrap_or_else(|err| {
            tracing::error!("Failed to build RegexSet: {}", err);
            panic!("RegexSet initialization failed")
        })
});

#[derive(Debug, PartialEq)]
enum ParsedURL {
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
    fn from_captures(variant_index: usize, captures: regex::Captures) -> Option<Self> {
        match variant_index {
            0 => Some(ParsedURL::Tiktok {
                url: captures.get(0).unwrap().as_str().to_string(),
            }),
            1 => Some(ParsedURL::Instagram {
                url: captures.get(0).unwrap().as_str().to_string(),
                post_type: captures.name("type").unwrap().as_str().to_string(),
                data: captures.name("data").unwrap().as_str().to_string(),
            }),
            2 => Some(ParsedURL::Twitter {
                url: captures.get(0).unwrap().as_str().to_string(),
                username: captures.name("username").unwrap().as_str().to_string(),
                data: captures.name("data").unwrap().as_str().to_string(),
            }),
            _ => None,
        }
    }
}

fn find_match_index(input: &str) -> Option<Vec<u8>> {
    let matches = PATTERNS.matches(input);
    if matches.matched_any() {
        let response: Vec<u8> = matches.iter().map(|idx| idx as u8).collect();
        Some(response)
    } else {
        None
    }
}

fn get_parsed_url(input: &str, match_index: u8) -> Option<ParsedURL> {
    let pattern = URL_PATTERNS.get(match_index as usize)?;

    let re = match Regex::new(pattern) {
        Ok(re) => re,
        Err(err) => {
            tracing::error!("Failed to compile regex: {}", err);
            return None;
        }
    };

    let captures = re.captures(input)?;
    ParsedURL::from_captures(match_index as usize, captures)
}

fn get_match(user_input: &str) -> Option<ParsedURL> {
    find_match_index(user_input)
        .and_then(|matches| matches.first().copied())
        .and_then(|match_index| get_parsed_url(user_input, match_index))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // TODO: The / in the end of the input URL mirrors the output URL.
    // 		 Need to standerdize output regardless of input
    fn test_tiktok_url() {
        let matches = get_match("https://vt.tiktok.com/ZSYXeWygm/");
        assert_eq!(
            matches,
            Some(ParsedURL::Tiktok {
                url: ("https://vt.tiktok.com/ZSYXeWygm/".to_string())
            })
        );
    }

    #[test]
    fn test_instagram_post_url() {
        let matches = get_match("https://instagram.com/p/CMeJMFBs66n/");
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
        let matches = get_match("https://www.instagram.com/reel/C6lmbgLLflh/");
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
        let matches = get_match("https://x.com/loltyler1/status/179560257244486sf33");
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
        let matches = get_match("http://www.twitter.com/rit_chill/status/1756388311445221859");
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
