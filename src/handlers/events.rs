//! Core handler for all types of events

use std::sync::Arc;

use anyhow::{Context, Ok};
use tracing::debug;
use twilight_gateway::Event;
use twilight_http::Client;
use twilight_model::application::interaction::application_command::CommandData;
use twilight_model::application::interaction::message_component::MessageComponentInteractionData;
use twilight_model::application::interaction::{Interaction, InteractionData};
use twilight_model::channel::Message;
use twilight_model::channel::message::MessageType;
use twilight_model::gateway::GatewayReaction;

use crate::commands;
use crate::database::ServerConfig;
use crate::models::{SanitizerMode, SettingsMenuType};
use crate::sanitize;

/// Handles all types of incomming events from Discord.
pub async fn handle_event(event: Event, client: Arc<Client>) {
    match event {
        Event::InteractionCreate(ctx) => {
            if let Err(error) = handle_interaction(ctx.0, &client).await {
                tracing::error!(?error, "Failed to handle Event::InteractionCreate");
            }
        }
        Event::MessageCreate(ctx) => {
            // Early exit if message is by bot, or has content with no url, or is a reply.
            if crate::BOT_USER_ID
                .get()
                .is_some_and(|&bot_id| ctx.0.author.id == bot_id)
                || (!sanitize::contains_url(&ctx.0.content) && !(ctx.0.kind == MessageType::Reply))
            {
                return;
            }

            if let Err(error) = handle_on_message(ctx.0, &client).await {
                tracing::error!(?error, "Failed to handle Event::MessageCreate")
            }
        }
        Event::ReactionAdd(ctx) => {
            // Early exit if reaction is by bot.
            if crate::BOT_USER_ID
                .get()
                .is_some_and(|&bot_id| bot_id == ctx.0.user_id)
            {
                return;
            }

            if let Err(error) = handle_reaction_add(ctx.0, &client).await {
                tracing::error!(?error, "Failed to handle Event::ReactionAdd");
            }
        }
        _ => (),
    }
}

/// Handles twilight_gateway::Event::ReactionAdd events.
async fn handle_reaction_add(reaction: GatewayReaction, client: &Client) -> anyhow::Result<()> {
    // Exits early if reaction is not in a guild. This exit should never happen.
    let Some(guild_id) = reaction.guild_id else {
        return Ok(());
    };

    let reaction_emoji_id = match &reaction.emoji {
        twilight_model::channel::message::EmojiReactionType::Custom { id, .. } => *id,
        _ => return Ok(()),
    };

    if reaction_emoji_id == crate::EMOJI_ID {
        let guild_id: u64 = guild_id.into();
        let server_config = ServerConfig::get_or_default(guild_id).await?;
        let message = client
            .message(reaction.channel_id, reaction.message_id)
            .await?
            .model()
            .await?;

        sanitize::process_message(&message, &client, Some(server_config)).await?;
    }

    Ok(())
}

/// Handles twilight_gateway::Event::MessageCreate events.
async fn handle_on_message(message: Message, client: &Client) -> anyhow::Result<()> {
    // Retrieves guild_id, else early exits.
    let Some(guild_id) = message.guild_id else {
        sanitize::process_message(&message, client, None).await?;
        return Ok(());
    };
    let guild_id: u64 = guild_id.into();
    let server_config = ServerConfig::get_or_default(guild_id).await?;

    match server_config.sanitizer_mode {
        SanitizerMode::Automatic => {
            sanitize::process_message(&message, client, Some(server_config)).await?;
        }
        SanitizerMode::ManualEmote => {
            sanitize::add_emote(&message, client).await?;
        }
        SanitizerMode::ManualMention => {
            // Early exit if not mentioned.
            if !sanitize::is_bot_mentioned(&message) && message.kind != MessageType::Reply {
                return Ok(());
            }
            sanitize::process_message(&message, client, Some(server_config)).await?;
        }
        SanitizerMode::ManualBoth => {
            sanitize::add_emote(&message, client).await?;
            if !sanitize::is_bot_mentioned(&message) && message.kind != MessageType::Reply {
                return Ok(());
            }
            sanitize::process_message(&message, client, Some(server_config)).await?;
        }
    }

    Ok(())
}

/// Handles twilight_gateway::Event::InteractionCreate events.
async fn handle_interaction(mut interaction: Interaction, client: &Client) -> anyhow::Result<()> {
    let Some(data) = interaction.data.take() else {
        debug!("Ignoring interaction with no data");
        return Ok(());
    };

    match data {
        // Handles command invocations
        InteractionData::ApplicationCommand(data) => {
            debug!("Recieved ApplicationCommand event with name: {}", data.name);
            handle_app_command(data.name.as_str(), &interaction, client, &data).await
        }
        // Handles component invocations
        InteractionData::MessageComponent(data) => {
            debug!(
                "Recieved MessageComponent event with custom_id: {}",
                data.custom_id
            );
            handle_component(&data, &interaction, client).await
        }
        _ => {
            debug!("Ignoring unknown interaction type");

            Ok(())
        }
    }
}

/// Matches command name from event data to call the respective command handler
async fn handle_app_command(
    command_name: &str,
    interaction: &Interaction,
    client: &Client,
    data: &CommandData,
) -> anyhow::Result<()> {
    match command_name {
        "credits" => commands::CreditsCommand::handle(interaction, client).await,
        "settings" => commands::SettingsCommand::handle(interaction, client).await,
        "Sanitize" | "sanitize" => {
            commands::SanitizeCommand::handle(interaction, client, data).await
        }
        unknown_name => anyhow::bail!("unknown command: {}", unknown_name),
    }
}

/// Matches the component id from event data to call the respective component handler
async fn handle_component(
    data: &MessageComponentInteractionData,
    interaction: &Interaction,
    client: &Client,
) -> anyhow::Result<()> {
    let menu_type = data
        .custom_id
        .parse::<SettingsMenuType>()
        .with_context(|| format!("Unknown component: {}", data.custom_id))?;
    commands::SettingsCommand::handle_component(interaction, menu_type, data, client).await
}
