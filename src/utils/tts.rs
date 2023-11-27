use std::time::Instant;

use serde::Serialize;
use serenity::builder::{CreateAttachment, CreateEmbedFooter, CreateMessage};
use serenity::prelude::*;
use serenity::{builder::CreateEmbed, model::channel::Message};
use tracing::{debug, error};

use crate::constants::{FOOTER_TEXT, SUCCESS_COLOR};

#[derive(Serialize)]
pub struct SpeechRequest {
    pub model: String,
    pub input: String,
    pub voice: String,
}

pub async fn handle_tts(ctx: &Context, msg: Message, voice: &str) {
    let openai_token = std::env::var("OPENAI_TOKEN").expect("OPENAI_TOKEN not set");

    // Get the message content
    let message_text = msg.content.clone();
    // Send the request to the TTS server

    // Get the message
    let message = msg.content.clone();
    // Send the request to the TTS API
    let client = reqwest::Client::new();
    let speech_request = SpeechRequest {
        model: "tts-1".to_string(),
        input: message,
        voice: voice.to_string(),
    };

    let now = Instant::now();
    let response = client
        .post("https://api.openai.com/v1/audio/speech")
        .header("Authorization", format!("Bearer {}", openai_token))
        .header("Content-Type", "application/json")
        .json(&speech_request)
        .send()
        .await
        .unwrap(); // todo: handle errors

    debug!("Request took {}s", now.elapsed().as_secs_f32());
    let elapsed = now.elapsed().as_secs_f32();

    let cost = calculate_cost(&message_text);

    // Check if the message is too long, if it is just don't send it
    let show_msg: bool = if message_text.len() > 1000 {
        false
    } else {
        true
    };
    let message_text = if show_msg {
        message_text
    } else {
        "Response too long to display".to_string()
    };
    // The response is a file, so we need to get the bytes
    let bytes = response.bytes().await.unwrap();

    // Send the bytes to the channel
    let footer = CreateEmbedFooter::new(FOOTER_TEXT);

    let embed = CreateEmbed::default()
        .title(format!("{}'s message", msg.author.name))
        .color(SUCCESS_COLOR)
        .field("Voice", voice, false)
        .field("Message", format!("```\n{}\n```", &message_text), false)
        .field("Time", format!("{:.2} seconds", elapsed), true)
        .field("Cost", format!("${:.4}", cost), true)
        .footer(footer);

    let builder = CreateMessage::new()
        .reference_message(&msg)
        .add_file(CreateAttachment::bytes(
            bytes.as_ref(),
            format!("{}.mp3", msg.id).as_str(),
        ))
        .embed(embed);

    let send_result = msg.channel_id.send_message(&ctx.http, builder).await;

    // Check if the message was sent successfully and handle any errors
    if let Err(why) = send_result {
        error!("Error sending message: {:?}", why);
    }

    return;
}

fn calculate_cost(message: &str) -> f64 {
    let message_length = message.len();
    let cost_per_character: f64 = 0.015 / 1000.0;
    let cost = message_length as f64 * cost_per_character;
    return cost;
}
