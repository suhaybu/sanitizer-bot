use std::time::Duration;

use anyhow::Context;
use tokio::sync::{Notify, OnceCell};
use turso::Connection;
use turso::sync::{Builder, Database};

static DB: OnceCell<Database> = OnceCell::const_new();
static PUSH_NOTIFY: Notify = Notify::const_new();

/// Guards every write to the local db file - both application writes
/// (INSERT/UPDATE/DELETE via `conn.execute`) and sync engine pushes.
pub static WRITE_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Initialize database connection during pre run (main.rs).
pub async fn init_database() -> anyhow::Result<()> {
    let db = init_database_internal().await?;
    DB.set(db)
        .map_err(|_| anyhow::anyhow!("Database already initialized"))?;

    let conn = get_connection().await?;

    // Create tables
    create_tables(&conn).await?;

    // Single background worker that serializes/coalesces all push requests
    // so writes never race each other against the remote.
    tokio::spawn(push_worker());

    // Perform initial database pull with small backoff to avoid startup races.
    tokio::spawn(async {
        let mut delay = Duration::from_millis(250);
        let mut last_err: Option<anyhow::Error> = None;
        // Will retry initial database pull 3 times.
        for attempt in 1..=3 {
            match pull_database().await {
                Ok(()) => {
                    tracing::debug!(
                        "Initial database pull completed successfully (attempt {})",
                        attempt
                    );
                    return;
                }
                Err(e) => {
                    last_err = Some(e);
                    tracing::debug!(
                        "Initial database pull failed (attempt {}), retrying after {:?}",
                        attempt,
                        delay
                    );
                    tokio::time::sleep(delay).await;
                    delay *= 2;
                }
            }
        }
        if let Some(e) = last_err {
            tracing::error!("Failed initial database pull after retries: {:?}", e);
        }
    });

    Ok(())
}

fn db() -> anyhow::Result<&'static Database> {
    DB.get()
        .context("Database has not been initialized; call init_database() first")
}

pub async fn get_connection() -> anyhow::Result<Connection> {
    let db = db()?;

    // Retry a few times to handle transient lock/availability during startup
    const MAX_RETRIES: u32 = 5;
    const INITIAL_DELAY: Duration = Duration::from_millis(100);

    for attempt in 1..=MAX_RETRIES {
        match db.connect().await {
            Ok(conn) => return Ok(conn),
            Err(e) => {
                if attempt == MAX_RETRIES {
                    return Err(e)
                        .context("Failed to get database connection after multiple retries.");
                }

                let delay = INITIAL_DELAY * 2_u32.pow(attempt - 1);
                tracing::warn!(
                    attempt,
                    delay_ms = delay.as_millis(),
                    "Database connection failed, retrying"
                );
                tokio::time::sleep(delay).await;
            }
        }
    }
    unreachable!("retry loop must return or error out")
}

/// Ask the background worker to push local writes to the remote.
pub fn request_push() {
    PUSH_NOTIFY.notify_one();
}

async fn push_worker() {
    loop {
        PUSH_NOTIFY.notified().await;
        if let Err(e) = push_database().await {
            tracing::warn!("Failed to push database after write: {:?}", e);
        }
    }
}

/// Push local writes to the remote directly. Prefer `request_push()` for
/// writes so pushes stay serialized; this is exposed for cases (e.g. tests,
/// or explicit "sync now" flows) that need to push and observe the result.
async fn push_database() -> anyhow::Result<()> {
    let _guard = WRITE_LOCK.lock().await;
    tracing::debug!("Pushing local writes to remote");
    db()?
        .push()
        .await
        .context("Failed to push local writes to remote")?;
    Ok(())
}

pub async fn pull_database() -> anyhow::Result<()> {
    tracing::debug!("Pulling changes from remote");
    db()?
        .pull()
        .await
        .context("Failed to pull changes from remote")?;
    tracing::debug!("Database pull completed");
    Ok(())
}

async fn init_database_internal() -> anyhow::Result<Database> {
    let url = std::env::var("TURSO_DATABASE_URL").context("TURSO_DATABASE_URL must be set")?;
    let auth_token = std::env::var("TURSO_AUTH_TOKEN").context("TURSO_AUTH_TOKEN must be set")?;

    tracing::info!("Initializing Turso synced database");

    let db = Builder::new_remote("local.db")
        .with_remote_url(&url)
        .with_auth_token(&auth_token)
        .bootstrap_if_empty(true)
        .build()
        .await
        .context("Failed to build database connection")?;

    Ok(db)
}

async fn create_tables(conn: &Connection) -> anyhow::Result<()> {
    tracing::debug!("Ensuring database schema exists");

    conn.execute("PRAGMA busy_timeout = 5000", ())
        .await
        .context("Failed to set busy_timeout")?;

    let create_server_configs_table = r#"
        CREATE TABLE IF NOT EXISTS server_configs (
            guild_id INTEGER PRIMARY KEY,
            sanitizer_mode INTEGER NOT NULL DEFAULT 0,
            delete_permission INTEGER NOT NULL DEFAULT 0,
            hide_original_embed BOOLEAN NOT NULL DEFAULT true
        )
    "#;

    conn.execute(create_server_configs_table, ())
        .await
        .context("Failed to create server_configs table")?;

    let create_response_map_table = r#"
        CREATE TABLE IF NOT EXISTS response_map (
            user_message_id INTEGER PRIMARY KEY,
            bot_message_id INTEGER NOT NULL,
            guild_id INTEGER,
            channel_id INTEGER NOT NULL
        )
        "#;

    conn.execute(create_response_map_table, ())
        .await
        .context("Failed to create response_map table")?;

    let create_index = r#"
            CREATE INDEX IF NOT EXISTS idx_response_map_location
            ON response_map (guild_id, channel_id)
        "#;

    conn.execute(create_index, ())
        .await
        .context("Failed to create index for response_map")?;

    tracing::debug!("Database schema set.");
    Ok(())
}
