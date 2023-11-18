use std::env;
use std::sync::Arc;

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serenity::framework::StandardFramework;
use serenity::http::{Http, Typing};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::{async_trait, framework::standard::CommandResult};
use std::time::Instant;
use tracing::{debug, error, info};

use serenity::framework::standard::macros::{command, group};

mod structs;
use structs::*;

mod functions;
use functions::*;

const MAX_TOKENS: u32 = 600;

const ALLOWED_EXTENSIONS: [&str; 5] = [".png", ".jpg", ".jpeg", ".gif", ".webp"];
const LOW_QUALITY_ID: u64 = 1175189750311817249;
const HIGH_QUALITY_ID: u64 = 1175210913972883576;

#[group]
// #[commands(ping)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }

    async fn message(&self, ctx: Context, new_message: Message) {
        // Ignore messages in other channels
        if new_message.channel_id != LOW_QUALITY_ID && new_message.channel_id != HIGH_QUALITY_ID {
            debug!("Ignoring message in other channel");
            return;
        }
        let quality = match new_message.channel_id {
            serenity::model::id::ChannelId(LOW_QUALITY_ID) => "low",
            serenity::model::id::ChannelId(HIGH_QUALITY_ID) => "high",
            _ => unreachable!(),
        };

        // Ignore messages from bots
        if new_message.author.bot {
            debug!("Ignoring message from bot");
            return;
        }
        // Ignore messages from users without the role
        let member = new_message.member(&ctx).await.unwrap();
        let role_id = serenity::model::id::RoleId(1175203159195533382);
        if !member.roles.contains(&role_id) {
            info!("User {} doesn't have the role", member.user.name); // this should never happen as the discord is setup so that only people with the role can send messages
            return;
        }
        // Ignore messages that don't contain an attachment
        let attachment_count = new_message.attachments.len();
        // TODO: Also check for embeds
        debug!("Attachment count: {}", attachment_count);
        if attachment_count == 0 {
            debug!("Ignoring message without attachment");
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

        // typing indicator
        let http = Http::new(&env::var("DISCORD_TOKEN").expect("Token not set!"));
        let typing = Typing::start(Arc::new(http), new_message.channel_id.into()).unwrap();

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
                let reply = v["choices"][0]["message"]["content"].as_str().expect(
                    format!("content should be a string\nfull response: \n\n{:?}", v).as_str(),
                );
                let (height, width) = (file.height.unwrap(), file.width.unwrap());

                let total_cost = convert_tokens_to_cost(
                    input_tokens as u32,
                    output_tokens as u32,
                    width as u32,
                    height as u32,
                    quality,
                );

                // Form the embed reply
                let embed_result = new_message
                    .channel_id
                    .send_message(&ctx.http, |m| {
                        m.embed(|e| {
                            e.title("Image Analysis Result")
                                .description(format!(
                                    "Analysis for the submitted image in {} quality.",
                                    quality
                                ))
                                .image(file.url.as_str()) // Use the URL of the submitted image
                                .fields(vec![
                                    ("Prompt", text, false), // Display the prompt used for the analysis
                                    ("Response", format!("```\n{}\n```", reply), false), // Display the OpenAI API response
                                ])
                                .field(
                                    "Analysis Time",
                                    format!("{:.2} seconds", elapsed as f64 / 1000.0),
                                    true,
                                ) // Display the time taken for analysis
                                .field("Estimated Cost", format!("${:.4}", total_cost), true) // Display the estimated cost
                                .footer(|f| f.text("Powered by OpenAI | Created by @DuckyBlender"))
                            // Add a footer
                            // .timestamp(Timestamp::now()) // Add a timestamp for the current time
                        })
                    })
                    .await;

                // Check if the message was sent successfully and handle any errors
                if let Err(why) = embed_result {
                    error!("Error sending message: {:?}", why);
                    // send a reply to the user
                    new_message
                        .reply(
                            ctx,
                            format!(
                                "Error sending message: {:?}\n\n`Cost: ${:.2}`",
                                why, total_cost
                            ),
                        )
                        .await
                        .unwrap();
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

                // Reply to the user with the error message
                new_message
                    .reply(ctx, format!("Error from OpenAI API: {}", error_message))
                    .await
                    .unwrap();
            }
            // REQUEST ERROR
            Err(error) => {
                error!("Error sending request to OpenAI API: {:?}", error);
                // Reply to the user with a generic error message
                new_message
                    .reply(
                        ctx,
                        "Error communicating with OpenAI API. Please try again later.",
                    )
                    .await
                    .unwrap();
            }
        }
        typing.stop();
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

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
        error!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}
