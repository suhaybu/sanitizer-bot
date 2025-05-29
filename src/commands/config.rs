use anyhow::Result;
use poise::serenity_prelude as serenity;
use tracing::debug;

use crate::Context;
use crate::handlers::db::{DeletePermission, SanitizerMode, ServerConfig};

/// Configure Sanitizer settings for this server 🛠️
#[poise::command(
    slash_command,
    rename = "config",
    default_member_permissions = "MANAGE_GUILD",
    install_context = "Guild",
    interaction_context = "Guild",
    guild_only = true
)]
pub async fn config(ctx: Context<'_>) -> Result<()> {
    debug!("Config command invoked for guild: {:?}", ctx.guild_id());

    let guild_id = match ctx.guild_id() {
        Some(id) => id.get(),
        None => {
            ctx.say("❌ This command can only be used in servers!")
                .await?;
            return Ok(());
        }
    };

    // Get current server config
    let server_config = ServerConfig::get_or_default(guild_id).await?;
    debug!("Current server config: {:?}", server_config);

    // Create embed
    let embed = serenity::CreateEmbed::new()
        .title("Sanitizer Settings 🛠️")
        .description(
            "**Sanitizer mode**\nSwitch between **Automatic** and **Manual** modes.\n\n\
             **Delete button**\nChange who is allowed to use the delete button.\n\n\
             **Hide original embed**\nToggle whether to hide the original message's embed.",
        )
        .color(0x12F2E4); // 1242180 in hex

    // Create sanitizer mode select menu
    let sanitizer_mode_menu = serenity::CreateSelectMenu::new(
        "sanitizer_mode",
        serenity::CreateSelectMenuKind::String {
            options: vec![
                serenity::CreateSelectMenuOption::new("Automatic", "automatic")
                    .description("Bot automatically fixes all compatible links (Default)")
                    .emoji(serenity::ReactionType::Unicode("🤖".to_string()))
                    .default_selection(server_config.sanitizer_mode == SanitizerMode::Automatic),
                serenity::CreateSelectMenuOption::new("Manual (Emote)", "manual_emote")
                    .description("Fix links by reacting with 🫧")
                    .emoji(serenity::ReactionType::Unicode("🫧".to_string()))
                    .default_selection(server_config.sanitizer_mode == SanitizerMode::ManualEmote),
                serenity::CreateSelectMenuOption::new("Manual (Mention)", "manual_mention")
                    .description("Fix links by mentioning @Sanitizer")
                    .emoji(serenity::ReactionType::Unicode("💬".to_string()))
                    .default_selection(
                        server_config.sanitizer_mode == SanitizerMode::ManualMention,
                    ),
                serenity::CreateSelectMenuOption::new(
                    "Manual (Both: Emote + Mention)",
                    "manual_both",
                )
                .description("Fix links using either emote or mention")
                .emoji(serenity::ReactionType::Unicode("🔄".to_string()))
                .default_selection(server_config.sanitizer_mode == SanitizerMode::ManualBoth),
            ],
        },
    )
    .placeholder("Select Sanitizer Mode");

    // Create delete permission select menu
    let delete_permission_menu = serenity::CreateSelectMenu::new(
        "delete_permission",
        serenity::CreateSelectMenuKind::String {
            options: vec![
                serenity::CreateSelectMenuOption::new("Default", "author_and_mods")
                    .description("Author and moderators only (Default)")
                    .emoji(serenity::ReactionType::Unicode("👤".to_string()))
                    .default_selection(
                        server_config.delete_permission == DeletePermission::AuthorAndMods,
                    ),
                serenity::CreateSelectMenuOption::new("Everyone", "everyone")
                    .description("Allow all users to delete")
                    .emoji(serenity::ReactionType::Unicode("🌐".to_string()))
                    .default_selection(
                        server_config.delete_permission == DeletePermission::Everyone,
                    ),
                serenity::CreateSelectMenuOption::new("Disabled", "disabled")
                    .description("Disable delete button feature")
                    .emoji(serenity::ReactionType::Unicode("🚫".to_string()))
                    .default_selection(
                        server_config.delete_permission == DeletePermission::Disabled,
                    ),
            ],
        },
    )
    .placeholder("Select Delete Button Permission");

    // Create hide embed select menu
    let hide_embed_menu = serenity::CreateSelectMenu::new(
        "hide_original_embed",
        serenity::CreateSelectMenuKind::String {
            options: vec![
                serenity::CreateSelectMenuOption::new("On", "hide")
                    .description("Hide original message's embed (Default)")
                    .emoji(serenity::ReactionType::Unicode("✅".to_string()))
                    .default_selection(server_config.hide_original_embed),
                serenity::CreateSelectMenuOption::new("Off", "show")
                    .description("Keep original message's embed visible")
                    .emoji(serenity::ReactionType::Unicode("❌".to_string()))
                    .default_selection(!server_config.hide_original_embed),
            ],
        },
    )
    .placeholder("Select Original Embed Visibility");

    // Create action rows
    let components = vec![
        serenity::CreateActionRow::SelectMenu(sanitizer_mode_menu),
        serenity::CreateActionRow::SelectMenu(delete_permission_menu),
        serenity::CreateActionRow::SelectMenu(hide_embed_menu),
    ];

    // Send the message
    ctx.send(
        poise::CreateReply::default()
            .embed(embed)
            .components(components)
            .ephemeral(true),
    )
    .await?;

    Ok(())
}
