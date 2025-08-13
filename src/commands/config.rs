use twilight_http::Client;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::application::interaction::{application_command::CommandData, Interaction};
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_util::builder::{
    embed::EmbedBuilder,
    InteractionResponseDataBuilder,
};
use twilight_model::channel::message::component::{
    ActionRow, Component, SelectMenu, SelectMenuOption,
};

use crate::handlers::db::{DeletePermission, SanitizerMode, ServerConfig};

/// Configure Sanitizer settings for this server 🛠️
#[derive(CommandModel, CreateCommand, Debug)]
#[command(name = "config", desc = "Configure Sanitizer settings for this server")]
pub struct ConfigCommand;

impl ConfigCommand {
    pub async fn handle(
        interaction: Interaction,
        _data: CommandData,
        client: &Client,
    ) -> anyhow::Result<()> {
        let guild_id = match interaction.guild_id {
            Some(id) => id.get(),
            None => {
                let data = InteractionResponseDataBuilder::new()
                    .content("❌ This command can only be used in servers!")
                    .flags(twilight_model::channel::message::MessageFlags::EPHEMERAL)
                    .build();
                let response = InteractionResponse {
                    kind: InteractionResponseType::ChannelMessageWithSource,
                    data: Some(data),
                };
                client
                    .interaction(interaction.application_id)
                    .create_response(interaction.id, &interaction.token, &response)
                    .await?;
                return Ok(());
            }
        };

        let server_config = ServerConfig::get_or_default(guild_id).await?;

        let embed = EmbedBuilder::new()
            .title("Sanitizer Settings 🛠️")
            .description("**Sanitizer mode**\nSwitch between **Automatic** and **Manual** modes.\n\n\
             **Delete Permission (Coming soon)**\nChange who is allowed to use the delete button.\n\n\
             **Keep or Hide original embed**\nToggle whether the original message's embed \n (media preview) should be hidden or kept.")
            .color(0x12F2E4)
            .build();

        // Select menus
        let sanitizer_options = [
            ("Mode: Automatic", "automatic", server_config.sanitizer_mode == SanitizerMode::Automatic),
            ("Mode: Manual (Emote)", "manual_emote", server_config.sanitizer_mode == SanitizerMode::ManualEmote),
            ("Mode: Manual (Mention)", "manual_mention", server_config.sanitizer_mode == SanitizerMode::ManualMention),
            ("Mode: Manual (Both: Emote + Mention)", "manual_both", server_config.sanitizer_mode == SanitizerMode::ManualBoth),
        ];
        let sanitizer_menu = Component::SelectMenu(SelectMenu {
            custom_id: "sanitizer_mode".into(),
            placeholder: Some("Select Sanitizer Mode".into()),
            min_values: Some(1),
            max_values: Some(1),
            disabled: false,
            options: sanitizer_options
                .iter()
                .map(|(label, value, selected)| SelectMenuOption {
                    default: *selected,
                    description: None,
                    emoji: None,
                    label: (*label).into(),
                    value: (*value).into(),
                })
                .collect(),
        });

        let delete_options = [
            ("Delete Permission: Author and Mods", "author_and_mods", server_config.delete_permission == DeletePermission::AuthorAndMods),
            ("Delete Permission: Everyone", "everyone", server_config.delete_permission == DeletePermission::Everyone),
            ("Delete Permission: Disabled", "disabled", server_config.delete_permission == DeletePermission::Disabled),
        ];
        let delete_menu = Component::SelectMenu(SelectMenu {
            custom_id: "delete_permission".into(),
            placeholder: Some("Delete Button - Coming Soon!".into()),
            min_values: Some(1),
            max_values: Some(1),
            disabled: true,
            options: delete_options
                .iter()
                .map(|(label, value, selected)| SelectMenuOption {
                    default: *selected,
                    description: None,
                    emoji: None,
                    label: (*label).into(),
                    value: (*value).into(),
                })
                .collect(),
        });

        let hide_options = [
            ("Hide Original Embeds", "hide", server_config.hide_original_embed),
            ("Keep Original Embeds", "show", !server_config.hide_original_embed),
        ];
        let hide_menu = Component::SelectMenu(SelectMenu {
            custom_id: "hide_original_embed".into(),
            placeholder: Some("Select Original Embed Visibility".into()),
            min_values: Some(1),
            max_values: Some(1),
            disabled: false,
            options: hide_options
                .iter()
                .map(|(label, value, selected)| SelectMenuOption {
                    default: *selected,
                    description: None,
                    emoji: None,
                    label: (*label).into(),
                    value: (*value).into(),
                })
                .collect(),
        });

        let components: Vec<Component> = vec![
            Component::ActionRow(ActionRow {
                components: vec![sanitizer_menu],
            }),
            Component::ActionRow(ActionRow {
                components: vec![delete_menu],
            }),
            Component::ActionRow(ActionRow {
                components: vec![hide_menu],
            }),
        ];

        let data = InteractionResponseDataBuilder::new()
            .embeds([embed])
            .components(components)
            .flags(twilight_model::channel::message::MessageFlags::EPHEMERAL)
            .build();

        let response = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(data),
        };
        client
            .interaction(interaction.application_id)
            .create_response(interaction.id, &interaction.token, &response)
            .await?;

        // Save default config so future updates persist
        let _ = server_config.save().await;

        Ok(())
    }
}
