use anyhow::{Error, Result};
use config::data::Data;
use config::framework::create_framework;
use dotenvy::dotenv;
use poise::serenity_prelude as serenity;

use tracing::{debug, error};
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

mod commands;
mod config;
mod logic;

pub type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::try_new("info,serenity=warn").expect("Invalid filter"));

    let subscriber =
        tracing_subscriber::registry().with(tracing_subscriber::fmt::layer().with_filter(filter));

    #[cfg(debug_assertions)]
    let subscriber = subscriber.with(console_subscriber::spawn());

    subscriber.init();

    dotenv().expect(".env file not found");

    let token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not found");
    let intents = serenity::GatewayIntents::all();
    let framework = create_framework().expect("Failed to create framework");

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .expect("Error creating client");

    debug!("Bot is starting!");
    if let Err(error) = client.start().await {
        error!("Client error: {error:?}");
        std::process::exit(1);
    }

    Ok(())
}

// Implemented example handler. Needs to be updated
async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            // TODO
            debug!("{:?} App is Online", data_about_bot.user.name)
        }
        serenity::FullEvent::Message { new_message } => {
            if new_message.content.to_lowercase().contains("hello")
                && new_message.author.id != ctx.cache.current_user().id
            {
                let author_name = new_message.author.clone().name;
                println!("{author_name}");

                new_message
                    .reply(ctx, format!("Hello! #{}", author_name))
                    .await?;
            }
        }
        _ => {}
    }
    Ok(())
}
