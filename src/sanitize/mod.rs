mod core;
mod messages;

pub use core::{UrlProcessor, contains_url};
pub use messages::{add_emote, is_bot_mentioned, process_message};
