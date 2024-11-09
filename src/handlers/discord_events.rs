use anyhow::Error;
use poise::serenity_prelude::{self as serenity, EmojiId};
use tracing::{debug, info};

use crate::handlers::sanitize_input;
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
            on_message(ctx, new_message).await?;
        }
        _ => {}
    }
    Ok(())
}

async fn on_message(ctx: &serenity::Context, message: &serenity::Message) -> Result<(), Error> {
    debug!("Message detected");

    // Exits if message author is from bot itself
    if message.author.id == ctx.cache.current_user().id {
        return Ok(());
    }

    let input = message.content.trim();
    debug!("input = {input}");

    let response = match sanitize_input(input).await {
        None => return Ok(()), // Exit early if no match
        Some(response) => response,
    };

    message.reply(ctx, response).await?;
    message
        .react(
            ctx,
            serenity::ReactionType::Custom {
                animated: false,
                id: EmojiId::new(1206376642042138724),
                name: Some("Sanitized".to_string()),
            },
        )
        .await?;

    Ok(())
}
