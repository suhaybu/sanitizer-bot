use anyhow::Error;
use poise::serenity_prelude as serenity;
use tracing::{debug, info};

use crate::Data;
use crate::handlers::{handle_response_event, sanitize_input};

use super::db::models::ServerConfig;

pub async fn get_event_handler(
    framework: poise::FrameworkContext<'_, Data, Error>,
    event: &serenity::FullEvent,
) -> Result<(), Error> {
    let ctx = framework.serenity_context;
    // let data = framework.user_data;

    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            info!("🤖 {} is Online", data_about_bot.user.name.to_string())
        }
        serenity::FullEvent::Message { new_message } => {
            // TODO: Add some kind of verification here to check SERVER_ID pref
            on_message(&ctx, new_message).await?;
        }
        serenity::FullEvent::GuildCreate { guild, .. } => {
            let _config = ServerConfig::get_or_default(guild.id.get()).await?;
        }
        _ => {}
    }
    Ok(())
}

async fn on_message(ctx: &serenity::Context, message: &serenity::Message) -> Result<(), Error> {
    debug!("Message received from user: {}", message.author.name);
    debug!("Channel type: {:?}", message.channel_id);

    // Exits if message author is from bot itself
    if message.author.id == ctx.cache.current_user().id {
        debug!("Skipping message from self");
        return Ok(());
    }

    let server_config = if let Some(guild_id) = message.guild_id {
        Some(ServerConfig::get_or_default(guild_id.get()).await?)
    } else {
        None
    };

    if let Some(config) = &server_config {
        match config.sanitizer_mode {
            crate::handlers::db::SanitizerMode::ManualMention => {}
            _ => {}
        }
    }

    let input = message.content.trim();
    if !input.to_lowercase().contains("http") {
        debug!("No URL found in message");
        return Ok(());
    }

    debug!("URL found, processing input: {}", input);

    let response = match sanitize_input(input).await {
        None => return Ok(()), // Exit early if no match
        Some(response) => response,
    };

    let bot_message = message.reply(ctx, response).await?;

    if let Some(config) = server_config {
        if config.hide_original_embed && message.guild_id.is_some() {
            message
                .channel_id
                .edit_message(
                    ctx,
                    message.id,
                    serenity::EditMessage::new().suppress_embeds(true),
                )
                .await?;
        }
    }

    // message
    //     .react(
    //         ctx,
    //         serenity::ReactionType::Custom {
    //             animated: false,
    //             id: EmojiId::new(1206376642042138724),
    //             name: Some("Sanitized".to_string()),
    //         },
    //     )
    //     .await?;

    handle_response_event(ctx, message, &bot_message).await?;

    Ok(())
}
