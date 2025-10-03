//! Stores business logic for commands

mod credits;
mod sanitize;
mod settings;

pub use credits::CreditsCommand;
pub use sanitize::SanitizeCommand;
pub use settings::SettingsCommand;

use std::sync::Arc;

use twilight_http::Client;

/// This command registers all the discord commands, and is called in main::run().
pub async fn register_global_commands(client: &Arc<Client>) -> anyhow::Result<()> {
    let commands = [
        credits::CreditsCommand::create_command(),
        settings::SettingsCommand::create_command(),
        sanitize::SanitizeCommand::create_command(),
        sanitize::SanitizeCommand::create_command_message(),
    ];
    let application = client.current_user_application().await?.model().await?;

    if let Err(error) = client
        .interaction(application.id)
        .set_global_commands(&commands)
        .await
    {
        tracing::error!(?error, "failed to register commands");
    }

    tracing::info!("Registered {} global commands", commands.len());
    Ok(())
}
