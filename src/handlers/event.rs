use std::sync::LazyLock;

use anyhow::Error;
use poise::serenity_prelude::{self as serenity, EmojiId};
use tracing::{debug, error, info};

use crate::Data;
use crate::handlers::{handle_response_event, sanitize_input};

use super::db::{DeletePermission, SanitizerMode, ServerConfig};

static SANITIZER_EMOJI: LazyLock<serenity::ReactionType> =
    LazyLock::new(|| serenity::ReactionType::Custom {
        animated: false,
        id: EmojiId::new(1206376642042138724),
        name: Some("Sanitized".to_string()),
    });

pub async fn get_event_handler(
    framework: poise::FrameworkContext<'_, Data, Error>,
    event: &serenity::FullEvent,
) -> Result<(), Error> {
    let ctx = framework.serenity_context;
    // let data = framework.user_data;

    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            info!("🤖 {} is Online", data_about_bot.user.name.to_string());
            ctx.set_presence(
                Some(serenity::ActivityData::watching("for embeds")),
                serenity::OnlineStatus::Online,
            );
        }
        serenity::FullEvent::Message { new_message } => {
            // TODO: Add some kind of verification here to check SERVER_ID pref
            on_message(&ctx, new_message).await?;
        }
        serenity::FullEvent::ReactionAdd { add_reaction } => {
            on_reaction_add(&ctx, add_reaction).await?;
        }
        serenity::FullEvent::GuildCreate { guild, .. } => {
            match ServerConfig::get_or_default(guild.id.get()).await {
                Ok(_) => {}
                Err(e) => {
                    error!(
                        "Failed to get server config on guild_create for {}: {:?}",
                        guild.id.get(), e
                    );
                }
            }
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
    // Exits if message author is from bot itself
    if message.author.id == ctx.cache.current_user().id {
        debug!("Skipping message from self");
        return Ok(());
    }

    debug!("Message received from user: {}", message.author.name);
    debug!("Channel id: {:?}", message.channel_id);

    let server_config = match message.guild_id {
        Some(guild_id) => ServerConfig::get_or_default(guild_id.get()).await?,
        None => ServerConfig::default(0),
    };

    let should_process = match server_config.sanitizer_mode {
        SanitizerMode::Automatic => true,
        SanitizerMode::ManualMention => message.mentions_me(ctx).await?,
        SanitizerMode::ManualEmote => {
            if message.content.trim().to_lowercase().contains("http")
                && crate::handlers::user_input::ParsedURL::new(message.content.trim()).is_some()
            {
                let _ = message.react(ctx, SANITIZER_EMOJI.clone()).await?;
            }
            false
        }
        SanitizerMode::ManualBoth => {
            if message.mentions_me(ctx).await? {
                true
            } else {
                if message.content.trim().to_lowercase().contains("http")
                    && crate::handlers::user_input::ParsedURL::new(message.content.trim()).is_some()
                {
                    let _ = message.react(ctx, SANITIZER_EMOJI.clone()).await?;
                }
                false
            }
        }
    };

    // Exit early
    if !should_process {
        return Ok(());
    }

    let result = process_message(ctx, &message, &server_config, message.guild_id.is_some()).await;

    // If we're in ManualBoth mode and the trigger was a mention, clean up the emoji
    if matches!(server_config.sanitizer_mode, SanitizerMode::ManualBoth) {
        // Remove from the current message (reply)
        match message
            .delete_reaction_emoji(ctx, SANITIZER_EMOJI.clone())
            .await
        {
            Ok(()) => debug!(
                "Removed sanitizer emoji reactions from current message after mention trigger"
            ),
            Err(e) => error!(
                "Failed to remove sanitizer emoji reactions from current message: {}",
                e
            ),
        }

        // Also remove from the referenced/original message if present, to prevent re-triggers
        if let Some(referenced_message) = &message.referenced_message {
            match referenced_message
                .delete_reaction_emoji(ctx, SANITIZER_EMOJI.clone())
                .await
            {
                Ok(()) => debug!(
                    "Removed sanitizer emoji reactions from referenced message after mention trigger"
                ),
                Err(e) => error!(
                    "Failed to remove sanitizer emoji reactions from referenced message: {}",
                    e
                ),
            }
        }
    }

    result
}

