use std::{mem, sync::Arc};

use twilight_gateway::Event;
use twilight_http::Client;
use twilight_model::application::interaction::{
    application_command::CommandData, Interaction, InteractionData,
};
use twilight_model::id::{marker::UserMarker, Id};

/// Process incoming Discord gateway events.
pub async fn process_events(event: Event, client: Arc<Client>, bot_user_id: Id<UserMarker>) {
    match event {
        Event::InteractionCreate(interaction) => {
            let mut interaction = interaction.0;
            if let Some(InteractionData::ApplicationCommand(data)) = mem::take(&mut interaction.data)
            {
                if let Err(error) = handle_command(interaction, *data, &client).await {
                    tracing::error!(?error, "error while handling command");
                }
            }
        }
        Event::MessageCreate(msg) => {
            crate::handlers::event::on_message_twilight(*msg, client.clone(), bot_user_id).await;
        }
        Event::ReactionAdd(reaction) => {
            crate::handlers::event::on_reaction_add_twilight(*reaction, client.clone()).await;
        }
        _ => {}
    }
}

async fn handle_command(
    interaction: Interaction,
    data: CommandData,
    client: &Client,
) -> anyhow::Result<()> {
    tracing::info!(command = ?data.name, "handling command");
    match &*data.name {
        "sanitize" => crate::commands::sanitize::SanitizeCommand::handle(interaction, data, client).await,
        "credits" => crate::commands::credits::CreditsCommand::handle(interaction, data, client).await,
        "config" => crate::commands::config::ConfigCommand::handle(interaction, data, client).await,
        _ => Ok(()),
    }
}


