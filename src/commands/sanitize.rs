use anyhow::Result;
use poise::serenity_prelude as serenity;
use tracing::debug;

use crate::handlers::handle_response_interaction;
use crate::handlers::sanitize_input;
use crate::Context;

const INVALID_URL_MESSAGE: &str =
    "❌ Invalid URL. Please provide a valid TikTok, Instagram, or Twitter/X link.";

/// Fix the embed of your link! 🫧
#[poise::command(
    slash_command,
    rename = "sanitize",
    default_member_permissions = "SEND_MESSAGES",
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn sanitize_slash(
    ctx: Context<'_>,
    #[description = "Your link goes here"] link: String,
) -> Result<()> {
    sanitize_handler(ctx, link).await
}

#[poise::command(
    context_menu_command = "Sanitize",
    required_permissions = "SEND_MESSAGES",
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn sanitize_menu(ctx: Context<'_>, link: serenity::Message) -> Result<()> {
    sanitize_handler(ctx, link.content).await
}

async fn sanitize_handler(ctx: Context<'_>, link: String) -> Result<()> {
    debug!("Sanitize command invoked for link: {}", link);
    debug!("Channel type: {:?}", ctx.channel_id());

    let _ = ctx.defer().await; // sends "Is thinking..." before response
    debug!("Deferred initial response");

    // Get initial response ready
    let response = match sanitize_input(&link).await {
        Some(sanitized_url) => {
            debug!("URL sanitized successfully: {}", sanitized_url);
            sanitized_url
        }
        None => {
            debug!("Invalid URL provided");
            INVALID_URL_MESSAGE.to_string()
        }
    };

    // Defer first to get more time
    ctx.defer_response(false).await?;

    let bot_message = ctx.say(response).await?;
    let bot_message = bot_message.message().await?;

    handle_response_interaction(&ctx, &bot_message).await?;

    Ok(())
}
