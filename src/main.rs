use anyhow::{Context as _, Error, Result};
use poise::serenity_prelude as serenity;
use poise::{Framework, FrameworkOptions};
use std::env::var;
use tracing::error;

use config::data::Data;

mod commands;
mod config;
mod handlers;

pub type Context<'a> = poise::ApplicationContext<'a, Data, Error>;

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

    let token = var("DISCORD_TOKEN").expect("DISCORD_TOKEN not found in environment");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = Framework::builder()
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands)
                    .await
                    .context("Failed to register commands globally")?;
                Ok(Data {})
            })
        })
        .options(FrameworkOptions {
            event_handler: |framework, event| {
                Box::pin(handlers::get_event_handler(framework, event))
            },
            commands: commands::get_all(), // Loads all commands from commands/mod.rs -> fn get_all
            ..Default::default()
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .context("Failed to create Discord client")?;

    client.start().await.context("Failed to start client")?;

    Ok(())
}
