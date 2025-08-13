use anyhow::Context as _;
use twilight_http::Client;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::application::interaction::{application_command::CommandData, Interaction};
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::handlers::sanitize_input;

const INVALID_URL_MESSAGE: &str =
    "❌ Invalid URL. Please provide a valid TikTok, Instagram, or Twitter/X link.";

/// Fix the embed of your link! 🫧
#[derive(CommandModel, CreateCommand, Debug)]
#[command(name = "sanitize", desc = "Fix the embed of your link!")]
pub struct SanitizeCommand {
    /// Your link goes here
    pub link: String,
}

impl SanitizeCommand {
    pub async fn handle(
        interaction: Interaction,
        data: CommandData,
        client: &Client,
    ) -> anyhow::Result<()> {
        let command = SanitizeCommand::from_interaction(data.into())
            .context("failed to parse command data")?;

        let content = match sanitize_input(&command.link).await {
            Some(sanitized) => sanitized,
            None => INVALID_URL_MESSAGE.to_string(),
        };

        let data = InteractionResponseDataBuilder::new().content(content).build();
        let response = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(data),
        };
        client
            .interaction(interaction.application_id)
            .create_response(interaction.id, &interaction.token, &response)
            .await?;
        Ok(())
    }
}
