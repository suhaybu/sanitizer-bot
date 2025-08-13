use anyhow::Result;
use dotenvy::dotenv;
use tracing::{debug, error};
use tracing_subscriber::{fmt::time::OffsetTime, EnvFilter};
use time::{macros::format_description, UtcOffset};

use crate::handlers::db;

pub fn init() -> Result<()> {
    setup_logging()?;
    dotenv().expect("Critical Error: Failed to load .env file");

    // Warm up DB and drop the connection before syncing to avoid lock contention
    {
        let _ = db::get_connection()?;
    }

    // Perform initial database sync with small backoff to avoid startup races
    tokio::spawn(async {
        let mut delay = std::time::Duration::from_millis(250);
        let mut last_err: Option<anyhow::Error> = None;
        for attempt in 1..=3 {
            match db::sync_database().await {
                Ok(()) => {
                    debug!("Initial database sync completed successfully (attempt {})", attempt);
                    return;
                }
                Err(e) => {
                    last_err = Some(e);
                    debug!(
                        "Initial database sync failed (attempt {}), retrying after {:?}",
                        attempt, delay
                    );
                    tokio::time::sleep(delay).await;
                    delay = delay * 2;
                }
            }
        }
        if let Some(e) = last_err {
            error!("Failed initial database sync after retries: {:?}", e);
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
