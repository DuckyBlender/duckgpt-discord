use std::collections::HashMap;

use crate::Error;
use crate::ImageModels;

use crate::Context;
use chrono::Utc;
use comfyui_rs::ClientError;
use log::error;
use poise::{serenity_prelude::CreateEmbed, CreateReply};

pub async fn process_image_generation(
    prompt: &str,
    model: &ImageModels,
) -> Result<HashMap<String, Vec<u8>>, ClientError> {
    let client = comfyui_rs::Client::new("127.0.0.1:8188");
    match model {
        ImageModels::SDXLTurbo => {
            let json_prompt =
                serde_json::from_str(include_str!("../jsons/sdxl_turbo_api.json")).unwrap();
            let mut json_prompt: serde_json::Value = json_prompt;
            json_prompt["6"]["inputs"]["text"] = serde_json::Value::String(prompt.to_string());
            json_prompt["13"]["inputs"]["noise_seed"] =
                serde_json::Value::Number(serde_json::Number::from(rand::random::<u64>()));
            let images = client.get_images(json_prompt).await;
            if let Err(e) = images {
                return Err(e);
            }
            return Ok(images.unwrap());
        }
        ImageModels::StableCascade => {
            let json_prompt =
                serde_json::from_str(include_str!("../jsons/stable_cascade_api.json")).unwrap();
            let mut json_prompt: serde_json::Value = json_prompt;
            json_prompt["6"]["inputs"]["text"] = serde_json::Value::String(prompt.to_string());
            json_prompt["3"]["inputs"]["seed"] =
                serde_json::Value::Number(serde_json::Number::from(rand::random::<u64>()));
            json_prompt["33"]["inputs"]["seed"] =
                serde_json::Value::Number(serde_json::Number::from(rand::random::<u64>()));
            let images = client.get_images(json_prompt).await;
            if let Err(e) = images {
                return Err(e);
            }
            return Ok(images.unwrap());
        }
    }
}

pub async fn handle_error(ctx: &Context<'_>, e: ClientError) -> Result<(), Error> {
    let embed = CreateEmbed::default()
        .title("Error generating response")
        .description(format!("Error: {:?}", e))
        .color(0xff0000)
        .timestamp(Utc::now());
    let message = CreateReply::default().embed(embed);
    ctx.send(message).await?;
    error!("Failed to generate response: {:?}", e);
    Ok(())
}
