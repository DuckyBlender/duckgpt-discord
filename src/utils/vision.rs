use std::time::Instant;

use reqwest::header::{HeaderMap, CONTENT_TYPE, HeaderValue, AUTHORIZATION};
use serenity::model::channel::Message;
use serenity::prelude::*;
use tracing::{debug, error};

use crate::constants::*;
use crate::structs::*;

pub async fn handle_vision(ctx: &Context, msg: Message) {
    let quality = match msg.channel_id {
        serenity::model::id::ChannelId(LOW_QUALITY_CHANNEL_ID) => "low",
        serenity::model::id::ChannelId(HIGH_QUALITY_CHANNEL_ID) => "high",
        _ => unreachable!(),
    };

    // Ignore messages that don't contain an attachment or URL
    let attachment_count = msg.attachments.len();
    debug!("Attachment count: {}", attachment_count);
    if attachment_count == 0 {
        debug!("Ignoring message without attachment");
        return;
    }

    // Check if the attachment is an image
    let file = if attachment_count > 0 {
        let file = msg.attachments.first().unwrap(); // safe to unwrap
        if !ALLOWED_EXTENSIONS
            .iter()
            .any(|&x| file.filename.ends_with(x))
        {
            // reply with an error message
            msg
                .reply(
                    ctx,
                    format!(
                        "Invalid file type ({})! Supported file types: {}",
                        &file.filename.as_str(),
                        ALLOWED_EXTENSIONS.join(", ")
                    ),
                )
                .await
                .unwrap();
            return;
        }
        Some(file)
    } else {
        None
    };

    // great, now we have an image or URL
    // now get the text of the message
    let message_text = msg.content.clone(); // this is without the attachment or URL

    let openai_token = std::env::var("OPENAI_TOKEN").expect("OPENAI_TOKEN not set");

    let text = if message_text.is_empty() {
        debug!("Message text is empty, using default");
        "What is in this image?".to_string()
    } else {
        debug!("Prompt: {}", message_text);
        message_text
    };

    let chat_completion_request = ChatCompletionRequest {
        model: "gpt-4-vision-preview".to_string(),
        messages: vec![UserMessage {
            role: "user".to_string(),
            content: vec![
                Content {
                    content_type: "text".to_string(),
                    text: text.clone().into(),
                    image_url: None,
                },
                // TODO: Add support for multiple images
                Content {
                    content_type: "image_url".to_string(),
                    text: None,
                    image_url: file.map(|f| ImageUrl {
                        url: f.url.clone(),
                        detail: quality.to_string(),
                    }),
                },
            ],
        }],
        max_tokens: MAX_TOKENS,
    };

    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", openai_token)).unwrap(),
    );

    let now = Instant::now();
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .headers(headers)
        .json(&chat_completion_request)
        .send()
        .await;

    debug!("Request took {}ms", now.elapsed().as_millis());
    let elapsed = now.elapsed().as_millis();

    match response {
        // SUCCESSFUL RESPONSE
        Ok(response) if response.status().is_success() => {
            // Prase the string of data into serde_json::Value.
            let v: serde_json::Value = response.json().await.unwrap();

            // Access the nested total_tokens value
            let input_tokens = v["usage"]["prompt_tokens"].as_u64().expect(
                format!("prompt_tokens should be a u64\nfull response: \n\n{:?}", v).as_str(),
            );
            let output_tokens = v["usage"]["completion_tokens"].as_u64().expect(
                format!(
                    "completion_tokens should be a u64\nfull response: \n\n{:?}",
                    v
                )
                .as_str(),
            );
            let reply = v["choices"][0]["message"]["content"]
                .as_str()
                .expect(format!("content should be a string\nfull response: \n\n{:?}", v).as_str());
            let (height, width) = if let Some(file) = &file {
                (file.height.unwrap(), file.width.unwrap())
            } else {
                unreachable!()
            };

            let total_cost = convert_tokens_to_cost(
                input_tokens as u32,
                output_tokens as u32,
                width as u32,
                height as u32,
                quality,
            );

            // Split the reply into chunks of 1000 characters
            const MAX_EMBED_FIELD_VALUE_LEN: usize = 1000;
            let reply_chunks: Vec<String> = reply
                .chars()
                .collect::<Vec<char>>()
                .chunks(MAX_EMBED_FIELD_VALUE_LEN)
                .map(|chunk| chunk.iter().collect::<String>())
                .collect();

            // Send each chunk as a separate embed field
            for (i, chunk) in reply_chunks.iter().enumerate() {
                let title = if reply_chunks.len() > 1 {
                    format!(
                        "Image Analysis Result ({} of {})",
                        i + 1,
                        reply_chunks.len()
                    )
                } else {
                    "Image Analysis Result".to_string()
                };

                let embed_result = msg
                    .channel_id
                    .send_message(&ctx.http, |m| {
                        m.embed(|e| {
                            e.title(&title)
                                .description(format!(
                                    "Analysis for the submitted image in {} quality.",
                                    quality
                                ))
                                .image(file.map(|f| f.url.as_str()).unwrap_or(""))
                                .fields(vec![
                                    ("Prompt", text.clone(), false),
                                    ("Response", format!("```\n{}\n```", chunk), false),
                                ])
                                .field(
                                    "Analysis Time",
                                    format!("{:.2} seconds", elapsed as f64 / 1000.0),
                                    true,
                                )
                                .field("Estimated Cost", format!("${:.4}", total_cost), true)
                                .footer(|f| f.text("Powered by OpenAI | Created by @DuckyBlender"))
                        })
                    })
                    .await;

                // Check if the message was sent successfully and handle any errors
                if let Err(why) = embed_result {
                    error!("Error sending message: {:?}", why);
                    // send a reply to the user
                    msg
                        .reply(
                            ctx.clone(),
                            format!(
                                "Error sending message: {:?}\n\n`Cost: ${:.2}`",
                                why, total_cost
                            ),
                        )
                        .await
                        .unwrap();
                }
            }
        }
        // NON SUCCESSFUL RESPONSE
        Ok(response) => {
            let error_value: serde_json::Value = response.json().await.unwrap_or_else(|_| {
                serde_json::json!({
                    "error": {
                        "message": "Failed to parse error response from OpenAI API."
                    }
                })
            });
            let error_message = error_value["error"]["message"]
                .as_str()
                .unwrap_or("Unknown error occurred.");
            error!("Error from OpenAI API: {}", error_message);

            // Form the embed error message
            let embed_result = msg
                .channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.title("Error")
                            .description(format!("Error from OpenAI API: {}", error_message))
                            .footer(|f| f.text("Powered by OpenAI | Created by @DuckyBlender"))
                    })
                })
                .await;

            // Check if the message was sent successfully and handle any errors
            if let Err(why) = embed_result {
                error!("Error sending message: {:?}", why);
            }
        }
        // REQUEST ERROR
        Err(error) => {
            error!("Error sending request to OpenAI API: {:?}", error);

            // Form the embed error message
            let embed_result = msg
                .channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.title("Error")
                            .description(
                                "Error communicating with OpenAI API. Please try again later.",
                            )
                            .footer(|f| f.text("Powered by OpenAI | Created by @DuckyBlender"))
                    })
                })
                .await;

            // Check if the message was sent successfully and handle any errors
            if let Err(why) = embed_result {
                error!("Error sending message: {:?}", why);
            }
        }
    }
}

