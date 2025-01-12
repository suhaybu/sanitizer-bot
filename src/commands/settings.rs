use crate::Context;
use anyhow::Result;
use poise::serenity_prelude::{
    CreateActionRow, CreateEmbed, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption,
};

/// Display bot settings
#[poise::command(slash_command)]
pub async fn settings(ctx: Context<'_>) -> Result<()> {
    let settings_embed = CreateEmbed::new()
        .title("⚙️")
        .description(
            "**Sanitizer mode**\n\
            Switch between **Automatic** and **Manual** modes.\n\n\
            **Delete button**\n\
            Change who is allowed to use the delete button.\n\n\
            **Hide original embed**\n\
            Toggle whether to hide the original message's embed.",
        )
        .color(0x12F584);

    // Create sanitizer mode options
    let sanitizer_options = vec![
        CreateSelectMenuOption::new("Automatic (Default)", "746243167")
            .description("Bot automatically fixes all compatible links")
            .emoji('🤖'),
        CreateSelectMenuOption::new("Manual (Emote)", "78295750")
            .description("Fix links by reacting with 🫧")
            .emoji('🫧'),
        CreateSelectMenuOption::new("Manual (Mention)", "78295751")
            .description("Fix links by mentioning @Sanitizer")
            .emoji('💬'),
        CreateSelectMenuOption::new("Manual (Both: Emote + Mention)", "78295752")
            .description("Fix links using either emote or mention")
            .emoji('🔄'),
    ];

    // Create delete button options
    let delete_options = vec![
        CreateSelectMenuOption::new("Default", "746243168")
            .description("Author and moderators only (Default)")
            .emoji('👤'),
        CreateSelectMenuOption::new("Everyone", "78295753")
            .description("Allow all users to delete")
            .emoji('🌐'),
        CreateSelectMenuOption::new("Disabled", "78295754")
            .description("Disable delete button feature")
            .emoji('🚫'),
    ];

    // Create embed visibility options
    let visibility_options = vec![
        CreateSelectMenuOption::new("On", "746243169")
            .description("Hide original message's embed (Default)")
            .emoji('✅'),
        CreateSelectMenuOption::new("Off", "78295755")
            .description("Keep original message's embed visible")
            .emoji('❌'),
    ];

    let sanitizer_mode_menu = CreateSelectMenu::new(
        "239204057",
        CreateSelectMenuKind::String {
            options: sanitizer_options,
        },
    )
    .placeholder("Select Sanitizer Mode");

    let delete_button_menu = CreateSelectMenu::new(
        "239204058",
        CreateSelectMenuKind::String {
            options: delete_options,
        },
    )
    .placeholder("Select Delete Button Permission");

    let embed_visibility_menu = CreateSelectMenu::new(
        "239204059",
        CreateSelectMenuKind::String {
            options: visibility_options,
        },
    )
    .placeholder("Select Original Embed Visibility");

    let menu_rows = vec![
        CreateActionRow::SelectMenu(sanitizer_mode_menu),
        CreateActionRow::SelectMenu(delete_button_menu),
        CreateActionRow::SelectMenu(embed_visibility_menu),
    ];

    let builder = poise::CreateReply::default()
        .embed(settings_embed)
        .components(menu_rows)
        .ephemeral(true);

    ctx.send(builder).await?;

    Ok(())
}
