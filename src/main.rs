mod commands;
mod database;
mod handlers;
mod models;
mod sanitize;

use std::sync::{
    Arc, OnceLock,
    atomic::{AtomicBool, Ordering},
};

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

struct DiscordBot {
    token: String,
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for DiscordBot {
    async fn bind(self, _addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        tokio::select! {
            result = run_bot(self.token) => {
                if let Err(e) = result {
                    tracing::error!("Bot error: {:#}", e);
                    return Err(shuttle_runtime::Error::Custom(
                        shuttle_runtime::CustomError::msg(format!("Bot failed: {}", e))
                    ));
                }
            }
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("Received shutdown signal");
            }
        }
        Ok(())
    }
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
    #[shuttle_turso::Turso(
        addr = "{secrets.TURSO_DATABASE_URL}",
        token = "{secrets.TURSO_AUTH_TOKEN}"
    )]
    database: libsql::Database,
) -> Result<DiscordBot, shuttle_runtime::Error> {
    database::setup_database(database)
        .await
        .map_err(|_| shuttle_runtime::Error::Database("Failed to setup database".to_string()))?;

    let token = secrets.get("DISCORD_TOKEN").ok_or_else(|| {
        shuttle_runtime::Error::Custom(shuttle_runtime::CustomError::msg(
            "DISCORD_TOKEN not found in secrets",
        ))
    })?;

    Ok(DiscordBot { token })
}

/// Runner logic that spins the bot up.
async fn run_bot(token: String) -> anyhow::Result<()> {
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
