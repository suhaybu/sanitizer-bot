//! Credit Command: Presents the user with a list of acknowledgments.

use twilight_http::Client;
use twilight_model::{
    application::{
        command::{Command, CommandType},
        interaction::{Interaction, InteractionContextType},
    },
    channel::message::MessageFlags,
    http::interaction::{InteractionResponse, InteractionResponseType},
    oauth::ApplicationIntegrationType,
};
use twilight_util::builder::{
    InteractionResponseDataBuilder, command::CommandBuilder, embed::EmbedBuilder,
};

pub struct HelpCommand;

impl HelpCommand {
    /// Creates /help command.
    pub fn create_command() -> Command {
        CommandBuilder::new("help", "Learn more about me 📖", CommandType::ChatInput)
            .contexts([
                InteractionContextType::Guild,
                InteractionContextType::BotDm,
                InteractionContextType::PrivateChannel,
            ])
            .integration_types([ApplicationIntegrationType::GuildInstall])
            .build()
    }

    /// Handles responding to command invocation.
    pub async fn handle(ctx: &Interaction, client: &Client) -> anyhow::Result<()> {
        let embed = EmbedBuilder::new().title("Credits 🎺")
         .description("These are all the super cool projects that I rely on:\n\
        -  **Twitter**: Thanks to FixTweet's reliable [FxTwitter](https://github.com/FixTweet/FxTwitter) project\n\
        -  **TikTok & Instagram**: Thanks to [kkScript](https://kkscript.com/)\n\
        -  **Instagram** (Fallback): Powered by the awesome [InstaFix](https://github.com/Wikidepia/InstaFix) project\n\
        -  **Twitch**: Thanks to the [FxTwitch](https://github.com/seriaati/fxtwitch) project\n\
        -  **Reddit**: Thanks to the [vxReddit](https://github.com/dylanpdx/vxReddit) project\n\
        -# The code that powers me is publicly sourced [here](https://github.com/suhaybu/sanitizer-bot) on GitHub along with the [Privacy policy](https://github.com/suhaybu/sanitizer-bot/blob/master/PRIVACY.md).\n\
            ")
            .build();

        let data = InteractionResponseDataBuilder::new()
            .embeds([embed])
            .flags(MessageFlags::EPHEMERAL)
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
}
