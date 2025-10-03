use std::sync::OnceLock;

use anyhow::Context;
use libsql::{Connection, Database};

static DB: OnceLock<Database> = OnceLock::new();

pub async fn setup_database(database: Database) -> anyhow::Result<()> {
    DB.set(database)
        .map_err(|_| anyhow::anyhow!("Database already initialized"))?;

    tracing::info!("Database initialized successfully");

    let conn = get_connection().context("Failed to get database connection")?;

    create_tables(&conn)
        .await
        .context("Failed to create database tables")?;

    tokio::spawn(async {
        let mut delay = std::time::Duration::from_millis(250);
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
                    tracing::debug!(
                        "Initial database sync failed (attempt {}), retrying after {:?}",
                        attempt,
                        delay
                    );
                    tokio::time::sleep(delay).await;
                    delay *= 2;
                    if attempt == 3 {
                        tracing::error!("Failed initial database sync after retries: {:?}", e);
                    }
                }
            }
        }
    });

    Ok(())
}

pub fn get_connection() -> anyhow::Result<Connection> {
    let db = DB.get().context("Database not initialized")?;
    db.connect().context("Failed to get database connection")
}

pub async fn sync_database() -> anyhow::Result<()> {
    tracing::debug!("Syncing database with remote");
    let db = DB.get().context("Database not initialized")?;

    match db.sync().await {
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

async fn create_tables(conn: &Connection) -> anyhow::Result<()> {
    tracing::debug!("Ensuring database schema exists");

    let create_sanitizer_table = r#"
        CREATE TABLE IF NOT EXISTS server_configs (
            guild_id INTEGER PRIMARY KEY,
            sanitizer_mode INTEGER NOT NULL DEFAULT 0,
            delete_permission INTEGER NOT NULL DEFAULT 0,
            hide_original_embed BOOLEAN NOT NULL DEFAULT true
        )
    "#;

    conn.execute(create_sanitizer_table, ())
        .await
        .context("Failed to create server_configs table")?;

    tracing::debug!("Database schema ensured");
    Ok(())
}
