use std::sync::LazyLock;

use anyhow::{Context, Result};
use std::time::Duration;
use libsql::{Builder, Connection, Database};
use tracing::{debug, info};

static DB: LazyLock<Database> = LazyLock::new(|| {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            init_database()
                .await
                .expect("Failed to initialized database")
        })
    })
});

async fn init_database() -> Result<Database> {
    let url = std::env::var("TURSO_DATABASE_URL").context("TURSO_DATABASE_URL must be set")?;
    let auth_token = std::env::var("TURSO_AUTH_TOKEN").context("TURSO_AUTH_TOKEN must be set")?;

    info!("Initializing Turso Embedded Replica database");

    let db = Builder::new_remote_replica("local.db", url, auth_token)
        .build()
        .await
        .context("Failed to build database connection")?;

    let conn = db.connect().context("Failed to connect to the database")?;
    create_tables(&conn)
        .await
        .context("Failed to create database tables")?;

    Ok(db)
}

async fn create_tables(conn: &Connection) -> Result<()> {
    debug!("Ensuring database schema exists");

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
        .context("Failed to create Sanitizer table")?;

    debug!("Database schema ensured");
    Ok(())
}

pub fn get_connection() -> Result<Connection> {
    // Retry a few times to handle transient lock/availability during startup
    let mut delay = Duration::from_millis(100);
    for attempt in 1..=5 {
        match DB.connect() {
            Ok(conn) => return Ok(conn),
            Err(e) => {
                if attempt == 5 {
                    return Err(e).context("Failed to get database connection");
                }
                tracing::debug!(
                    "get_connection failed (attempt {}), retrying in {:?}",
                    attempt, delay
                );
                std::thread::sleep(delay);
                delay *= 2;
            }
        }
    }
    unreachable!("retry loop must return or error out")
}

pub async fn sync_database() -> Result<()> {
    debug!("Syncing database with remote");
    DB.sync().await.context("Failed to sync database")?;
    debug!("Database sync completed");
    Ok(())
}
