use anyhow::Error;
use poise::serenity_prelude as serenity;
use tracing::{debug, info};

use crate::handlers::ParsedURL;
use crate::Data;

pub async fn get_event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            info!("🤖 {} is Online", data_about_bot.user.name.to_string())
        }
        serenity::FullEvent::Message { new_message } => {
            handle_messages(ctx, new_message).await?;
        }
        _ => {}
    }
    Ok(())
}

async fn handle_messages(
    ctx: &serenity::Context,
    message: &serenity::Message,
) -> Result<(), Error> {
    debug!("Message detected");
    let input = message.content.trim();

    // Exits if message author is from bot itself
    if message.author.id == ctx.cache.current_user().id {
        return Ok(());
    }

    debug!("input = {input}");

    // Check for any supported URLs in the message
    if let Some(parsed_url) = ParsedURL::new(input) {
        let response = match parsed_url {
            ParsedURL::Tiktok { url } => {
                format!("TikTok link detected (TEST): {url}")
            }
            ParsedURL::Instagram {
                url: _,
                post_type,
                data,
            } => {
                format!("[{post_type} via Instagram](https://g.ddinstagram.com/{post_type}{data})")
            }
            ParsedURL::Twitter {
                url: _,
                username,
                data,
            } => {
                format!("[@{username} via X (Twitter)](https://fxtwitter.com/{username}{data})")
            }
        };

        debug!("response = {input}");
        // Reply to the message with the sanitized URL
        message.reply(ctx, response).await?;
    }

    Ok(())
}
