//! Core handler for all types of events

use std::sync::Arc;

use anyhow::{Context, Ok};
use twilight_gateway::Event;
use twilight_http::Client;
use twilight_model::application::interaction::application_command::CommandData;
use twilight_model::application::interaction::message_component::MessageComponentInteractionData;
use twilight_model::application::interaction::{Interaction, InteractionData};
use twilight_model::channel::Message;
use twilight_model::channel::message::{Component, MessageFlags, MessageType};
use twilight_model::gateway::GatewayReaction;
use twilight_model::guild::Permissions;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_util::builder::InteractionResponseDataBuilder;
use twilight_util::builder::message::{ContainerBuilder, TextDisplayBuilder};

use crate::discord::commands;
use crate::discord::models::{DeletePermission, SanitizerMode, SettingsMenuType};
use crate::utils::database::ServerConfig;
use crate::utils::sanitize;

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
            // Early exit if reaction is by bot or not in a guild.
            if crate::BOT_USER_ID
                .get()
                .is_some_and(|&bot_id| bot_id == ctx.0.user_id)
                || ctx.0.guild_id.is_none()
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
        anyhow::bail!("ReactionAdd is not in a guild.")
    };

    let reaction_emoji_id = match &reaction.emoji {
        twilight_model::channel::message::EmojiReactionType::Custom { id, .. } => *id,
        _ => return Ok(()),
    };

    if reaction_emoji_id == crate::EMOJI_ID {
        let server_config = ServerConfig::get_or_default(guild_id.get()).await?;
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
    let server_config = ServerConfig::get_or_default(guild_id.get()).await?;

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
        tracing::debug!("Ignoring interaction with no data");
        return Ok(());
    };

    match data {
        // Handles command invocations
        InteractionData::ApplicationCommand(data) => {
            tracing::debug!("Recieved ApplicationCommand event with name: {}", data.name);
            handle_app_command(data.name.as_str(), &interaction, client, &data).await
        }
        // Handles component invocations
        InteractionData::MessageComponent(data) => {
            tracing::debug!(
                "Recieved MessageComponent event with custom_id: {}",
                data.custom_id
            );
            handle_component(&data, &interaction, client).await
        }
        _ => {
            tracing::debug!("Ignoring unknown interaction type");

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
    match data.custom_id.as_str() {
        "delete" => handle_delete_button(interaction, client).await,
        _ => {
            let menu_type = data
                .custom_id
                .parse::<SettingsMenuType>()
                .with_context(|| format!("Unknown component: {}", data.custom_id))?;
            commands::SettingsCommand::handle_component(interaction, menu_type, data, client).await
        }
    }
}

async fn handle_delete_button(interaction: &Interaction, client: &Client) -> anyhow::Result<()> {
    let Some(ref bot_message) = interaction.message else {
        tracing::debug!("Delete button pressed but no message found");
        return Ok(());
    };

    let user_message = client
        .message(bot_message.channel_id, bot_message.id)
        .await?
        .model()
        .await?;

    // Early exit should not happen. Delete button is only to be created in guild context.
    let Some(guild_id) = interaction.guild_id else {
        anyhow::bail!("Interaction missing guild_id")
    };
    let server_config = ServerConfig::get_or_default(guild_id.get()).await?;

    if server_config.delete_permission == DeletePermission::Disabled {
        tracing::debug!("Early exit: Delete button is disabled in server config");
        return Ok(());
    }

    let interaction_user_id = interaction
        .member
        .as_ref()
        .and_then(|member| member.user.as_ref().map(|user| user.id))
        .or_else(|| interaction.user.as_ref().map(|user| user.id))
        .ok_or_else(|| anyhow::anyhow!("User ID could not be found in interaction."))?;

    let user_has_permission = match server_config.delete_permission {
        DeletePermission::Everyone => true,
        DeletePermission::Disabled => false,
        DeletePermission::AuthorAndMods => {
            let author_id = user_message
                .referenced_message
                .as_ref()
                .map(|msg| msg.author.id);

            let is_author = author_id.map_or(false, |author_id| author_id == interaction_user_id);

            let has_manage_message = interaction
                .member
                .as_ref()
                .and_then(|m| m.permissions)
                .map_or(false, |perms| perms.contains(Permissions::MANAGE_MESSAGES));

            is_author || has_manage_message
        }
    };

    // Early exit and a message to the user if user has insufficient permissions.
    if !user_has_permission {
        tracing::debug!(
            "User {} does not have permission to delete this message.",
            interaction_user_id
        );

        let container = ContainerBuilder::new()
            .spoiler(false)
            .component(
                TextDisplayBuilder::new("You do not have permission to delete this message")
                    .build(),
            )
            .build();
        let data = InteractionResponseDataBuilder::new()
            .components([Component::Container(container)])
            .flags(MessageFlags::IS_COMPONENTS_V2 | MessageFlags::EPHEMERAL)
            .build();
        let response = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(data),
        };

        client
            .interaction(interaction.application_id)
            .create_response(interaction.id, &interaction.token, &response)
            .await?;

        return Ok(());
    };

    // Delete bot's response.
    client
        .delete_message(bot_message.channel_id, bot_message.id)
        .await?;
    tracing::debug!(
        "Deleted bot message {} in channel {}",
        bot_message.id,
        bot_message.channel_id
    );

    if server_config.hide_original_embed {
        let referenced_message = user_message
            .referenced_message
            .as_ref()
            .context("No referenced message found to unsuppress")?;

        if let Err(e) = client
            .update_message(referenced_message.channel_id, referenced_message.id)
            .flags(MessageFlags::empty())
            .await
        {
            tracing::warn!("Failed to unsuppress original message embed: {:?}", e);
        }
    }

    Ok(())
}
