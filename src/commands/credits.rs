use anyhow::Error;
use poise::serenity_prelude as serenity;

use crate::Context;

/// Roll the credits! 🎺
#[poise::command(slash_command)]
pub async fn credits(ctx: Context<'_>) -> Result<(), Error> {
    let embed = serenity::CreateEmbed::new()
        .title("Credits")
        .description("These are all the super cool projects I rely on:\n\
        -  **Twitter**: Thanks to FixTweet's reliable [FxTwitter](https://github.com/FixTweet/FxTwitter) project\n\
        -  **TikTok & Instagram**: Thanks to [QuickVids](https://quickvids.app/) super fast and easy to use API\n\
        -  **Instagram** (Fallback): Powered by the awesome [InstaFix](https://github.com/Wikidepia/InstaFix)");

    let builder = poise::CreateReply::default().embed(embed).ephemeral(true);

    ctx.send(builder).await?;

    Ok(())
}
