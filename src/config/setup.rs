use anyhow::Result;
use dotenvy::dotenv;
use tracing::{debug, error};
use tracing_subscriber::EnvFilter;

use crate::handlers::db;

pub fn init() -> Result<()> {
    setup_logging()?;
    dotenv().expect("Critical Error: Failed to load .env file");

    let _conn = db::get_connection()?;

    // Perform initial database sync
    tokio::spawn(async {
        if let Err(e) = db::sync_database().await {
            error!("Failed initial database sync: {:?}", e);
        } else {
            debug!("Initial database sync completed successfully");
        }
    });

    debug!("Initialization complete");
    Ok(())
}

fn setup_logging() -> Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::try_new("info,serenity=warn").expect("Invalid default filter")
    });

    // Debug mode
    // let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
    //     EnvFilter::try_new("sanitizer_bot_rs=debug,serenity=warn,rustls=warn,tungstenite=warn")
    //         .expect("Invalid default filter")
    // });

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .init();

    debug!("Logging initialized");
    Ok(())
}
