use std::sync::LazyLock;

use anyhow::Context;
use libsql::{Connection, Database};
use std::time::Duration;

static DB: LazyLock<Database> = LazyLock::new(|| {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            init_database_internal()
                .await
                .expect("Failed to initialize database")
        })
    })
});

/// Initialize database connection during pre run (main.rs).
pub async fn init_database() -> anyhow::Result<()> {
    // Force initialization of the lazy static, and warm up DB connecton.
    let _ = &*DB;
    let conn = get_connection()?;

    // Create tables
    create_tables(&conn).await?;

    // Perform initial database sync with small backoff to avoid startup races.
    tokio::spawn(async {
        let mut delay = Duration::from_millis(250);
        let mut last_err: Option<anyhow::Error> = None;
        // Will retry initial database sync 3 times.
        for attempt in 1..=3 {
            match sync_database().await {
                Ok(()) => {
                    tracing::debug!(
                        "Initial database sync completed successfully (attempt {})",
                        attempt
                    );
                    return;
                }
                Err(e) => {
                    last_err = Some(e);
                    tracing::debug!(
                        "Initial database sync failed (attempt {}), retrying after {:?}",
                        attempt,
                        delay
                    );
                    tokio::time::sleep(delay).await;
                    delay *= 2;
                }
            }
        }
        if let Some(e) = last_err {
            tracing::error!("Failed initial database sync after retries: {:?}", e);
        }
    });

    Ok(())
}

pub fn get_connection() -> anyhow::Result<Connection> {
    // Retry a few times to handle transient lock/availability during startup
    const MAX_RETRIES: u32 = 5;
    const INITIAL_DELAY: Duration = Duration::from_millis(100);

    for attempt in 1..=MAX_RETRIES {
        match DB.connect() {
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
                std::thread::sleep(delay);
            }
        }
    }
    unreachable!("retry loop must return or error out")
}

pub async fn sync_database() -> anyhow::Result<()> {
    tracing::debug!("Syncing database with remote");

    match DB.sync().await {
        Ok(_) => {
            tracing::debug!("Database sync completed");
            Ok(())
        }
        Err(e) if e.to_string().contains("Sync is not supported") => {
            tracing::debug!("Skipping sync - running in local mode");
            Ok(())
        }
        Err(e) => Err(e).context("Failed to sync database with remote"),
    }
}

async fn init_database_internal() -> anyhow::Result<Database> {
    let url = std::env::var("TURSO_DATABASE_URL").context("TURSO_DATABASE_URL must be set")?;
    let auth_token = std::env::var("TURSO_AUTH_TOKEN").context("TURSO_AUTH_TOKEN must be set")?;

    tracing::info!("Initializing Turso Embedded Replica database");

    let db = libsql::Builder::new_remote_replica("local.db", url, auth_token)
        .build()
        .await
        .context("Failed to build database connection")?;

    Ok(db)
}

async fn create_tables(conn: &Connection) -> anyhow::Result<()> {
    tracing::debug!("Ensuring database schema exists");

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

    tracing::debug!("Database schema set.");
    Ok(())
}
