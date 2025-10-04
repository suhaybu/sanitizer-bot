use anyhow::Ok;
use twilight_http::{Client, request::channel::reaction::RequestReactionType};
use twilight_model::channel::{
    Message,
    message::{
        AllowedMentions, Component, EmojiReactionType, MessageFlags, MessageType,
        component::{ActionRow, Button, ButtonStyle},
    },
};

use crate::{
    BOT_USER_ID,
    database::ServerConfig,
    models::{DeletePermission, SanitizerMode},
    sanitize::{UrlProcessor, core::Platform},
};

/// Converts the URL in a message if there is a valid URL.
pub async fn process_message(
    message: &Message,
    client: &Client,
    server_config: Option<ServerConfig>,
) -> anyhow::Result<()> {
    let message = match Platform::try_detect(&message.content) {
        Some(_) => message,
        None => match server_config.as_ref() {
            Some(config)
                if (config.sanitizer_mode == SanitizerMode::ManualMention
                    || config.sanitizer_mode == SanitizerMode::ManualBoth)
                    && message.kind == MessageType::Reply =>
            {
                match message.referenced_message.as_deref() {
                    Some(ref_msg) => ref_msg,
                    None => return Ok(()),
                }
            }
            _ => return Ok(()),
        },
    };

    let url = match UrlProcessor::try_new(&message.content) {
        Some(url) => url,
        None => return Ok(()),
    };

    let output = url
        .capture_url()
        .and_then(|captures| captures.format_output())
        .ok_or_else(|| anyhow::anyhow!("Failed to process URL"))?;

    let response = client
        .create_message(message.channel_id)
        .content(&output)
        .flags(MessageFlags::SUPPRESS_NOTIFICATIONS)
        .reply(message.id)
        .allowed_mentions(Some(&AllowedMentions::default()))
        .await?
        .model() // Converts Response<Message> -> Message
        .await?;

    // Early exits if message is not in a server
    let Some(server_config) = server_config else {
        return Ok(());
    };

    // Removes all Sanitized emoji reactions after responding.
    if server_config.sanitizer_mode == SanitizerMode::ManualEmote
        || server_config.sanitizer_mode == SanitizerMode::ManualBoth
    {
        tracing::debug!("Removing all Sanitized emoji reactions.");
        client
            .delete_all_reaction(
                message.channel_id,
                message.id,
                &RequestReactionType::Custom {
                    id: crate::EMOJI_ID,
                    name: Some("Sanitized"),
                },
            )
            .await?;
    }

    // Adds a delete button if config allows it.
    if server_config.delete_permission != DeletePermission::Disabled {
        let delete_emoji = EmojiReactionType::Unicode {
            name: "🗑️".to_string(),
        };

        let components = Component::ActionRow(ActionRow {
            id: None,
            components: Vec::from([Component::Button(Button {
                id: None,
                custom_id: Some("delete".to_owned()),
                disabled: false,
                emoji: Some(delete_emoji),
                label: Some("Delete".to_owned()),
                style: ButtonStyle::Danger,
                url: None,
                sku_id: None,
            })]),
        });

        client
            .update_message(response.channel_id, response.id)
            .components(Some(&[components]))
            .await?;
    }

    if server_config.hide_original_embed {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        client
            .update_message(message.channel_id, message.id)
            .flags(MessageFlags::SUPPRESS_EMBEDS)
            .await?;
    }

    Ok(())
}

/// Adds an emote to a valid message in the Sanitizer::ManualEmote/Both mode.
pub async fn add_emote(message: &Message, client: &Client) -> anyhow::Result<()> {
    // Exits early if URL is not valid
    if UrlProcessor::try_new(&message.content).is_none() {
        tracing::debug!("No valid URL found in message");
        return Ok(());
    };

    client
        .create_reaction(
            message.channel_id,
            message.id,
            &RequestReactionType::Custom {
                id: crate::EMOJI_ID,
                name: Some("Sanitized"),
            },
        )
        .await?;

    Ok(())
}

pub fn is_bot_mentioned(message: &Message) -> bool {
    let bot_user_id = BOT_USER_ID.get().expect("BOT_USER_ID not initialized");

    message
        .mentions
        .iter()
        .any(|mention| &mention.id == bot_user_id)
}
