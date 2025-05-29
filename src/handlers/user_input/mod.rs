mod external_api;
mod parse_url;
mod sanitize_input;

pub use self::sanitize_input::sanitize_input;
pub use parse_url::ParsedURL;

// Checks if context is Guild Install
pub fn is_guild_install(ctx: &crate::Context<'_>) -> bool {
    ctx.interaction
        .authorizing_integration_owners
        .0
        .iter()
        .any(|owner| {
            matches!(
                owner,
                poise::serenity_prelude::AuthorizingIntegrationOwner::GuildInstall(_)
            )
        })
}
