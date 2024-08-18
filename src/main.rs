use dotenvy::dotenv;
use poise::serenity_prelude as serenity;

use config::data::Data;
use config::framework::create_framework;

mod commands;
mod config;

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
