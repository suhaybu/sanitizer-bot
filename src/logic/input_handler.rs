// Todo

use regex::{Regex, RegexSet};

enum URLMatchType {
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

fn get_match(input: &str) -> Option<URLMatchType> {
    let tiktok_pattern = r"(?i)https?://(?:\w{1,3}\.)?tiktok\.com/[^/]+/?\S*";
    let instagram_pattern =
        r"(?i)https?://(?:www\.)?instagram\.com/(?P<type>reel|p)(?P<data>/[^/\s?]+)";
    let twitter_pattern =
        r"(?i)https?://(www\.)?(twitter|x)\.com/(?P<username>\w+)(?P<data>/status/[^?\s]*)";

    let patterns = RegexSet::new(&[tiktok_pattern, instagram_pattern, twitter_pattern]).unwrap();
    let matches = patterns.matches(input);

    let matched_index = match matches.iter().next() {
        Some(index) => index,
        None => return None,
    };

    match matched_index {
        0 => {
            let re = Regex::new(tiktok_pattern).unwrap();
            re.captures(input).map(|captures| URLMatchType::Tiktok {
                url: captures.get(1).unwrap().as_str().to_string(),
            })
        }
        _ => None,
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_tiktok_url() {
//         let matches = get_match("https://vt.tiktok.com/ZSYXeWygm/");
//         assert_eq!(matches, vec![0]); // Should match the TikTok pattern
//     }

//     #[test]
//     fn test_instagram_post_url() {
//         let matches = get_match("https://instagram.com/p/CMeJMFBs66n/");
//         assert_eq!(matches, vec![1]); // Should match the Instagram post pattern
//     }

//     #[test]
//     fn test_instagram_reel_url() {
//         let matches = get_match("https://www.instagram.com/reel/C6lmbgLLflh/");
//         assert_eq!(matches, vec![1]); // Should match the Instagram reel pattern
//     }

//     #[test]
//     fn test_twitter_url() {
//         let matches = get_match("https://x.com/loltyler1/status/179560257244486sf33");
//         assert_eq!(matches, vec![2]); // Should match the Twitter/X pattern
//     }

//     #[test]
//     fn test_twitter_with_www_url() {
//         let matches = get_match("http://www.twitter.com/rit_chill/status/1756388311445221859");
//         assert_eq!(matches, vec![2]); // Should match the Twitter pattern
//     }
// }
