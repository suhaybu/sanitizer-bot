use std::sync::LazyLock;

use tracing::{debug, error};
use twilight_http::Client;
use twilight_model::channel::Message;
use twilight_model::channel::message::ReactionType;
use twilight_model::gateway::payload::incoming::{MessageCreate, ReactionAdd};
use twilight_model::id::{marker::{EmojiMarker, UserMarker}, Id};

use crate::handlers::response::wait_for_message_embed;
use crate::handlers::sanitize_input;

use super::db::{SanitizerMode, ServerConfig};

static SANITIZER_EMOJI_ID: LazyLock<Id<EmojiMarker>> = LazyLock::new(|| Id::new(1206376642042138724));

pub async fn on_message_twilight(event: MessageCreate, client: std::sync::Arc<Client>, bot_user_id: Id<UserMarker>) {
    let message = event.0;
    if message.author.bot { return; }

    debug!("Message received from user: {}", message.author.name);
    debug!("Channel id: {:?}", message.channel_id);

    let server_config = match message.guild_id {
        Some(guild_id) => ServerConfig::get_or_default(guild_id.get()).await.unwrap_or(ServerConfig::default(0)),
        None => ServerConfig::default(0),
    };

    let mentions_bot = message.mentions.iter().any(|u| u.id == bot_user_id);

    let should_process = match server_config.sanitizer_mode {
        SanitizerMode::Automatic => true,
        SanitizerMode::ManualMention => mentions_bot,
        SanitizerMode::ManualEmote => false,
        SanitizerMode::ManualBoth => mentions_bot,
    };

    if !should_process { return; }

    if let Err(e) = process_message_twilight(&message, &server_config, client.clone()).await {
        error!(?e, "failed to process message");
    }
}

pub async fn on_reaction_add_twilight(event: ReactionAdd, client: std::sync::Arc<Client>) {
    let reaction = event.0;
    if reaction.guild_id.is_none() { return; }

    if matches!(&reaction.emoji, ReactionType::Custom { id, .. } if id == &*SANITIZER_EMOJI_ID) {
        // Load message
        if let Ok(resp) = client.message(reaction.channel_id, reaction.message_id).await {
            if let Ok(msg) = resp.model().await {
            let server_config = ServerConfig::get_or_default(reaction.guild_id.unwrap().get())
                .await
                .unwrap_or(ServerConfig::default(0));
            if matches!(server_config.sanitizer_mode, SanitizerMode::ManualEmote | SanitizerMode::ManualBoth) {
                if let Err(e) = process_message_twilight(&msg, &server_config, client.clone()).await {
                    error!(?e, "failed to process reaction message");
                }
            }
            }
        }
    }
}

async fn process_message_twilight(
    message: &Message,
    server_config: &ServerConfig,
    client: std::sync::Arc<Client>,
) -> anyhow::Result<()> {
    debug!("process_message called:");
    debug!("message.id: {}", message.id);
    debug!("message.author: {}", message.author.name);
    debug!("server_config.sanitizer_mode: {:?}", server_config.sanitizer_mode);
    debug!("server_config.hide_original_embed: {}", server_config.hide_original_embed);

    let (input, message_to_suppress_id, channel_id) = if message.content.trim().to_lowercase().contains("http") {
        (message.content.trim(), message.id, message.channel_id)
    } else if let Some(referenced) = message.referenced_message.as_ref() {
        if referenced.content.trim().to_lowercase().contains("http") {
            (referenced.content.trim(), referenced.id, message.channel_id)
        } else {
            return Ok(());
        }
    } else {
        return Ok(());
    };

    debug!("URL found, processing input: {}", input);
    let response = match sanitize_input(input).await {
        Some(r) => r,
        None => return Ok(()),
    };

    let created = client
        .create_message(channel_id)
        .content(&response)?
        .await?
        .model()
        .await?;
    debug!("Bot replied with message ID: {}", created.id);

    if server_config.hide_original_embed && message.guild_id.is_some() {
        if let Some(msg) = wait_for_message_embed(&client, created.id, created.channel_id).await {
            if !check_message_valid(&msg) {
                // delete invalid
                let _ = client.delete_message(msg.channel_id, msg.id).await;
            } else {
                // suppress embeds
                let _ = client
                    .update_message(channel_id, message_to_suppress_id)
                    .flags(twilight_model::channel::message::MessageFlags::SUPPRESS_EMBEDS)
                    .await;
            }
        }
    }

    Ok(())
}

fn check_message_valid(msg: &Message) -> bool {
    if msg.embeds.is_empty() { return false; }
    let e = &msg.embeds[0];
    if e.video.is_some() { return true; }
    true
}
