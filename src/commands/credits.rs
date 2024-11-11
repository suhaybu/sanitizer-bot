use crate::Context;
use anyhow::Error;
use poise::serenity_prelude as serenity;

/// Displays your or another user's account creation date
#[poise::command(slash_command)]
pub async fn credits(ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}