async fn on_reaction_add(
    ctx: &serenity::Context,
    reaction: &serenity::Reaction,
) -> Result<(), Error> {
    let bot_user_id = ctx.cache.current_user().id;

    // Skip if the reaction is from the bot
    if let Some(user_id) = reaction.user_id {
        if user_id == bot_user_id {
            debug!("Skipping reaction from self");
            return Ok(());
        }
    }
    // Skip if message is from the bot
    let message = match reaction.message(ctx).await {
        Ok(message) => message,
        Err(e) => {
            error!("Failed to get message from reaction: {}", e);
            return Ok(());
        }
    };

    if message.author.id == bot_user_id {
        debug!("Skipping, reacted message is by self");
        return Ok(());
    }

    let is_correct_emoji = reaction.emoji == *SANITIZER_EMOJI;

    if !is_correct_emoji {
        debug!("Reaction is not the sanitizer emoji, skipping");
        return Ok(());
    }

    debug!("Sanitizer emoji detected");

    let server_config = match reaction.guild_id {
        Some(guild_id) => ServerConfig::get_or_default(guild_id.get()).await?,
        None => {
            // I don't think this case ever happens, but just to be safe
            debug!("Reaction in DM, skipping");
            return Ok(());
        }
    };

    match server_config.sanitizer_mode {
        SanitizerMode::ManualEmote | SanitizerMode::ManualBoth => {
            debug!("Emote mode enabled, processing message");
            let result =
                process_message(ctx, &message, &server_config, reaction.guild_id.is_some()).await;

            // Remove all occurrences of the sanitizer emoji from the original message
            // (both the bot's and users') to prevent repeated triggers
            match message
                .delete_reaction_emoji(ctx, SANITIZER_EMOJI.clone())
                .await
            {
                Ok(()) => debug!(
                    "Removed sanitizer emoji reactions to prevent repeated triggers"
                ),
                Err(e) => error!("Failed to remove sanitizer emoji reactions: {}", e),
            }

            result
        }
        _ => {
            debug!("Manual emote not enabled, exiting");
            return Ok(());
        }
    }
}

