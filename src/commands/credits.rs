use anyhow::Result;
use poise::serenity_prelude::{self as serenity};

use crate::Context;

/// Roll the credits! 🎺
#[poise::command(slash_command)]
pub async fn credits(ctx: Context<'_>) -> Result<()> {
    let embed = serenity::CreateEmbed::new()
        .title("Credits")
        .description("These are all the super cool projects I rely on:\n\
        -  **Twitter**: Thanks to FixTweet's reliable [FxTwitter](https://github.com/FixTweet/FxTwitter) project\n\
        -  **TikTok & Instagram**: Thanks to [kkScript](https://kkscript.com/)\n\
        -  **Instagram** (Fallback): Powered by the awesome [InstaFix](https://github.com/Wikidepia/InstaFix)\n\
        -# The code for this bot is public sourced on my GitHub [here](https://github.com/suhaybu/sanitizer-bot).");

    let builder = poise::CreateReply::default().embed(embed).ephemeral(true);

    ctx.send(builder).await?;

    Ok(())
}
