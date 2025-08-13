use std::time::Duration;
use tokio::time::sleep;
use tracing::debug;

use twilight_http::Client;
use twilight_model::channel::Message;

// use crate::Result; // No crate-wide Result type alias is defined

// Logic used to validate if bot's response is valid
fn check_bot_response(bot_message: &Message) -> bool {
    debug!("Checking bot response for message ID: {}", bot_message.id);

    if bot_message.embeds.is_empty() {
        debug!("No embeds found in message");
        return false;
    }

    let f_embed = &bot_message.embeds[0];
    debug!("First embed: {:?}", f_embed);

    if f_embed.video.is_some() {
        debug!("Video embed found - valid response");
        return true;
    }

    match &bot_message.content {
        content if content.contains("fxtwitter.com") => {
            let valid = f_embed.description.as_deref() != Some("Sorry, that post doesn't exist :(");
            debug!(
                "Twitter response: valid={}, description={:?}",
                valid, f_embed.description
            );
            valid
        }
        content if content.contains("ddinstagram.com") => {
            let valid = f_embed.description.as_deref() != Some("Post might not be available");
            debug!(
                "Instagram response: valid={}, description={:?}",
                valid, f_embed.description
            );
            valid
        }
        _ => {
            debug!("Unknown platform - defaulting to valid");
            true
        }
    }
}

pub async fn wait_for_message_embed(
    http: &Client,
    message_id: twilight_model::id::Id<twilight_model::id::marker::MessageMarker>,
    channel_id: twilight_model::id::Id<twilight_model::id::marker::ChannelMarker>,
) -> Option<Message> {
    if let Ok(resp) = http.message(channel_id, message_id).await {
        if let Ok(msg) = resp.model().await {
            if check_bot_response(&msg) {
                return Some(msg);
            }
        }
    }

    let timeout = Duration::from_secs(8);
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        if let Ok(resp) = http.message(channel_id, message_id).await {
            if let Ok(msg) = resp.model().await {
                if check_bot_response(&msg) {
                    return Some(msg);
                }
            }
        }
        sleep(Duration::from_millis(500)).await;
    }

    None
}
