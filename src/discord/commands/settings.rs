//! Settings Command: Creates a Settings Container allowing users to configure the bot's behavior.

use anyhow::{Context, bail};
use twilight_http::Client;
use twilight_model::{
    application::{
        command::{Command, CommandType},
        interaction::{
            Interaction, InteractionContextType, message_component::MessageComponentInteractionData,
        },
    },
    channel::message::{
        EmojiReactionType, MessageFlags,
        component::{Component, Container, SelectMenuType, SeparatorSpacingSize},
    },
    http::interaction::{InteractionResponse, InteractionResponseType},
    oauth::ApplicationIntegrationType,
};
use twilight_util::builder::{
    InteractionResponseDataBuilder,
    command::CommandBuilder,
    message::{
        ActionRowBuilder, ContainerBuilder, SelectMenuBuilder, SelectMenuOptionBuilder,
        SeparatorBuilder, TextDisplayBuilder,
    },
};

use crate::{
    discord::models::{DeletePermission, HideOriginalEmbed, SanitizerMode, SettingsMenuType},
    utils::{ServerConfig, config_cache},
};

pub struct SettingsCommand;

impl SettingsCommand {
    /// Creates /settings command.
    pub fn create_command() -> Command {
        CommandBuilder::new(
            "settings",
            "Configure Sanitizer's settings for this server 🛠️",
            CommandType::ChatInput,
        )
        .contexts([InteractionContextType::Guild])
        .integration_types([ApplicationIntegrationType::GuildInstall])
        .build()
    }

    /// Handles responding to command invocation.
    pub async fn handle(ctx: &Interaction, client: &Client) -> anyhow::Result<()> {
        let Some(guild_id) = ctx.guild_id else {
            bail!("Settings can only be used in guilds!")
        };

        // Get current server configuration
        let config = config_cache().get_or_fetch(guild_id.get()).await?;

        let settings_container = Self::construct_container(&config);
        let data = InteractionResponseDataBuilder::new()
            .components([Component::Container(settings_container)])
            .flags(MessageFlags::IS_COMPONENTS_V2 | MessageFlags::EPHEMERAL)
            .build();

        let response = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(data),
        };

        client
            .interaction(ctx.application_id)
            .create_response(ctx.id, &ctx.token, &response)
            .await?;

