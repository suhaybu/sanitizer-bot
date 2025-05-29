use anyhow::Error;
use poise::serenity_prelude as serenity;
use tracing::{debug, error, info};

use crate::Data;
use crate::handlers::{handle_response_event, sanitize_input};

use super::db::{DeletePermission, SanitizerMode, ServerConfig};

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
        serenity::FullEvent::InteractionCreate { interaction } => {
            if let serenity::Interaction::Component(component) = interaction {
                handle_component_interaction(&ctx, component).await?;
            }
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

async fn handle_component_interaction(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
) -> Result<(), Error> {
    debug!(
        "Component interaction received: {}",
        interaction.data.custom_id
    );

    let guild_id = match interaction.guild_id {
        Some(id) => id.get(),
        None => {
            interaction
                .create_response(
                    ctx,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content("❌ This can only be used in servers!")
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }
    };

    // Check if user has manage guild permissions
    let member = interaction.member.as_ref().unwrap();
    let permissions = member.permissions.unwrap_or_default();
    if !permissions.manage_guild() {
        interaction
            .create_response(
                ctx,
                serenity::CreateInteractionResponse::Message(
                    serenity::CreateInteractionResponseMessage::new()
                        .content("❌ You need the 'Manage Server' permission to use this!")
                        .ephemeral(true),
                ),
            )
            .await?;
        return Ok(());
    }

    // Get current config
    let mut config = ServerConfig::get_or_default(guild_id).await?;

    // Handle different select menu types
    let response_message = match interaction.data.custom_id.as_str() {
        "sanitizer_mode" => {
            if let serenity::ComponentInteractionDataKind::StringSelect { values } =
                &interaction.data.kind
            {
                if let Some(value) = values.first() {
                    config.sanitizer_mode = match value.as_str() {
                        "automatic" => SanitizerMode::Automatic,
                        "manual_emote" => SanitizerMode::ManualEmote,
                        "manual_mention" => SanitizerMode::ManualMention,
                        "manual_both" => SanitizerMode::ManualBoth,
                        _ => SanitizerMode::Automatic,
                    };

                    let mode_text = match config.sanitizer_mode {
                        SanitizerMode::Automatic => "Automatic",
                        SanitizerMode::ManualEmote => "Manual (Emote)",
                        SanitizerMode::ManualMention => "Manual (Mention)",
                        SanitizerMode::ManualBoth => "Manual (Emote and Mention)",
                    };

                    format!("✅ Switched to {} mode", mode_text)
                } else {
                    "❌ No option selected".to_string()
                }
            } else {
                "❌ Invalid interaction data".to_string()
            }
        }
        "delete_permission" => {
            if let serenity::ComponentInteractionDataKind::StringSelect { values } =
                &interaction.data.kind
            {
                if let Some(value) = values.first() {
                    config.delete_permission = match value.as_str() {
                        "author_and_mods" => DeletePermission::AuthorAndMods,
                        "everyone" => DeletePermission::Everyone,
                        "disabled" => DeletePermission::Disabled,
                        _ => DeletePermission::AuthorAndMods,
                    };

                    let perm_text = match config.delete_permission {
                        DeletePermission::AuthorAndMods => "default (Author and moderators)",
                        DeletePermission::Everyone => "everyone",
                        DeletePermission::Disabled => "disabled",
                    };

                    format!("✅ Set delete button permissions to {}", perm_text)
                } else {
                    "❌ No option selected".to_string()
                }
            } else {
                "❌ Invalid interaction data".to_string()
            }
        }
        "hide_original_embed" => {
            if let serenity::ComponentInteractionDataKind::StringSelect { values } =
                &interaction.data.kind
            {
                if let Some(value) = values.first() {
                    config.hide_original_embed = value == "hide";

                    if config.hide_original_embed {
                        "✅ Enabled hiding original message's embed".to_string()
                    } else {
                        "✅ Disabled hiding original message's embed".to_string()
                    }
                } else {
                    "❌ No option selected".to_string()
                }
            } else {
                "❌ Invalid interaction data".to_string()
            }
        }
        _ => {
            debug!(
                "Unknown component interaction: {}",
                interaction.data.custom_id
            );
            return Ok(());
        }
    };

    match config.save().await {
        Ok(()) => {
            // Send response
            interaction
                .create_response(
                    ctx,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(response_message)
                            .ephemeral(true),
                    ),
                )
                .await?;

            if let Err(e) = super::db::sync_database().await {
                error!("Failed to sync database after config save: {:?}", e);
            } else {
                debug!(
                    "Database synced successfully after config change for guild {}",
                    guild_id
                );
            }
        }
        Err(e) => {
            error!("Failed to save server config: {:?}", e);

            // Save the updated config
            if let Err(e) = config.save().await {
                error!("Failed to save server config: {:?}", e);
                interaction
                    .create_response(
                        ctx,
                        serenity::CreateInteractionResponse::Message(
                            serenity::CreateInteractionResponseMessage::new()
                                .content("❌ Failed to save configuration. Please try again.")
                                .ephemeral(true),
                        ),
                    )
                    .await?;
            }
            return Ok(());
        }
    }

    Ok(())
}
