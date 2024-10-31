// src/config/setup.rs
use anyhow::Result;
use dotenvy::dotenv;
use tracing::debug;
use tracing_subscriber::{prelude::*, EnvFilter};

pub fn init() -> Result<()> {
    setup_logging()?;
    load_environment()?;
    debug!("Initialization complete");
    Ok(())
}

fn setup_logging() -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::try_new("info,serenity=warn").expect("Invalid filter"));

    let subscriber =
        tracing_subscriber::registry().with(tracing_subscriber::fmt::layer().with_filter(filter));

    #[cfg(debug_assertions)]
    let subscriber = subscriber.with(console_subscriber::spawn());

    subscriber.init();
    debug!("Logging initialized");
    Ok(())
}

fn load_environment() -> Result<()> {
    dotenv().expect("Critical Error: Failed to load .env file");
    debug!("Environment variables loaded");
    Ok(())
}
