use serenity::prelude::*;
use serenity::model::channel::Message;
use tracing::info;

pub async fn handle_tts(ctx: &Context, msg: Message, voice: &str) {
    info!("TTS channel message");
    msg.reply(ctx, format!("TTS channel message in voice {}", voice)).await.unwrap();
}