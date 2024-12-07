use poise::serenity_prelude::EditInteractionResponse;
use poise::serenity_prelude::{self as serenity, CreateEmbed, CreateMessage, EditMessage};
use std::time::Duration;
use tokio::time::sleep;
use tracing::debug;

use crate::Context;
use crate::Result;

// On message (Listener)
pub async fn handle_event_response(
    ctx: &serenity::Context,
    user_message: &serenity::Message,
    bot_message: &serenity::Message,
) -> Result<()> {
    // Wait for embeds to appear (up to 10 seconds)
    let valid_response = tokio::time::timeout(Duration::from_secs(10), async {
        while bot_message.embeds.is_empty() {
            sleep(Duration::from_secs(1)).await;
        }
        check_bot_response(bot_message)
    })
    .await
    .unwrap_or(false);

    match (valid_response, user_message.guild_id.is_some()) {
        (true, true) => {
            debug!("Valid response in guild, suppressing embeds");
            user_message
                .channel_id
                .edit_message(
                    ctx,
                    user_message.id,
                    EditMessage::new().suppress_embeds(true),
                )
                .await?;
        }
        (true, false) => {
            debug!("Valid response in DM, skipping embed suppression");
        }
        (false, _) => {
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
                        .add_embed(error_embed), // .allowed_mentions(CreateAllowedMentions::new()),
                )
                .await?;

            sleep(Duration::from_secs(10)).await;
            error_message.delete(ctx).await?;
        }
    }

    Ok(())
}

// On interaction (Command invokation)
pub async fn handle_interaction_response(
    ctx: &Context<'_>,
    bot_message: &serenity::Message,
) -> Result<()> {
    debug!(
        "Starting interaction response handler for message ID: {}",
        bot_message.id
    );
    debug!("Initial embed count: {}", bot_message.embeds.len());
    debug!("Message content: {}", bot_message.content);

    let valid_response = tokio::time::timeout(Duration::from_secs(10), async {
        debug!("Entering timeout block, waiting for embeds");
        while bot_message.embeds.is_empty() {
            debug!("No embeds yet, sleeping...");
            sleep(Duration::from_secs(1)).await;
            debug!("Current embed count: {}", bot_message.embeds.len());
        }
        debug!("Embeds found, checking response validity");
        check_bot_response(bot_message)
    })
    .await
    .unwrap_or(false);

    debug!("Response validity check completed: {}", valid_response);

    if !valid_response {
        if ctx.guild_id().is_some() {
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

// Logic used to validate if response is true
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
            true
        }
    }
}
