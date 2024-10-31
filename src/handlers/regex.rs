// Todo
#![allow(dead_code)]
use std::sync::LazyLock;

use regex::{Regex, RegexSet, RegexSetBuilder, SetMatches};

const TIKTOK_PATTERN: &str = r"(?i)https?://(?:\w{1,3}\.)?tiktok\.com/[^/]+/?\S*";
const INSTAGRAM_PATTERN: &str =
    r"(?i)https?://(?:www\.)?instagram\.com/(?P<type>reel|p)(?P<data>/[^/\s?]+)";
const TWITTER_PATTERN: &str =
    r"(?i)https?://(www\.)?(twitter|x)\.com/(?P<username>\w+)(?P<data>/status/[^?\s]*)";

static PATTERNS: LazyLock<RegexSet> = LazyLock::new(|| {
    // Take note of the order the Patterns are loaded, that will be essenstial for the get_match fn
    RegexSetBuilder::new([TIKTOK_PATTERN, INSTAGRAM_PATTERN, TWITTER_PATTERN])
        .case_insensitive(true)
        .multi_line(true)
        .build()
        .unwrap_or_else(|err| {
            eprintln!("Failed to build RegexSet PATTERNS: {err}");
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

fn get_response() -> String {
    todo!()
}

fn find_matches(input: &str) -> Option<Vec<u8>> {
    let matches = PATTERNS.matches_at(input, 0);
    if matches.matched_any() {
        let mut response = Vec::new();

        for match_index in matches.iter() {
            response.push(match_index as u8);
        }
        Some(response)
    } else {
        None
    }
}

fn get_parsed_url(input: &str, match_index: u8) -> Option<ParsedURL> {
    match match_index {
        0 => {
            // Tiktok match
            #[cfg(test)]
            println!("Accessed branch 1");
            let re = Regex::new(TIKTOK_PATTERN).unwrap();
            return re.captures(input).map(|captures| ParsedURL::Tiktok {
                url: captures.get(0).unwrap().as_str().to_string(),
            });
        }
        1 => {
            // Instagram match
            #[cfg(test)]
            println!("Accessed branch 2");
            let re = Regex::new(INSTAGRAM_PATTERN).unwrap();
            return re.captures(input).map(|captures| ParsedURL::Instagram {
                url: captures.get(0).unwrap().as_str().to_string(),
                post_type: captures.name("type").unwrap().as_str().to_string(),
                data: captures.name("data").unwrap().as_str().to_string(),
            });
        }
        2 => {
            // Twitter match
            #[cfg(test)]
            println!("Accessed branch 3");
            let re = Regex::new(TWITTER_PATTERN).unwrap();
            return re.captures(input).map(|captures| ParsedURL::Twitter {
                url: captures.get(0).unwrap().as_str().to_string(),
                username: captures.name("username").unwrap().as_str().to_string(),
                data: captures.name("data").unwrap().as_str().to_string(),
            });
        }
        _ => None,
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     // TODO: The / in the end of the input URL mirrors the output URL.
//     // 		 Need to standerdize output regardless of input
//     fn test_tiktok_url() {
//         let matches = get_match("https://vt.tiktok.com/ZSYXeWygm/");
//         assert_eq!(
//             matches,
//             Some(ParsedURL::Tiktok {
//                 url: ("https://vt.tiktok.com/ZSYXeWygm/".to_string())
//             })
//         );
//     }

//     #[test]
//     fn test_instagram_post_url() {
//         let matches = get_match("https://instagram.com/p/CMeJMFBs66n/");
//         assert_eq!(
//             matches,
//             Some(ParsedURL::Instagram {
//                 url: "https://instagram.com/p/CMeJMFBs66n".to_string(),
//                 post_type: "p".to_string(),
//                 data: "/CMeJMFBs66n".to_string(),
//             }),
//         );
//     }

//     #[test]
//     fn test_instagram_reel_url() {
//         let matches = get_match("https://www.instagram.com/reel/C6lmbgLLflh/");
//         assert_eq!(
//             matches,
//             Some(ParsedURL::Instagram {
//                 url: "https://www.instagram.com/reel/C6lmbgLLflh".to_string(),
//                 post_type: "reel".to_string(),
//                 data: "/C6lmbgLLflh".to_string(),
//             }),
//         );
//     }

//     #[test]
//     fn test_twitter_url() {
//         let matches = get_match("https://x.com/loltyler1/status/179560257244486sf33");
//         assert_eq!(
//             matches,
//             Some(ParsedURL::Twitter {
//                 url: "https://x.com/loltyler1/status/179560257244486sf33".to_string(),
//                 username: "loltyler1".to_string(),
//                 data: "/status/179560257244486sf33".to_string(),
//             }),
//         );
//     }

//     #[test]
//     fn test_twitter_with_www_url() {
//         let matches = get_match("http://www.twitter.com/rit_chill/status/1756388311445221859");
//         assert_eq!(
//             matches,
//             Some(ParsedURL::Twitter {
//                 url: "http://www.twitter.com/rit_chill/status/1756388311445221859".to_string(),
//                 username: "rit_chill".to_string(),
//                 data: "/status/1756388311445221859".to_string(),
//             }),
//         );
//     }
// }
