use crate::Data;
use anyhow::Error;
use poise::serenity_prelude as serenity;
use tracing::debug;

pub async fn events(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            debug!("{:?} App is Online", data_about_bot.user.name)
        }
        serenity::FullEvent::Message { new_message } => {
            handle_message(ctx, new_message).await?;
        }
        _ => {}
    }
    Ok(())
}

async fn handle_message(ctx: &serenity::Context, message: &serenity::Message) -> Result<(), Error> {
    if message.content.to_lowercase().contains("hello")
        && message.author.id != ctx.cache.current_user().id
    {
        let author_name = message.author.name.clone();
        message
            .reply(ctx, format!("Hello! #{}", author_name))
            .await?;
    }
    Ok(())
}
