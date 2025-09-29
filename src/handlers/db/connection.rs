use anyhow::{Context, Result};
use libsql::{Connection, Database};
use std::sync::OnceLock;
use std::time::Duration;
use tracing::{debug, warn};

static DB: OnceLock<Database> = OnceLock::new();

/// Initialize the database with the provided Database instance from Shuttle
pub fn init_database(database: Database) {
    DB.set(database).expect("Database already initialized");
}

/// Get the initialized database instance
fn get_db() -> &'static Database {
    DB.get().expect("Database not initialized. Call init_database first.")
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
    // Retry a few times to handle transient lock/availability issues
    let mut delay = Duration::from_millis(100);
    let max_attempts = 5;
    
    for attempt in 1..=max_attempts {
        match get_db().connect() {
            Ok(conn) => {
                if attempt > 1 {
                    debug!("Database connection established on attempt {}", attempt);
                }
                return Ok(conn);
            }
            Err(e) => {
                if attempt == max_attempts {
                    return Err(e).context("Failed to get database connection after multiple attempts");
                }
                warn!(
                    "Database connection failed (attempt {}/{}), retrying in {:?}: {}",
                    attempt, max_attempts, delay, e
                );
                std::thread::sleep(delay);
                delay = delay.saturating_mul(2); // Exponential backoff
            }
        }
    }
    
    unreachable!("Retry loop must return or error out")
}

pub async fn sync_database() -> Result<()> {
    debug!("Syncing database with remote");
    get_db().sync().await.context("Failed to sync database")?;
    debug!("Database sync completed");
    Ok(())
}

/// Setup database tables - should be called during initialization
pub async fn setup_database() -> Result<()> {
    let conn = get_connection()?;
    create_tables(&conn).await?;
    Ok(())
}
