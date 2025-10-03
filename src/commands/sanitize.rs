use anyhow::Context;
use twilight_http::Client;
use twilight_model::{
    application::{
        command::{Command, CommandOption, CommandOptionType, CommandType},
        interaction::{
            Interaction, InteractionContextType,
            application_command::{CommandData, CommandOptionValue},
        },
    },
    http::interaction::{InteractionResponse, InteractionResponseType},
    oauth::ApplicationIntegrationType,
};
use twilight_util::builder::command::CommandBuilder;

use crate::sanitize::UrlProcessor;

pub struct SanitizeCommand;

impl SanitizeCommand {
    /// Creates /sanitize command.
    pub fn create_command() -> Command {
        CommandBuilder::new(
            "sanitize",
            "Fix the embed of your link! 🫧",
            CommandType::ChatInput,
        )
        .contexts([
            InteractionContextType::Guild,
            InteractionContextType::BotDm,
            InteractionContextType::PrivateChannel,
        ])
        .integration_types([
            ApplicationIntegrationType::GuildInstall,
            ApplicationIntegrationType::UserInstall,
        ])
        .option(CommandOption {
            autocomplete: None,
            channel_types: None,
            choices: None,
            description: "Your link goes here".to_string(),
            description_localizations: None,
            kind: CommandOptionType::String,
            max_length: Some(100),
            max_value: None,
            min_length: None,
            min_value: None,
            name: "link".to_string(),
            name_localizations: None,
            options: None,
            required: Some(true),
        })
        .build()
    }

    /// Creates Sanitize command for on right-click.
    pub fn create_command_message() -> Command {
        CommandBuilder::new("Sanitize", "", CommandType::Message)
            .contexts([
                InteractionContextType::Guild,
                InteractionContextType::BotDm,
                InteractionContextType::PrivateChannel,
            ])
            .integration_types([
                ApplicationIntegrationType::GuildInstall,
                ApplicationIntegrationType::UserInstall,
            ])
            .build()
    }

    pub async fn handle(
        ctx: &Interaction,
        client: &Client,
        data: &CommandData,
    ) -> anyhow::Result<()> {
        // Acknowledge the user interaction first.
        let deferred_response = InteractionResponse {
            kind: InteractionResponseType::DeferredChannelMessageWithSource,
            data: None,
        };

        client
            .interaction(ctx.application_id)
            .create_response(ctx.id, &ctx.token, &deferred_response)
            .await?;

        // Extract message input.
        let user_input = match data.kind {
            // For the /command.
            CommandType::ChatInput => {
                let data_option = data
                    .options
                    .first()
                    .context("No Options provided for ChatInput command")?;

                let CommandOptionValue::String(link) = &data_option.value else {
                    anyhow::bail!("Expected String option, got: {:?}", data_option.value);
                };

                link
            }
            // For the right-click on message.
            CommandType::Message => {
                let data_resolved = data
                    .resolved
                    .as_ref()
                    .context("No Resolved data for Message.")?;

                let message = data_resolved
                    .messages
                    .values()
                    .next()
                    .context("No message found in resolved data")?;

                &message.content
            }
            _ => anyhow::bail!("Unexpected CommandType: {:?}", data.kind),
        };

        let url = match UrlProcessor::try_new(user_input) {
            Some(url) => url,
            None => return Ok(()),
        };

        let output = url
            .capture_url()
            .and_then(|captures| captures.format_output())
            .ok_or_else(|| anyhow::anyhow!("Failed to process URL"))?;

        client
            .interaction(ctx.application_id)
            .update_response(&ctx.token)
            .content(Some(&output))
            .await?;

        Ok(())
    }
}