async fn process_message(
    ctx: &serenity::Context,
    message: &serenity::Message,
    server_config: &ServerConfig,
    is_guild_context: bool,
) -> Result<(), Error> {
    debug!("process_message called:");
    debug!("message.id: {}", message.id);
    debug!("message.author: {}", message.author.name);
    debug!(
        "server_config.sanitizer_mode: {:?}",
        server_config.sanitizer_mode
    );
    debug!(
        "server_config.hide_original_embed: {}",
        server_config.hide_original_embed
    );

    let (input, message_to_suppress) = match server_config.sanitizer_mode {
        SanitizerMode::ManualMention | SanitizerMode::ManualBoth => {
            if message.content.trim().to_lowercase().contains("http") {
                debug!("Using message content as input");
                (message.content.trim(), message) // Return message with mention + url
            } else if let Some(referenced_message) = &message.referenced_message {
                if referenced_message
                    .content
                    .trim()
                    .to_lowercase()
                    .contains("http")
                {
                    debug!("Using referenced message content as input");
                    (
                        referenced_message.content.trim(),
                        referenced_message.as_ref(),
                    )
                } else {
                    debug!("Referenced message does not contain URL, exiting");
                    return Ok(()); // Referenced message does not contain a url, so exit
                }
            } else {
                debug!("No referenced message exists, exiting");
                return Ok(()); // No referenced message exists, so exit
            }
        }

        _ => {
            debug!("Using message content as input (automatic mode)");
            (message.content.trim(), message)
        }
    };

    debug!("URL found, processing input: {}", input);

    let response = match sanitize_input(input).await {
        Some(response) => {
            debug!("sanitize_input returned: {}", response);
            response
        }
        None => {
            debug!("sanitize_input returned None, exiting");
            return Ok(());
        } // Exit early if no match
    };

    // Build reply. In guilds, include a Delete button when enabled in server config
    let bot_message = if is_guild_context {
        let mut msg = serenity::CreateMessage::new()
            .reference_message(message_to_suppress)
            .content(response)
            .allowed_mentions(serenity::CreateAllowedMentions::new());

        if !matches!(server_config.delete_permission, DeletePermission::Disabled) {
            let delete_button = serenity::CreateButton::new("delete_bot_response")
                .label("Delete")
                .style(serenity::ButtonStyle::Danger);
            let components = vec![serenity::CreateActionRow::Buttons(vec![delete_button])];
            msg = msg.components(components);
        }

        message_to_suppress.channel_id.send_message(ctx, msg).await?
    } else {
        let msg = serenity::CreateMessage::new()
            .reference_message(message_to_suppress)
            .content(response)
            .allowed_mentions(serenity::CreateAllowedMentions::new());
        message_to_suppress.channel_id.send_message(ctx, msg).await?
    };
    debug!("Bot replied with message ID: {}", bot_message.id);

    let should_suppress_embeds = server_config.hide_original_embed && is_guild_context;

    debug!(
        "Calling handle_response_event with hide_original_embed: {}",
        server_config.hide_original_embed
    );
    handle_response_event(
        ctx,
        message_to_suppress,
        &bot_message,
        should_suppress_embeds,
    )
    .await?;

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

    // Early handle for Delete button interactions (no Manage Server required)
    if interaction.data.custom_id == "delete_bot_response" {
        use super::db::DeletePermission as DP;
        let config = ServerConfig::get_or_default(guild_id).await?;

        // Determine if the clicker is allowed
        let member = interaction.member.as_ref().unwrap();
        let permissions = member.permissions.unwrap_or_default();

        // Original author ID if the bot's message is a reply to a user's message
        let original_author_id = interaction
            .message
            .referenced_message
            .as_ref()
            .map(|m| m.author.id);

        let user_id = interaction.user.id;

        let is_moderator = permissions.manage_messages() || permissions.administrator();
        let is_author = original_author_id.map(|id| id == user_id).unwrap_or(false);

        let allowed = match config.delete_permission {
            DP::Disabled => false,
            DP::Everyone => true,
            DP::AuthorAndMods => is_author || is_moderator,
        };

        if !allowed {
            interaction
                .create_response(
                    ctx,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content("❌ You don't have permission to delete this.")
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }

        // Try to delete the bot's message (the message containing the button)
        match interaction.message.delete(ctx).await {
            Ok(()) => {
                interaction
                    .create_response(
                        ctx,
                        serenity::CreateInteractionResponse::Message(
                            serenity::CreateInteractionResponseMessage::new()
                                .content("🗑️ Deleted.")
                                .ephemeral(true),
                        ),
                    )
                    .await?;

                // Attempt to restore original embeds on the referenced message (if we had suppressed)
                if config.hide_original_embed {
                    if let Some(orig) = interaction.message.referenced_message.as_ref() {
                        match orig
                            .channel_id
                            .edit_message(
                                ctx,
                                orig.id,
                                serenity::EditMessage::new().suppress_embeds(false),
                            )
                            .await
                        {
                            Ok(_) => debug!(
                                "Restored original embeds for message {} after deletion",
                                orig.id
                            ),
                            Err(e) => error!(
                                "Failed to restore original embeds for message {}: {}",
                                orig.id, e
                            ),
                        }
                    } else if let Some(msg_ref) = interaction.message.message_reference.as_ref() {
                        if let Some(orig_id) = msg_ref.message_id {
                            let channel_id = msg_ref.channel_id;
                            match channel_id
                                .edit_message(
                                    ctx,
                                    orig_id,
                                    serenity::EditMessage::new().suppress_embeds(false),
                                )
                                .await
                            {
                                Ok(_) => debug!(
                                    "Restored original embeds for message {} (via reference) after deletion",
                                    orig_id
                                ),
                                Err(e) => error!(
                                    "Failed to restore original embeds for message {} (via reference): {}",
                                    orig_id, e
                                ),
                            }
                        } else {
                            debug!(
                                "message_reference present but missing message_id; cannot restore embeds"
                            );
                        }
                    } else {
                        debug!("No referenced message; cannot restore original embeds");
                    }
                }
            }
            Err(e) => {
                error!("Failed to delete bot message: {}", e);
                interaction
                    .create_response(
                        ctx,
                        serenity::CreateInteractionResponse::Message(
                            serenity::CreateInteractionResponseMessage::new()
                                .content("❌ Failed to delete message. Try again.")
                                .ephemeral(true),
                        ),
                    )
                    .await?;
            }
        }

        return Ok(());
    }

    // Check if user has manage guild permissions for config interactions
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

            return Ok(());
        }
    }

    Ok(())
}
