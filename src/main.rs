use anyhow::{Context as _, Error, Result};
use poise::serenity_prelude as serenity;
use poise::{Framework, FrameworkOptions};
use tracing::error;

use config::data::Data;

mod commands;
mod config;
mod handlers;

pub type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(err) = run().await {
        error!("Critical error: {:#}", err);
        error!("Error details: {:#?}", err);
        std::process::exit(1);
    }
    Ok(())
}

async fn run() -> Result<()> {
    config::setup::init()?;
    let token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not found in environment");

    let intents = serenity::GatewayIntents::all();

    let framework = Framework::builder()
        .options(FrameworkOptions {
            event_handler: |ctx, event, framework, data| {
                Box::pin(handlers::discord::events(ctx, event, framework, data))
            },
            commands: commands::get_all(),
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands)
                    .await
                    .context("Failed to register commands globally")?;
                Ok(Data {})
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .context("Failed to create Discord client")?;

    client.start().await.context("Failed to start client")?;

    Ok(())
}
