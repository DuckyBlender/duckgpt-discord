// DEV NOTE: This file uses a different json parsing system then the rest. Here every JSON is a struct, and the structs are defined in this file.
// Currently experimenting which one is better.

use std::time::Instant;

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serenity::{
    all::ChannelId,
    builder::{CreateEmbed, CreateEmbedFooter, CreateMessage},
    model::channel::Message,
    prelude::*,
};
use tracing::error;

use crate::constants::{
    DALLE2_CHANNEL_ID, DALLE3_CHANNEL_ID, DALLE3_HD_CHANNEL_ID, ERROR_COLOR, FOOTER_TEXT,
    SUCCESS_COLOR,
};

pub async fn handle_image(ctx: &Context, msg: Message) {
    let openai_token = std::env::var("OPENAI_TOKEN").expect("OPENAI_TOKEN not set");

    // Check which channel the message was sent in (DALL-E 2 or DALL-E 3)
    let channel_id = msg.channel_id;
    let quality = if channel_id == DALLE3_HD_CHANNEL_ID {
        Quality::Hd
    } else {
        Quality::Standard
    };

    let dalle_2_channel = ChannelId::new(DALLE2_CHANNEL_ID);
    let dalle_3_channel = ChannelId::new(DALLE3_CHANNEL_ID);
    let dalle_3_hd_channel = ChannelId::new(DALLE3_HD_CHANNEL_ID);

    let model = match channel_id {
        // no idea why this "ref" thing is needed, but it is
        // todo: make this less ugly
        ref id if *id == dalle_2_channel => Model::DallE2,
        ref id if *id == dalle_3_channel => Model::DallE3,
        ref id if *id == dalle_3_hd_channel => Model::DallE3,
        _ => panic!("Invalid channel ID"),
    };

    // Get the message
    let message = msg.content.clone();

    // Send the request to the DALLE API
    let client = reqwest::Client::new();

    // Set the headers
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", openai_token)).unwrap(),
    );

    // Create the request body
    let request_body = RequestBody {
        prompt: message.clone(),
        model: Some(model),
        n: Some(1),
        quality: Some(quality),
        response_format: Some(ResponseFormat::Url),
        size: Some(Size::Size1024x1024),
        style: Some(Style::Vivid),
    };

    let now = Instant::now();

    // Make the request
    let response = client
        .post("https://api.openai.com/v1/images/generations")
        .headers(headers)
        .json(&request_body)
        .send()
        .await;

    let elapsed = now.elapsed().as_secs_f32();

    // Check if the request was successful
    match response {
        Ok(response) => {
            if response.status().is_success() {
                // The request was successful, parse the response
                let response = response.json::<ResponseBody>().await.unwrap();
                // Get the image URL
                let image_url = response.data[0].url.clone();

                // if its dalle 3 and hd, set the model to &str "dall-e-3-hd"
                let model_friendly_name = match model {
                    Model::DallE2 => "DALL·E 2",
                    Model::DallE3 => match quality {
                        Quality::Standard => "DALL·E 3",
                        Quality::Hd => "DALL·E 3 HD",
                    },
                };

                let show_msg: bool = if message.len() > 1000 { false } else { true };
                let message_text = if show_msg {
                    message
                } else {
                    "Response too long to display".to_string()
                };

                // Send the image URL to the channel
                let footer = CreateEmbedFooter::new(FOOTER_TEXT);
                let embed = CreateEmbed::default()
                    .title(format!("Image from {}", msg.author.name))
                    .color(SUCCESS_COLOR)
                    .field(
                        "Message",
                        format!("```\n{}\n```", message_text.clone()),
                        false,
                    )
                    .field("Time", format!("{:.2} seconds", elapsed), true)
                    .field(
                        "Cost",
                        format!(
                            "${:.2}",
                            calculate_cost(model, Some(quality), Size::Size1024x1024, 1)
                        ),
                        true,
                    )
                    .field("Model", format!("**{:?}**", model_friendly_name), true)
                    .image(image_url)
                    .footer(footer);

                let builder = CreateMessage::new().reference_message(&msg).embed(embed);

                let send_result = msg.channel_id.send_message(&ctx.http, builder).await;

                // Check if the message was sent successfully
                match send_result {
                    Ok(_) => {
                        // The message was sent successfully
                    }
                    Err(e) => {
                        // There was an error sending the message
                        error!("Error sending message: {}", e);
                    }
                }
            } else {
                // The request was not successful, handle the error response
                let error_json = response.text().await.unwrap();
                let error_response = serde_json::from_str::<ErrorResponse>(&error_json);

                match error_response {
                    Ok(error) => {
                        // Inform the user what went wrong using an embed
                        let footer = CreateEmbedFooter::new(FOOTER_TEXT);
                        let embed = CreateEmbed::default()
                            .title("Error")
                            .color(ERROR_COLOR)
                            .field("Error Code", format!("`{}`", &error.error.code), true)
                            .field("Error Message", format!("`{}`", &error.error.message), true)
                            .footer(footer);

                        let builder = CreateMessage::new().reference_message(&msg).embed(embed);

                        let send_result = msg.channel_id.send_message(&ctx.http, builder).await;

                        // Check if the message was sent successfully
                        match send_result {
                            Ok(_) => {
                                // The message was sent successfully
                            }
                            Err(e) => {
                                // There was an error sending the message
                                error!("Error sending error message: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        // There was an error parsing the error response
                        error!("Error parsing error response: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            // There was an error making the request
            error!("Error making request: {}", e);
            // Inform the user that there was an error making the request
            let footer = CreateEmbedFooter::new(FOOTER_TEXT);

            let embed = CreateEmbed::default()
                .title("Error")
                .color(ERROR_COLOR)
                .description("An error occurred while making the request to the DALL-E API.")
                .field("Error Details", format!("{}", e), false)
                .footer(footer);

            let builder = CreateMessage::new().reference_message(&msg).embed(embed);

            let send_result = msg.channel_id.send_message(&ctx.http, builder).await;

            // Check if the message was sent successfully
            match send_result {
                Ok(_) => {
                    // The message was sent successfully
                }
                Err(e) => {
                    // There was an error sending the message
                    error!("Error sending error message: {}", e);
                }
            }
        }
    };
}

fn calculate_cost(
    model: Model,
    quality: Option<Quality>,
    resolution: Size,
    number_of_images: u32,
) -> f64 {
    let price_per_image = match model {
        Model::DallE3 => match quality {
            Some(Quality::Standard) => match resolution {
                Size::Size1024x1024 => 0.040,
                Size::Size1024x1792 | Size::Size1792x1024 => 0.080,
                _ => panic!("Invalid resolution for DALL·E 3 Standard quality"),
            },
            Some(Quality::Hd) => match resolution {
                Size::Size1024x1024 => 0.080,
                Size::Size1024x1792 | Size::Size1792x1024 => 0.120,
                _ => panic!("Invalid resolution for DALL·E 3 HD quality"),
            },
            None => panic!("Quality must be specified for DALL·E 3"),
        },
        Model::DallE2 => match resolution {
            Size::Size1024x1024 => 0.020,
            Size::Size512x512 => 0.018,
            Size::Size256x256 => 0.016,
            _ => panic!("Invalid resolution for DALL·E 2"),
        },
    };

    price_per_image * number_of_images as f64
}

/// REQUEST BODY

// Define the possible values for the `model` field.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Model {
    #[serde(rename = "dall-e-2")]
    DallE2,
    #[serde(rename = "dall-e-3")]
    DallE3,
}

// Define the possible values for the `quality` field.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Quality {
    Standard,
    Hd,
}

// Define the possible values for the `response_format` field.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseFormat {
    Url,
    B64Json,
}

// Define the possible values for the `size` field.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Size {
    #[serde(rename = "256x256")]
    Size256x256,
    #[serde(rename = "512x512")]
    Size512x512,
    #[serde(rename = "1024x1024")]
    Size1024x1024,
    #[serde(rename = "1792x1024")]
    Size1792x1024,
    #[serde(rename = "1024x1792")]
    Size1024x1792,
}

// Define the possible values for the `style` field.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Style {
    Vivid,
    Natural,
}

// Define the main `RequestBody` struct.
#[derive(Debug, Serialize, Deserialize)]
pub struct RequestBody {
    pub prompt: String,
    #[serde(default = "default_model")]
    pub model: Option<Model>,
    #[serde(default = "default_n")]
    pub n: Option<u8>,
    #[serde(default = "default_quality")]
    pub quality: Option<Quality>,
    #[serde(default = "default_response_format")]
    pub response_format: Option<ResponseFormat>,
    #[serde(default = "default_size")]
    pub size: Option<Size>,
    #[serde(default = "default_style")]
    pub style: Option<Style>,
    // pub user: Option<String>,
}

// Provide default values for optional fields.
fn default_model() -> Option<Model> {
    Some(Model::DallE2)
}

fn default_n() -> Option<u8> {
    Some(1)
}

fn default_quality() -> Option<Quality> {
    Some(Quality::Standard)
}

fn default_response_format() -> Option<ResponseFormat> {
    Some(ResponseFormat::Url)
}

fn default_size() -> Option<Size> {
    Some(Size::Size1024x1024)
}

fn default_style() -> Option<Style> {
    Some(Style::Vivid)
}

/// RESPONSE BODY

// Define the `Image` struct to represent each image data with a URL.
#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    pub url: String,
}

// Define the main `ResponseBody` struct.
#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseBody {
    pub created: u64,
    pub data: Vec<Image>,
}

/// ERROR STRUCTS

// Define the `ErrorDetail` struct to represent the error details.
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    pub param: Option<String>,
    #[serde(rename = "type")]
    pub type_: String,
}

// Define the `ErrorResponse` struct to represent the error response.
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}
