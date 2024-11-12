use crate::Data;
use anyhow::Error;
use poise::serenity_prelude::{
    CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use poise::ApplicationContext;

/// Roll the credits! 🎺
#[poise::command(slash_command)]
pub async fn credits(ctx: ApplicationContext<'_, Data, Error>) -> Result<(), Error> {
    let embed = CreateEmbed::new()
        .title("Credits")
        .description("These are all the super cool projects I rely on:\n\
        -  **Twitter**: Thanks to FixTweet's reliable [FxTwitter](https://github.com/FixTweet/FxTwitter) project\n\
        -  **TikTok & Instagram**: Thanks to [QuickVids](https://quickvids.app/) super fast and easy to use API\n\
        -  **Instagram** (Fallback): Powered by the awesome [InstaFix](https://github.com/Wikidepia/InstaFix)");

    // ctx.send(poise::CreateReply::default().embed(embed)).await?;

    ctx.interaction
        .create_response(
            ctx.serenity_context,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .add_embed(embed)
                    .ephemeral(true),
            ),
        )
        .await?;

    Ok(())
}
