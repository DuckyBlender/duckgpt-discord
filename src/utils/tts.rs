use std::time::Instant;

use serde::Serialize;
use serenity::model::channel::Message;
use serenity::prelude::*;
use tracing::{debug, error};

use crate::constants::{SUCCESS_COLOR, FOOTER_TEXT};

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
    let embed_result = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(format!("TTS from {}", msg.author.name))
                    .color(SUCCESS_COLOR)
                    .field("Voice", voice, false)
                    .field("Message", format!("```\n{}\n```", &message_text), false)
                    .field("Time", format!("{:.2} seconds", elapsed), true)
                    .field(
                        "Cost",
                        format!("${:.4}", cost),
                        true,
                    )
                    .footer(|f| f.text(FOOTER_TEXT))
            })
            .add_file((bytes.as_ref(), format!("{}.mp3", msg.id).as_str()))
        })
        .await;

    // Check if the message was sent successfully and handle any errors
    if let Err(why) = embed_result {
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
