use dotenvy::dotenv;
use poise::serenity_prelude as serenity;

use config::data::Data;
use config::framework::create_framework;
use config::Error;

mod commands;
mod config;
mod logic;

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");

    let token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not found");
    let intents = serenity::GatewayIntents::all();
    let framework = create_framework().expect("Failed to create framework");

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .expect("Error creating client");

    println!("Bot is starting!");
    if let Err(e) = client.start().await {
        println!("Client error: {e:?}")
    }
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
            print!("{:?} App is Online", data_about_bot.user.name)
        }
        serenity::FullEvent::Message { new_message } => {
            if new_message.content.to_lowercase().contains("hello")
                && new_message.author.id != ctx.cache.current_user().id
            {
                let author_name = new_message.clone().author.global_name;

                new_message
                    .reply(ctx, format!("Hello! #{:?}", author_name))
                    .await?;
            }
        }
        _ => {}
    }
    Ok(())
}
