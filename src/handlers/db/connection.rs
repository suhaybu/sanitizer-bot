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

    let _ = db.connect().context("Failed to connect to the database")?;

    Ok(db)
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
