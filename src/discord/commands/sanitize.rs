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
    channel::message::{
        Component, EmojiReactionType, MessageFlags,
        component::{
            ActionRow, Button, ButtonStyle, Container, MediaGallery, MediaGalleryItem, TextDisplay,
            UnfurledMediaItem,
        },
    },
    http::interaction::{InteractionResponse, InteractionResponseType},
    oauth::ApplicationIntegrationType,
};
use twilight_util::builder::command::CommandBuilder;

use crate::utils::sanitize::UrlProcessor;

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

        let original_url = url
            .get_original_url()
            .expect("Original URL could not be retrieved.");

        let output = url
            .capture_url()
            .and_then(|captures| captures.format_output_interaction())
            .ok_or_else(|| anyhow::anyhow!("Failed to process URL"))?;

        // For now, a way to handle Twitter responses as non-container.
        match output.1 {
            Some(url) => {
                let add_delete_button = ctx.is_dm()
                    || ctx.is_guild()
                        && ctx.context.is_some_and(|ctx_type| {
                            ctx_type == InteractionContextType::PrivateChannel
                        });

                let (container, action_row) = Self::construct_response_container(
                    output.0.as_str(),
                    url,
                    original_url,
                    false,
                    add_delete_button,
                );

                client
                    .interaction(ctx.application_id)
                    .update_response(&ctx.token)
                    .components(Some(&[Component::Container(container), action_row]))
                    .flags(MessageFlags::IS_COMPONENTS_V2)
                    .await?;
            }
            None => {
                client
                    .interaction(ctx.application_id)
                    .update_response(&ctx.token)
                    .content(Some(&output.0))
                    .await?;
            }
        }
        Ok(())
    }

    fn construct_response_container(
        display_text: &str,
        url: String,
        original_url: String,
        is_spoiler: bool,
        add_delete_button: bool,
    ) -> (Container, Component) {
        let mut container_components = Vec::new();

        container_components.push(Component::TextDisplay(TextDisplay {
            id: None,
            content: format!("-# **{}**", display_text),
        }));

        container_components.push(Component::MediaGallery(MediaGallery {
            id: None,
            items: vec![MediaGalleryItem {
                media: UnfurledMediaItem {
                    url: url,
                    proxy_url: None,
                    height: None,
                    width: None,
                    content_type: None,
                },
                description: None,
                spoiler: Some(false),
            }],
        }));

        let container = Container {
            id: None,
            accent_color: None,
            spoiler: Some(is_spoiler),
            components: container_components,
        };

        let mut buttons = vec![Component::Button(Button {
            id: None,
            custom_id: None,
            disabled: false,
            emoji: Some(EmojiReactionType::Unicode {
                name: "🔗".to_string(),
            }),
            label: Some("Open Link".to_string()),
            style: ButtonStyle::Link,
            url: Some(original_url),
            sku_id: None,
        })];

        if add_delete_button {
            buttons.push(Component::Button(Button {
                id: None,
                custom_id: Some("delete".to_string()),
                disabled: false,
                emoji: Some(EmojiReactionType::Unicode {
                    name: "🗑️".to_string(),
                }),
                label: Some("Delete".to_string()),
                style: ButtonStyle::Danger,
                url: None,
                sku_id: None,
            }));
        }

        let action_row = Component::ActionRow(ActionRow {
            id: None,
            components: buttons,
        });

        (container, action_row)
    }
}
