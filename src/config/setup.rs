use anyhow::Result;
use dotenvy::dotenv;
use tracing::{debug, error};
use tracing_subscriber::{fmt::time::OffsetTime, EnvFilter};
use time::{macros::format_description, UtcOffset};

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

    // Format timestamps to second precision: 2025-08-13T16:48:48
    let time_format = format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]");
    // Use UTC without fractional seconds or timezone suffix
    let timer = OffsetTime::new(UtcOffset::UTC, time_format);

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_timer(timer)
        .compact()
        .init();

    debug!("Logging initialized");
    Ok(())
}
