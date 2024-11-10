use ::serenity::all::CreateInteractionResponseFollowup;
use anyhow::Error;
use poise::serenity_prelude::{
    self as serenity, CommandInteraction, CreateEmbed, CreateMessage, EditMessage, EmojiId,
};
use std::time::Duration;
use tokio::time::sleep;

pub async fn handle_response(
    ctx: &serenity::Context,
    user_message: &serenity::Message,
    bot_message: &serenity::Message,
) -> Result<(), Error> {
    // Wait for embeds to appear (up to 10 seconds)
    for _ in 0..10 {
        sleep(Duration::from_secs(1)).await;
        if !bot_message.embeds.is_empty() {
            break;
        }
    }

    let is_valid_response = check_bot_response(&bot_message);

    // Supress embed
    if is_valid_response {
        user_message
            .channel_id
            .edit_message(
                ctx,
                user_message.id,
                EditMessage::new().suppress_embeds(true),
            )
            .await?;

        return Ok(());
    } else {
        bot_message.delete(ctx).await?;

        let error_embed = CreateEmbed::new()
            .title("Sorry   ꒰ ꒡⌓꒡꒱")
            .description("Something went wrong.")
            .color(0xd1001f);

        user_message
            .delete_reaction_emoji(
                ctx,
                serenity::ReactionType::Custom {
                    animated: false,
                    id: EmojiId::new(1206376642042138724),
                    name: Some("Sanitized".to_string()),
                },
            )
            .await?;

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

        return Ok(());
    }
}

pub async fn handle_interaction_response(
    ctx: &serenity::Context,
    interaction: &CommandInteraction,
    bot_message: &serenity::Message,
) -> Result<(), Error> {
    for _ in 0..10 {
        sleep(Duration::from_secs(1)).await;
        if !bot_message.embeds.is_empty() {
            break;
        }
    }

    let is_valid_response = check_bot_response(&bot_message);
    if !is_valid_response {
        // Delete the invalid bot message
        bot_message.delete(ctx).await?;

        // Create and send ephemeral error message
        let error_embed = CreateEmbed::new()
            .title("Sorry   ꒰ ꒡⌓꒡꒱")
            .description("Something went wrong.")
            .color(0xd1001f);

        interaction
            .create_followup(
                ctx,
                CreateInteractionResponseFollowup::new()
                    .add_embed(error_embed)
                    .ephemeral(true),
            )
            .await?;
    }

    Ok(())
}

fn check_bot_response(bot_message: &serenity::Message) -> bool {
    match &bot_message.content {
        content if content.contains("fxtwitter.com") => bot_message
            .embeds
            .first()
            .and_then(|embed| embed.title.as_ref())
            .map_or(false, |title| title != "FxTwitter / FixupX"),
        content if content.contains("ddinstagram.com") => bot_message
            .embeds
            .first()
            .and_then(|embed| embed.title.as_ref())
            .map_or(false, |title| {
                title != "InstaFix" && title != "Login • Instagram"
            }),
        _ => true,
    }
}
