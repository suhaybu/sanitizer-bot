use poise::serenity_prelude::EditInteractionResponse;
use poise::serenity_prelude::{self as srn, CreateEmbed, CreateMessage, EditMessage};
use std::time::Duration;
use tokio::time::sleep;
use tracing::debug;

use crate::Context;
use crate::Result;

// On message (Listener)
pub async fn handle_event_response(
    ctx: &srn::Context,
    user_message: &srn::Message,
    bot_message: &srn::Message,
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

            // user_message
            //     .delete_reaction_emoji(
            //         ctx,
            //         srn::ReactionType::Custom {
            //             animated: false,
            //             id: EmojiId::new(1206376642042138724),
            //             name: Some("Sanitized".to_string()),
            //         },
            //     )
            //     .await?;

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
    bot_message: &srn::Message,
) -> Result<()> {
    let valid_response = tokio::time::timeout(Duration::from_secs(10), async {
        while bot_message.embeds.is_empty() {
            sleep(Duration::from_secs(1)).await;
        }
        check_bot_response(bot_message)
    })
    .await
    .unwrap_or(false);

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
fn check_bot_response(bot_message: &srn::Message) -> bool {
    match &bot_message.content {
        content if content.contains("fxtwitter.com") => !matches!(
            bot_message
                .embeds
                .first()
                .and_then(|e| e.title.as_ref())
                .map(String::as_str),
            Some("FxTwitter / FixupX")
        ),
        content if content.contains("ddinstagram.com") => !matches!(
            bot_message
                .embeds
                .first()
                .and_then(|e| e.title.as_ref())
                .map(String::as_str),
            Some("InstaFix") | Some("Login • Instagram")
        ),
        _ => true,
    }
}