pub fn calculate_image_token_cost(width: u32, height: u32, detail: &str) -> u32 {
    const LOW_DETAIL_COST: u32 = 85;
    const HIGH_DETAIL_COST_PER_TILE: u32 = 170;
    const ADDITIONAL_COST: u32 = 85;
    const MAX_DIMENSION: u32 = 2048;
    const SCALE_TO: u32 = 768;
    const TILE_SIZE: u32 = 512;

    match detail {
        "low" => LOW_DETAIL_COST,
        "high" => {
            // Scale the image if either dimension is larger than the maximum allowed.
            let (scaled_width, scaled_height) = if width > MAX_DIMENSION || height > MAX_DIMENSION {
                let aspect_ratio = width as f32 / height as f32;
                if width > height {
                    (
                        MAX_DIMENSION,
                        (MAX_DIMENSION as f32 / aspect_ratio).round() as u32,
                    )
                } else {
                    (
                        (MAX_DIMENSION as f32 * aspect_ratio).round() as u32,
                        MAX_DIMENSION,
                    )
                }
            } else {
                (width, height)
            };

            // Further scale the image so that the shortest side is 768 pixels long.
            let (final_width, final_height) = {
                let aspect_ratio = scaled_width as f32 / scaled_height as f32;
                if scaled_width < scaled_height {
                    (SCALE_TO, (SCALE_TO as f32 / aspect_ratio).round() as u32)
                } else {
                    ((SCALE_TO as f32 * aspect_ratio).round() as u32, SCALE_TO)
                }
            };

            // Calculate the number of 512px tiles needed.
            let tiles_across = (final_width as f32 / TILE_SIZE as f32).ceil() as u32;
            let tiles_down = (final_height as f32 / TILE_SIZE as f32).ceil() as u32;
            let total_tiles = tiles_across * tiles_down;

            // Calculate the final token cost.
            total_tiles * HIGH_DETAIL_COST_PER_TILE + ADDITIONAL_COST
        }
        _ => panic!("Invalid detail level: {}", detail),
    }
}

pub fn convert_tokens_to_cost(
    input_tokens: u32,
    output_tokens: u32,
    width: u32,
    height: u32,
    detail_level: &str,
) -> f64 {
    const COST_PER_INPUT_TOKEN: f64 = 0.01 / 1000.0;
    const COST_PER_OUTPUT_TOKEN: f64 = 0.03 / 1000.0;
    let input_cost = input_tokens as f64 * COST_PER_INPUT_TOKEN;
    let output_cost = output_tokens as f64 * COST_PER_OUTPUT_TOKEN;
    let image_cost =
        calculate_image_token_cost(width, height, detail_level) as f64 * COST_PER_OUTPUT_TOKEN;

    // Calculate the total cost
    let total_cost = input_cost + output_cost + image_cost;

    total_cost
}
