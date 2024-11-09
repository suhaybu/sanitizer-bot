use crate::handlers::ParsedURL;
use crate::Context;
use anyhow::Error;
use poise::serenity_prelude as serenity;

/// Fix the embed of your link! 🫧
#[poise::command(slash_command)]
pub async fn sanitize(
    ctx: Context<'_>,
    #[description = "Your link goes here"] link: String,
) -> Result<(), Error> {
    let response = match ParsedURL::new(&link) {
        Some(parsed_url) => match parsed_url {
            ParsedURL::Tiktok { url } => {
                format!("TikTok link detected (TEST): {url}")
            }
            ParsedURL::Instagram {
                url: _,
                post_type,
                data,
            } => {
                format!("[{post_type} via Instagram](https://ddinstagram.com/{post_type}{data})")
            }
            ParsedURL::Twitter {
                url: _,
                username,
                data,
            } => {
                format!("[@{username} via X (Twitter)](https://fxtwitter.com/{username}{data})")
            }
        },
        None => "❌ Invalid URL. Please provide a valid TikTok, Instagram, or Twitter/X link."
            .to_string(),
    };

    ctx.say(response).await?;
    Ok(())
}
