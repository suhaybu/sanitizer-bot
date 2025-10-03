mod commands;
mod database;
mod handlers;
mod models;
mod sanitize;

use std::env;
use std::sync::OnceLock;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use anyhow::Context;
use time::{UtcOffset, macros::format_description};
use tracing::{Level, debug, error};
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::fmt::time::OffsetTime;
use twilight_gateway::{
    CloseFrame, ConfigBuilder, Event, EventTypeFlags, Intents, Shard, StreamExt as _,
};
use twilight_http::Client;
use twilight_model::gateway::{
    payload::outgoing::update_presence::UpdatePresencePayload,
    presence::{ActivityType, MinimalActivity, Status},
};
use twilight_model::id::Id;
use twilight_model::id::marker::{EmojiMarker, UserMarker};

use crate::handlers::handle_event;

// Flag that can be checked by any part of the program.
static SHUTDOWN: AtomicBool = AtomicBool::new(false);
// Gets set during initialization of the bot.
static BOT_USER_ID: OnceLock<Id<UserMarker>> = OnceLock::new();
// Custom Emoji ID
static EMOJI_ID: Id<EmojiMarker> = Id::<EmojiMarker>::new(1206376642042138724);
// 1265681253340942409

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if let Err(err) = run().await {
        error!("Critical error: {:#}", err);
        error!("Error details: {:#?}", err);
        std::process::exit(1);
    }
    Ok(())
}

/// Runner logic that spins the bot up.
async fn run() -> anyhow::Result<()> {
    prerun_init().await?;

    let token = env::var("DISCORD_TOKEN").context("DISCORD_TOKEN environment variable not set")?;
    let intents = Intents::GUILD_MESSAGES
        | Intents::DIRECT_MESSAGES
        | Intents::MESSAGE_CONTENT
        | Intents::GUILD_MESSAGE_REACTIONS;

    // Set the bot's Discord status.
    let activity = MinimalActivity {
        kind: ActivityType::Watching,
        name: String::from("for embeds"),
        url: None,
    };
    let bot_presence = UpdatePresencePayload {
        activities: vec![activity.into()],
        afk: false,
        since: None,
        status: Status::Online,
    };

    // Initialize Twilight HTTP client and gateway configuration.
    let client = Arc::new(Client::new(token.clone()));
    let config = ConfigBuilder::new(token.clone(), intents)
        .presence(bot_presence)
        .build();

    // Register global commands.
    commands::register_global_commands(&client).await?;

    // Fetch & store the bot's user id for future use in event handler.
    let bot = client.current_user().await?.model().await?;
    BOT_USER_ID
        .set(bot.id)
        .expect("BOT_USER_ID already initialized");
    tracing::info!("{} online with ID: {}", bot.name, bot.id);

    // Start gateway shards.
    let shards =
        twilight_gateway::create_recommended(&client, config, |_id, builder| builder.build())
            .await?;
    let shard_len = shards.len();
    let mut senders = Vec::with_capacity(shard_len);
    let mut tasks = Vec::with_capacity(shard_len);

    for shard in shards {
        senders.push(shard.sender());
        tasks.push(tokio::spawn(shard_runner(shard, client.clone())));
    }

    // Handle exiting Ctrl+C gracefully.
    tokio::signal::ctrl_c().await?;
    println!();
    tracing::info!("Shutting down bot gracefully.");

    SHUTDOWN.store(true, Ordering::Relaxed);
    for sender in senders {
        // Ignore error if shard's already shutdown.
        _ = sender.close(CloseFrame::NORMAL);
    }

    // Wait for all background tasks to finish.
    for join_handle in tasks {
        _ = join_handle.await;
    }

    Ok(())
}

/// Handles shards.
async fn shard_runner(mut shard: Shard, client: Arc<Client>) {
    // Runs until next_event returns None.
    while let Some(item) = shard.next_event(EventTypeFlags::all()).await {
        let event = match item {
            Ok(Event::GatewayClose(_)) if SHUTDOWN.load(Ordering::Relaxed) => break,
            Ok(event) => event,
            Err(error) => {
                tracing::warn!(?error, "error while receiving event");
                continue;
            }
        };

        // Process Discord events (see `process.rs` file).
        tracing::debug!(kind = ?event.kind(), shard = ?shard.id().number(), "received event");
        tokio::spawn(handle_event(event, client.clone()));
    }
}

/// Pre-run initialization sequence for the bot.
async fn prerun_init() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    // Initialize logging with tracing.
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(Level::INFO.as_str()));

    let time_format = format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]");
    let timer = OffsetTime::new(UtcOffset::UTC, time_format);

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_timer(timer)
        .compact()
        .init();

    debug!("Logging initialized");
    // Warm up DB and drop the connection before syncing to avoid lock contention.
    {
        let _ = database::get_connection()?;
    }

    // Perform initial database sync with small backoff to avoid startup races.
    tokio::spawn(async {
        let mut delay = std::time::Duration::from_millis(250);
        let mut last_err: Option<anyhow::Error> = None;
        // Will retry initial database sync 3 times.
        for attempt in 1..=3 {
            match database::sync_database().await {
                Ok(()) => {
                    debug!(
                        "Initial database sync completed successfully (attempt {})",
                        attempt
                    );
                    return;
                }
                Err(e) => {
                    last_err = Some(e);
                    debug!(
                        "Initial database sync failed (attempt {}), retrying after {:?}",
                        attempt, delay
                    );
                    tokio::time::sleep(delay).await;
                    delay *= 2;
                }
            }
        }
        if let Some(e) = last_err {
            error!("Failed initial database sync after retries: {:?}", e);
        }
    });

    Ok(())
}
