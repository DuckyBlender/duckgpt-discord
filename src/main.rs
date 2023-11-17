use std::env;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serenity::framework::StandardFramework;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::{async_trait, framework::standard::CommandResult};

use serenity::framework::standard::macros::{command, group};

mod structs;
use structs::*;

const MAX_TOKENS: u32 = 600;

const ALLOWED_EXTENSIONS: [&str; 5] = [".png", ".jpg", ".jpeg", ".gif", ".webp"];
const LOW_QUALITY_ID: u64 = 1175189750311817249;
const HIGH_QUALITY_ID: u64 = 1175210913972883576;

fn calculate_image_token_cost(width: u32, height: u32, detail: &str) -> u32 {
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

#[derive(Serialize, Deserialize, Debug)]
struct UsageStats {
    uses: u32,
    cost: u32,
}

fn convert_tokens_to_cost(
    input_tokens: u32,
    output_tokens: u32,
    width: u32,
    height: u32,
    detail_level: &str,
) -> f32 {
    const COST_PER_INPUT_TOKEN: f32 = 0.01 / 1000.0;
    const COST_PER_OUTPUT_TOKEN: f32 = 0.03 / 1000.0;
    let input_cost = input_tokens as f32 * COST_PER_INPUT_TOKEN;
    let output_cost = output_tokens as f32 * COST_PER_OUTPUT_TOKEN;
    let image_cost =
        calculate_image_token_cost(width, height, detail_level) as f32 * COST_PER_OUTPUT_TOKEN;

    // Calculate the total cost
    let total_cost = input_cost + output_cost + image_cost;

    // Update and save the usage stats
    update_usage_stats(input_tokens + output_tokens);

    total_cost
}

fn update_usage_stats(tokens_used: u32) {
    let mut stats = read_usage_stats().unwrap_or(UsageStats { uses: 0, cost: 0 });

    stats.uses += 1;
    stats.cost += tokens_used;

    let stats_json = serde_json::to_string(&stats).expect("Error converting stats to JSON");
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open("usage_stats.json")
        .expect("Error opening usage_stats.json for writing");
    file.write_all(stats_json.as_bytes())
        .expect("Error writing to usage_stats.json");
}

fn read_usage_stats() -> Option<UsageStats> {
    let mut file = match File::open("usage_stats.json") {
        Ok(file) => file,
        Err(_) => return None,
    };

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Error reading usage_stats.json");

    serde_json::from_str(&contents).ok()
}

#[group]
// #[commands(ping)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    async fn message(&self, ctx: Context, new_message: Message) {
        // Ignore messages in other channels
        if new_message.channel_id != LOW_QUALITY_ID && new_message.channel_id != HIGH_QUALITY_ID {
            // println!("Ignoring message in other channel");
            return;
        }
        let quality = match new_message.channel_id {
            serenity::model::id::ChannelId(LOW_QUALITY_ID) => "low",
            serenity::model::id::ChannelId(HIGH_QUALITY_ID) => "high",
            _ => unreachable!(),
        };

        // Ignore messages from bots
        if new_message.author.bot {
            // println!("Ignoring message from bot");
            return;
        }
        // Ignore messages from users without the role
        let member = new_message.member(&ctx).await.unwrap();
        // pub struct RoleId(#[serde(with = "snowflake")] pub u64);
        let role_id = serenity::model::id::RoleId(1175203159195533382);
        if !member.roles.contains(&role_id) {
            println!("User {} doesn't have the role", member.user.name);
            return;
        }
        // Ignore messages that don't contain an attachment
        let attachment_count = new_message.attachments.len();
        // TODO: Also check for embeds
        println!("Attachment count: {}", attachment_count);
        if attachment_count == 0 {
            println!("Ignoring message without attachment");
            // new_message.reply(ctx, "Please attach an image!").await.unwrap();
            return;
        }
        // Check if the attachment is an image
        let file = new_message.attachments.first().unwrap(); // safe to unwrap
        if !ALLOWED_EXTENSIONS
            .iter()
            .any(|&x| file.filename.ends_with(x))
        {
            // reply with an error message
            new_message
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
        // great, now we have an image
        // now get the text of the message
        let message_text = new_message.content.clone(); // this is without the attachment

        // TODO: Implement typing indicator

        let openai_token = std::env::var("OPENAI_TOKEN").expect("OPENAI_TOKEN not set");

        let text = if message_text.is_empty() {
            println!("Message text is empty, using default");
            "What is in this image?".to_string()
        } else {
            println!("Prompt: {}", message_text);
            message_text
        };

        let chat_completion_request = ChatCompletionRequest {
            model: "gpt-4-vision-preview".to_string(),
            messages: vec![UserMessage {
                role: "user".to_string(),
                content: vec![
                    Content {
                        content_type: "text".to_string(),
                        text: text.into(),
                        image_url: None,
                    },
                    // TODO: Add support for multiple images
                    Content {
                        content_type: "image_url".to_string(),
                        text: None,
                        image_url: Some(ImageUrl {
                            url: file.url.clone(),
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

        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .headers(headers)
            .json(&chat_completion_request)
            .send()
            .await
            .unwrap();

        // Prase the string of data into serde_json::Value.
        let v: serde_json::Value = response.json().await.unwrap();

        // Check if response is valid

        // Access the nested total_tokens value
        let input_tokens = v["usage"]["prompt_tokens"]
            .as_u64()
            .expect(format!("prompt_tokens should be a u64\nfull response: \n\n{:?}", v).as_str());
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
        let (height, width) = (file.height.unwrap(), file.width.unwrap());

        let total_cost = convert_tokens_to_cost(
            input_tokens as u32,
            output_tokens as u32,
            width as u32,
            height as u32,
            quality,
        );

        // TODO: Create an embed

        // Send the reply
        new_message
            .reply(ctx, format!("{}\n\n`Cost: ${:.2}`", reply, total_cost))
            .await
            .unwrap(); // todo: add error handling
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to load .env file");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix(".")) // set the bot's prefix to "."
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("Token not set!");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}
