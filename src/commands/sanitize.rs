use anyhow::Error;

use crate::handlers::sanitize_input;
use crate::Context;

const INVALID_URL_MESSAGE: &str =
    "❌ Invalid URL. Please provide a valid TikTok, Instagram, or Twitter/X link.";

/// Fix the embed of your link! 🫧
#[poise::command(slash_command)]
pub async fn sanitize(
    ctx: Context<'_>,
    #[description = "Your link goes here"] link: String,
) -> Result<(), Error> {
    let response = match sanitize_input(&link).await {
        Some(sanitized_url) => sanitized_url,
        None => INVALID_URL_MESSAGE.to_string(),
    };

    ctx.say(response).await?;
    Ok(())
}
