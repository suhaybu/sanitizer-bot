use poise::serenity_prelude::CreateAllowedMentions;
use poise::serenity_prelude::{
    self as serenity, CreateEmbed, CreateMessage, EditInteractionResponse, EditMessage,
};
use std::time::Duration;
use tokio::time::sleep;
use tracing::debug;

use crate::Context;
use crate::Result;

// On message (Listener)
pub async fn handle_response_event(
    ctx: &serenity::Context,
    user_message: &serenity::Message,
    bot_message: &serenity::Message,
    supress_embed: bool,
) -> Result<()> {
    debug!("handle_response_event called:");
    debug!("  user_message.id: {}", user_message.id);
    debug!("  bot_message.id: {}", bot_message.id);
    debug!("  supress_embed: {}", supress_embed);
    debug!("  user_message.guild_id: {:?}", user_message.guild_id);

    // Wait for embeds to appear (up to 10 seconds)
    let valid_response = wait_for_embed(&ctx, bot_message.id, bot_message.channel_id)
        .await
        .map(|msg| check_bot_response(&msg))
        .unwrap_or(false);

    debug!("valid_response: {}", valid_response);

    debug!("Final supress_embed: {}", supress_embed);

    match (valid_response, supress_embed) {
        (true, true) => {
            debug!("Valid response in guild, suppressing embeds");
            user_message
                .channel_id
                .edit_message(
                    &ctx,
                    user_message.id,
                    EditMessage::new().suppress_embeds(true),
                )
                .await?;
        }
        (true, false) => {
            debug!("Valid response, skipping embed suppression due to config or DM");
        }
        (false, _) => {
            debug!("Invalid response, deleting bot message and showing error");
            bot_message.delete(ctx).await?;

            let error_embed = CreateEmbed::new()
                .title("Sorry   ꒰ ꒡⌓꒡꒱")
                .description("Something went wrong.")
                .color(0xd1001f);

            let error_message = user_message
                .channel_id
                .send_message(
                    ctx,
                    CreateMessage::new()
                        .reference_message(user_message)
                        .add_embed(error_embed)
                        .allowed_mentions(CreateAllowedMentions::new()), // .allowed_mentions(CreateAllowedMentions::new()),
                )
                .await?;

            sleep(Duration::from_secs(10)).await;
            error_message.delete(ctx).await?;
        }
    }

    Ok(())
}

// On interaction (Command invocation)
pub async fn handle_response_interaction(
    ctx: &Context<'_>,
    bot_message: &serenity::Message,
) -> Result<()> {
    debug!(
        "Starting interaction response handler for message ID: {}",
        bot_message.id
    );
    debug!("Initial embed count: {}", bot_message.embeds.len());
    debug!("Message content: {}", bot_message.content);

    // Skip validation for private channels
    if ctx.interaction.context == Some(serenity::InteractionContext::PrivateChannel) {
        debug!("Skipping validation for private channel");
        return Ok(());
    }

    let valid_response = wait_for_embed(
        &ctx.serenity_context(),
        bot_message.id,
        bot_message.channel_id,
    )
    .await
    .map(|msg| check_bot_response(&msg))
    .unwrap_or(false);

    debug!("Response validity check completed: {}", valid_response);

    if !valid_response {
        if super::user_input::is_guild_install(&ctx) {
            // Delete the invalid bot message
            bot_message.delete(ctx).await?;
            // Create and send ephemeral error message
            let error_embed = CreateEmbed::new()
                .title("Sorry   ꒰ ꒡⌓꒡꒱")
                .description("Something went wrong.")
                .color(0xd1001f);

            let builder = poise::CreateReply::default()
                .embed(error_embed)
                .ephemeral(true);

            ctx.send(builder).await?;
        } else {
            // TODO: Edits valid response
            ctx.interaction
                .edit_response(
                    ctx,
                    EditInteractionResponse::new().add_embed(
                        CreateEmbed::new()
                            .title("Sorry   ꒰ ꒡⌓꒡꒱")
                            .description("Something went wrong.")
                            .color(0xd1001f),
                    ),
                )
                .await?;
        }
    }

    Ok(())
}

// Logic used to validate if bot's response is valid
fn check_bot_response(bot_message: &serenity::Message) -> bool {
    debug!("Checking bot response for message ID: {}", bot_message.id);

    if bot_message.embeds.is_empty() {
        debug!("No embeds found in message");
        return false;
    }

    let f_embed = bot_message.embeds.first().unwrap();
    debug!("First embed: {:?}", f_embed);

    if f_embed.video.is_some() {
        debug!("Video embed found - valid response");
        return true;
    }

    match &bot_message.content {
        content if content.contains("fxtwitter.com") => {
            let valid = f_embed.description.as_deref() != Some("Sorry, that post doesn't exist :(");
            debug!(
                "Twitter response: valid={}, description={:?}",
                valid, f_embed.description
            );
            valid
        }
        content if content.contains("ddinstagram.com") => {
            let valid = f_embed.description.as_deref() != Some("Post might not be available");
            debug!(
                "Instagram response: valid={}, description={:?}",
                valid, f_embed.description
            );
            valid
        }
        _ => {
            debug!("Unknown platform - defaulting to valid");
            return true;
        }
    }
}

async fn wait_for_embed(
    ctx: &serenity::Context,
    message_id: serenity::MessageId,
    channel_id: serenity::ChannelId,
) -> Option<serenity::Message> {
    if let Ok(msg) = channel_id.message(&ctx.http, message_id).await {
        if !msg.embeds.is_empty() {
            return Some(msg);
        }
    }

    let timeout = Duration::from_secs(8);
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        if let Ok(msg) = channel_id.message(&ctx.http, message_id).await {
            if !msg.embeds.is_empty() {
                return Some(msg);
            }
        }
        sleep(Duration::from_millis(500)).await;
    }

    None
}
