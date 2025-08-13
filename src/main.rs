use std::{env, sync::Arc};

use anyhow::Context as _;
use futures_util::StreamExt;
use tracing::error;

mod commands;
mod config;
mod handlers;
mod process;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if let Err(err) = run().await {
        error!("Critical error: {:#}", err);
        error!("Error details: {:#?}", err);
        std::process::exit(1);
    }
    Ok(())
}

async fn run() -> anyhow::Result<()> {
    config::setup::init()?;

    let token = env::var("DISCORD_TOKEN").context("DISCORD_TOKEN not found in environment")?;

    // Initialize Twilight HTTP client and gateway configuration.
    let client = Arc::new(twilight_http::Client::new(token.clone()));
    let config = twilight_gateway::Config::builder(
        token.clone(),
        // Use message content intents as we parse links from messages.
        twilight_gateway::Intents::GUILDS
            | twilight_gateway::Intents::GUILD_MESSAGES
            | twilight_gateway::Intents::MESSAGE_CONTENT,
    )
    .build();

    // Register global commands.
    use twilight_interactions::command::CreateCommand;
    let commands = [
        commands::sanitize::SanitizeCommand::create_command().into(),
        commands::credits::CreditsCommand::create_command().into(),
        commands::config::ConfigCommand::create_command().into(),
    ];

    let application = client.current_user_application().await?.model().await?;
    let interaction_client = client.interaction(application.id);

    tracing::info!(
        "Logged in to Discord as {} with ID: {}",
        application.name,
        application.id
    );

    if let Err(error) = interaction_client.set_global_commands(&commands).await {
        tracing::error!(?error, "failed to register commands");
    }

    // Fetch bot user to know our user ID for mention checks.
    let bot_user_id = client.current_user().await?.model().await?.id;

    // Start gateway shards.
    let mut shards = twilight_gateway::stream::create_recommended(&client, config, |_id, builder| {
        builder.build()
    })
    .await?
    .collect::<Vec<_>>();
    let mut stream = twilight_gateway::stream::ShardEventStream::new(shards.iter_mut());

    // Process Discord events
    while let Some((shard, event)) = stream.next().await {
        let event = match event {
            Ok(event) => event,
            Err(error) => {
                if error.is_fatal() {
                    tracing::error!(?error, "fatal error while receiving event");
                    break;
                }

                tracing::warn!(?error, "error while receiving event");
                continue;
            }
        };

        tracing::debug!(kind = ?event.kind(), shard = ?shard.id().number(), "received event");
        tokio::spawn(process::process_events(event, client.clone(), bot_user_id));
    }

    Ok(())
}
