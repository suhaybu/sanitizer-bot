use anyhow::{Context as _, Error, Result};
use libsql::Database;
use poise::FrameworkOptions;
use poise::serenity_prelude as serenity;
use shuttle_runtime::SecretStore;
use tracing::{debug, error, info};

use config::data::Data;

mod commands;
mod config;
mod handlers;

pub type Context<'a> = poise::ApplicationContext<'a, Data, Error>;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_turso::Turso(
        addr = "YOUR_URL_GOES_HERE",    // Replace with your actual Turso database URL
        token = "{secrets.TURSO_AUTH_TOKEN}"
    )]
    database: Database,
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    info!("Starting Sanitizer Bot");

    // Initialize database
    handlers::db::init_database(database);
    
    // Setup database tables
    handlers::db::setup_database()
        .await
        .context("Failed to setup database")?;

    // Perform initial database sync in background (skip if local file mode)
    tokio::spawn(async {
        match handlers::db::sync_database().await {
            Ok(()) => {
                debug!("Initial database sync completed successfully");
            }
            Err(e) => {
                // Sync not supported in local file mode - this is expected during local development
                let error_msg = format!("{:#}", e);
                if error_msg.contains("Sync is not supported") || error_msg.contains("File mode") {
                    debug!("Database sync skipped (running in local file mode)");
                } else {
                    error!("Failed initial database sync: {:?}", e);
                }
            }
        }
    });

    let token = secrets
        .get("DISCORD_TOKEN")
        .context("DISCORD_TOKEN not found in secrets")?;

    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands)
                    .await
                    .context("Failed to register commands globally")?;
                Ok(Data {})
            })
        })
        .options(FrameworkOptions {
            prefix_options: poise::PrefixFrameworkOptions {
                mention_as_prefix: false,
                ..Default::default()
            },
            event_handler: |framework, event| {
                Box::pin(handlers::get_event_handler(framework, event))
            },
            commands: crate::commands::get_all(), // Loads all commands from commands/mod.rs -> fn get_all
            ..Default::default()
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .context("Failed to create Discord client")?;

    Ok(client.into())
}
