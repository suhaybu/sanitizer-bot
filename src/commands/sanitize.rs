use anyhow::Error;
use poise::serenity_prelude as serenity;

use crate::handlers::handle_interaction_response;
use crate::handlers::sanitize_input;
use crate::Context;

const INVALID_URL_MESSAGE: &str =
    "❌ Invalid URL. Please provide a valid TikTok, Instagram, or Twitter/X link.";

/// Fix the embed of your link! 🫧
#[poise::command(context_menu_command = "Sanitize", slash_command)]
pub async fn sanitize(
    ctx: Context<'_>,
    #[description = "Your link goes here"] link: serenity::Message,
) -> Result<(), Error> {
    let _ = ctx.defer().await; // sends "Is thinking..." before response

    // Get initial response ready
    let response = match sanitize_input(&link.content).await {
        Some(sanitized_url) => sanitized_url,
        None => INVALID_URL_MESSAGE.to_string(),
    };

    if let Context::Application(application_ctx) = ctx {
        // Defer first to get more time
        application_ctx.defer_response(false).await?;

        let bot_message = ctx.say(response).await?;
        let bot_message = bot_message.message().await?;

        handle_interaction_response(&ctx, &bot_message).await?;
    }

    Ok(())
}
