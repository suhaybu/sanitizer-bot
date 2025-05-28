use std::sync::LazyLock;

use anyhow::{Context, Result};
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
    debug!("Creating database tables if they don't exist");

    let create_sanitizer_table = r#"
        CREATE TABLE IF NOT EXISTS Sanitizer (
            guild_id INTEGER PRIMARY KEY,
            sanitizer_mode INTEGER NOT NULL DEFAULT 0,
            delete_permission INTEGER NOT NULL DEFAULT 0,
            hide_original_embed BOOLEAN NOT NULL DEFAULT false
        )
    "#;

    conn.execute(create_sanitizer_table, ())
        .await
        .context("Failed to create Sanitizer table")?;

    debug!("Database tables created successfully");
    Ok(())
}

pub fn get_connection() -> Result<Connection> {
    DB.connect().context("Failed to get database connection")
}

pub async fn sync_database() -> Result<()> {
    debug!("Syncing database with remote");
    DB.sync().await.context("Failed to sync database")?;
    debug!("Database sync completed");
    Ok(())
}