        Ok(())
    }

    /// Handles responding to component (Container) invocations
    pub async fn handle_component(
        ctx: &Interaction,
        menu_type: SettingsMenuType,
        data: &MessageComponentInteractionData,
        client: &Client,
    ) -> anyhow::Result<()> {
        let Some(guild_id) = ctx.guild_id else {
            bail!("Settings can only be used in guilds!")
        };

        let selected_value = data
            .values
            .first()
            .context("No value selected from dropdown")?;

        // Debug logging to see what value we're trying to parse
        tracing::debug!("Selected value from dropdown: '{}'", selected_value);
        tracing::debug!("Menu type: {:?}", menu_type);

        let cache = config_cache();
        let mut config = cache.get_or_fetch(guild_id.get()).await?;

        // Update the appropriate setting
        match menu_type {
            SettingsMenuType::SanitizerMode => {
                config.sanitizer_mode = selected_value
                    .parse()
                    .with_context(|| format!("Invalid sanitizer mode: '{}'", selected_value))?
            }
            SettingsMenuType::DeletePermission => {
                config.delete_permission = selected_value
                    .parse()
                    .with_context(|| format!("Invalid delete permission: '{}'", selected_value))?
            }
            SettingsMenuType::HideOriginalEmbed => {
                let hide_setting = selected_value
                    .parse()
                    .with_context(|| format!("Invalid hide embed setting: '{}'", selected_value))?;
                config.hide_original_embed = matches!(hide_setting, HideOriginalEmbed::On);
            }
        }

        cache.update_config(guild_id.get(), config).await?;

        let confirmation_msg = match menu_type {
            SettingsMenuType::SanitizerMode => "✅ Sanitizer Mode updated".to_string(),
            SettingsMenuType::DeletePermission => "✅ Delete Permission updated".to_string(),
            SettingsMenuType::HideOriginalEmbed => "✅ Original Link Preview updated".to_string(),
        };

        let response_data = InteractionResponseDataBuilder::new()
            .content(confirmation_msg)
            .flags(MessageFlags::EPHEMERAL)
            .build();

        let response = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(response_data),
        };

        client
            .interaction(ctx.application_id)
            .create_response(ctx.id, &ctx.token, &response)
            .await?;

        Ok(())
    }

    /// Returns Container embed to be displayed to the user (ComponentsV2).
    fn construct_container(config: &ServerConfig) -> Container {
        ContainerBuilder::new()
            .spoiler(false)
            .component(TextDisplayBuilder::new("## Sanitizer Settings 🛠️").build())
            .component(
                SeparatorBuilder::new()
                    .divider(true)
                    .spacing(SeparatorSpacingSize::Small)
                    .build(),
            )
            .component(TextDisplayBuilder::new("### Sanitizer Mode").build())
            .component(TextDisplayBuilder::new("Change how the bot can be activated.").build())
            .component(
                ActionRowBuilder::new()
                    .component(
                        SelectMenuBuilder::new(
                            SettingsMenuType::SanitizerMode.as_ref(),
                            SelectMenuType::Text,
                        )
                        .max_values(1)
                        .min_values(1)
                        .placeholder("Select Sanitizer Mode")
                        .option(
                            SelectMenuOptionBuilder::new(
                                "Automatic",
                                SanitizerMode::Automatic.as_ref(),
                            )
                            .default(config.sanitizer_mode == SanitizerMode::Automatic)
                            .description("Fix links automatically. (Default)")
                            .emoji(EmojiReactionType::Unicode {
                                name: "🤖".to_string(),
                            })
                            .build(),
                        )
                        .option(
                            SelectMenuOptionBuilder::new(
                                "Manual: Emote",
                                SanitizerMode::ManualEmote.as_ref(),
                            )
                            .default(config.sanitizer_mode == SanitizerMode::ManualEmote)
                            .description("Fix links once a emoji reaction is added.")
                            .emoji(EmojiReactionType::Unicode {
                                name: "🎭".to_string(),
                            })
                            .build(),
                        )
                        .option(
                            SelectMenuOptionBuilder::new(
                                "Manual: Mention",
                                SanitizerMode::ManualMention.as_ref(),
                            )
                            .default(config.sanitizer_mode == SanitizerMode::ManualMention)
                            .description("Fix links by mentioning the bot in a message.")
                            .emoji(EmojiReactionType::Unicode {
                                name: "💬".to_string(),
                            })
                            .build(),
                        )
                        .option(
                            SelectMenuOptionBuilder::new(
                                "Manual: Emote + Mention",
                                SanitizerMode::ManualBoth.as_ref(),
                            )
                            .default(config.sanitizer_mode == SanitizerMode::ManualBoth)
                            .description("Fix links either on emoji reaction or on mention.")
                            .emoji(EmojiReactionType::Unicode {
                                name: "🔁".to_string(),
                            })
                            .build(),
                        )
                        .build(),
                    )
                    .build(),
            )
            .component(
                SeparatorBuilder::new()
                    .divider(true)
                    .spacing(SeparatorSpacingSize::Small)
                    .build(),
            )
            .component(TextDisplayBuilder::new("### Delete Button").build())
            .component(
                TextDisplayBuilder::new(
                    "Change who is allowed to delete the responses of the bot.",
                )
                .build(),
            )
            .component(
                ActionRowBuilder::new()
                    .component(
                        SelectMenuBuilder::new(
                            SettingsMenuType::DeletePermission.as_ref(),
                            SelectMenuType::Text,
                        )
                        .max_values(1)
                        .min_values(1)
                        .placeholder("Select Delete Button Permission")
                        .option(
                            SelectMenuOptionBuilder::new(
                                "Author and Mods",
                                DeletePermission::AuthorAndMods.as_ref(),
                            )
                            .default(config.delete_permission == DeletePermission::AuthorAndMods)
                            .description("Author & users that can Manage Messages. (Default)")
                            .emoji(EmojiReactionType::Unicode {
                                name: "👥".to_string(),
                            })
                            .build(),
                        )
                        .option(
                            SelectMenuOptionBuilder::new(
                                "Everyone",
                                DeletePermission::Everyone.as_ref(),
                            )
                            .default(config.delete_permission == DeletePermission::Everyone)
                            .description("Allow any user to delete the bot response.")
                            .emoji(EmojiReactionType::Unicode {
                                name: "🌐".to_string(),
                            })
                            .build(),
                        )
                        .option(
                            SelectMenuOptionBuilder::new(
                                "Disabled",
                                DeletePermission::Disabled.as_ref(),
                            )
                            .default(config.delete_permission == DeletePermission::Disabled)
                            .description("Disable the delete button feature.")
                            .emoji(EmojiReactionType::Unicode {
                                name: "🚫".to_string(),
                            })
                            .build(),
                        )
                        .build(),
                    )
                    .build(),
            )
            .component(
                SeparatorBuilder::new()
                    .divider(true)
                    .spacing(SeparatorSpacingSize::Small)
                    .build(),
            )
            .component(TextDisplayBuilder::new("### Original link preview").build())
            .component(
                TextDisplayBuilder::new(
                    "Change whether the original message's link preview should be kept or removed.",
                )
                .build(),
            )
            .component(
                ActionRowBuilder::new()
                    .component(
                        SelectMenuBuilder::new(
                            SettingsMenuType::HideOriginalEmbed.as_ref(),
                            SelectMenuType::Text,
                        )
                        .max_values(1)
                        .min_values(1)
                        .placeholder("Select Original Link Preview Visibility")
                        .option(
                            SelectMenuOptionBuilder::new(
                                "Keep original preview",
                                HideOriginalEmbed::Off.as_ref(),
                            )
                            .default(!config.hide_original_embed)
                            .description("Keep the embed of the original message.")
                            .emoji(EmojiReactionType::Unicode {
                                name: "✅".to_string(),
                            })
                            .build(),
                        )
                        .option(
                            SelectMenuOptionBuilder::new(
                                "Remove original preview",
                                HideOriginalEmbed::On.as_ref(),
                            )
                            .default(config.hide_original_embed)
                            .description("Hide the embed of the original message. (Default)")
                            .emoji(EmojiReactionType::Unicode {
                                name: "❌".to_string(),
                            })
                            .build(),
                        )
                        .build(),
                    )
                    .build(),
            )
            .build()
    }
}
