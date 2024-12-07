use anyhow::Error;
use poise::serenity_prelude as srn;
use tracing::{debug, info};

use crate::handlers::{handle_event_response, sanitize_input};
use crate::Data;

pub async fn get_event_handler(
    ctx: &srn::Context,
    event: &srn::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    match event {
        srn::FullEvent::Ready { data_about_bot, .. } => {
            info!("🤖 {} is Online", data_about_bot.user.name.to_string())
        }
        srn::FullEvent::Message { new_message } => {
            on_message(ctx, new_message).await?;
        }
        _ => {}
    }
    Ok(())
}

async fn on_message(ctx: &srn::Context, message: &srn::Message) -> Result<(), Error> {
    debug!("Message received from user: {}", message.author.name);
    debug!("Channel type: {:?}", message.channel_id);

    // Exits if message author is from bot itself
    if message.author.id == ctx.cache.current_user().id {
        debug!("Skipping message from self");
        return Ok(());
    }

    let input = message.content.trim();
    debug!("Processing input: {}", input);

    let response = match sanitize_input(input).await {
        None => return Ok(()), // Exit early if no match
        Some(response) => response,
    };

    let bot_message = message.reply(ctx, response).await?;

    // message
    //     .react(
    //         ctx,
    //         srn::ReactionType::Custom {
    //             animated: false,
    //             id: EmojiId::new(1206376642042138724),
    //             name: Some("Sanitized".to_string()),
    //         },
    //     )
    //     .await?;

    handle_event_response(ctx, message, &bot_message).await?;

    Ok(())
}
