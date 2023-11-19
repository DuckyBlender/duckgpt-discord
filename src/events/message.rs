use serenity::prelude::*;
use serenity::model::channel::Message;
use tracing::info;
use crate::utils::{tts::handle_tts, vision::handle_vision};
use crate::constants::*;

pub async fn handle(ctx: &Context, msg: Message) {
    // Check if the message is in a TTS channel
    if let Some(voice) = get_tts_voice_from_channel_id(msg.channel_id.0) {
        handle_tts(ctx, msg, voice).await;
        return;
    }

    // Check if the message is in a vision channel
    if is_vision_channel(msg.channel_id.0) {
        handle_vision(ctx, msg).await;
        return;
    }

    // Other messages are ignored
    info!("Ignoring message in channel {}", msg.channel_id.0);
}

fn get_tts_voice_from_channel_id(channel_id: u64) -> Option<&'static str> {
    match channel_id {
        ALLOY_CHANNEL_ID => Some("alloy"),
        ECHO_CHANNEL_ID => Some("echo"),
        FABLE_CHANNEL_ID => Some("fable"),
        ONYX_CHANNEL_ID => Some("onyx"),
        NOVA_CHANNEL_ID => Some("nova"),
        SHIMMER_CHANNEL_ID => Some("shimmer"),
        _ => None,
    }
}

fn is_vision_channel(channel_id: u64) -> bool {
    channel_id == LOW_QUALITY_CHANNEL_ID || channel_id == HIGH_QUALITY_CHANNEL_ID
}