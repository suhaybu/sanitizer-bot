use twilight_http::Client;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::application::interaction::{application_command::CommandData, Interaction};
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_util::builder::{embed::EmbedBuilder, InteractionResponseDataBuilder};

/// Roll the credits! 🎺
#[derive(CommandModel, CreateCommand, Debug)]
#[command(name = "credits", desc = "Show credits and acknowledgements")]
pub struct CreditsCommand;

impl CreditsCommand {
    pub async fn handle(
        interaction: Interaction,
        _data: CommandData,
        client: &Client,
    ) -> anyhow::Result<()> {
        let embed = EmbedBuilder::new()
            .title("Credits")
            .description("These are all the super cool projects I rely on:\n\
        -  **Twitter**: Thanks to FixTweet's reliable [FxTwitter](https://github.com/FixTweet/FxTwitter) project\n\
        -  **TikTok & Instagram**: Thanks to [QuickVids](https://quickvids.app/) super fast and easy to use API\n\
        -  **Instagram** (Fallback): Powered by the awesome [InstaFix](https://github.com/Wikidepia/InstaFix)\n\
        -# The code for this bot is public sourced on my GitHub [here](https://github.com/suhaybu/sanitizer-bot).")
            .build();

        let data = InteractionResponseDataBuilder::new().embeds([embed]).build();
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
