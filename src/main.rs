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

const LOW_QUALITY_CHANNEL_ID: u64 = 1175189750311817249;
const HIGH_QUALITY_CHANNEL_ID: u64 = 1175210913972883576;

const ALLOY_CHANNEL_ID: u64 = 1175600783979466843;
const ECHO_CHANNEL_ID: u64 = 1175601080093130875;
const FABLE_CHANNEL_ID: u64 = 1175601105070194708;
const ONYX_CHANNEL_ID: u64 = 1175601123273474148;
const NOVA_CHANNEL_ID: u64 = 1175601138683359272;
const SHIMMER_CHANNEL_ID: u64 = 1175601147155849236;

#[group]
// #[commands(ping)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }

    async fn message(&self, ctx: Context, msg: Message) {
        // Ignore messages in other channels
        match msg.channel_id {
            serenity::model::id::ChannelId(ALLOY_CHANNEL_ID)
            | serenity::model::id::ChannelId(ECHO_CHANNEL_ID)
            | serenity::model::id::ChannelId(FABLE_CHANNEL_ID)
            | serenity::model::id::ChannelId(ONYX_CHANNEL_ID)
            | serenity::model::id::ChannelId(NOVA_CHANNEL_ID)
            | serenity::model::id::ChannelId(SHIMMER_CHANNEL_ID) => {
                let voice = match msg.channel_id {
                    serenity::model::id::ChannelId(ALLOY_CHANNEL_ID) => "alloy",
                    serenity::model::id::ChannelId(ECHO_CHANNEL_ID) => "echo",
                    serenity::model::id::ChannelId(FABLE_CHANNEL_ID) => "fable",
                    serenity::model::id::ChannelId(ONYX_CHANNEL_ID) => "onyx",
                    serenity::model::id::ChannelId(NOVA_CHANNEL_ID) => "nova",
                    serenity::model::id::ChannelId(SHIMMER_CHANNEL_ID) => "shimmer",
                    _ => unreachable!(),
                };

                info!("TTS channel message");
                let openai_token = std::env::var("OPENAI_TOKEN").expect("OPENAI_TOKEN not set");

                // Get the message
                let message = msg.content.clone();
                // Send the request to the TTS API
                let client = reqwest::Client::new();
                let speech_request = SpeechRequest {
                    model: "tts-1".to_string(),
                    input: message,
                    voice: voice.to_string(),
                };

                let response = client
                    .post("https://api.openai.com/v1/audio/speech")
                    .header("Authorization", format!("Bearer {}", openai_token))
                    .header("Content-Type", "application/json")
                    .json(&speech_request)
                    .send()
                    .await
                    .unwrap();

                // The response is a file, so we need to get the bytes
                let bytes = response.bytes().await.unwrap();

                // Send the bytes to the channel
                let _ = msg
                    .channel_id
                    .send_message(&ctx.http, |m| {
                        m.content("Here is your TTS message!")
                            .add_file((bytes.as_ref(), "tts.mp3"))
                    })
                    .await;

                return ;
                
            }
            serenity::model::id::ChannelId(LOW_QUALITY_CHANNEL_ID) => {
                info!("Low quality channel message");
            }
            serenity::model::id::ChannelId(HIGH_QUALITY_CHANNEL_ID) => {
                info!("High quality channel message");
            }
            _ => {
                info!("Ignoring message in other channel");
                return;
            }
        }

        let quality = match msg.channel_id {
            serenity::model::id::ChannelId(LOW_QUALITY_CHANNEL_ID) => "low",
            serenity::model::id::ChannelId(HIGH_QUALITY_CHANNEL_ID) => "high",
            _ => unreachable!(),
        };

        // Ignore messages from bots
        if msg.author.bot {
            debug!("Ignoring message from bot");
            return;
        }
        // Ignore messages from users without the role
        let member = msg.member(&ctx).await.unwrap();
        if !member.roles.contains(&1175203159195533382.into()) {
            info!("User {} doesn't have the role", member.user.name); // this should never happen as the discord is setup so that only people with the role can send messages
            return;
        }
        // Ignore messages that don't contain an attachment or URL
        let attachment_count = msg.attachments.len();
        // let has_url = is_image_url(&msg.content);
        debug!("Attachment count: {}", attachment_count);
        if attachment_count == 0 {
            debug!("Ignoring message without attachment");
            // msg.reply(ctx, "Please attach an image or provide a URL!").await.unwrap();
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
                msg.reply(
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

        // typing indicator
        let http = Http::new(&env::var("DISCORD_TOKEN").expect("Token not set!"));
        let typing = Typing::start(Arc::new(http), msg.channel_id.into()).unwrap();

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
                let reply = v["choices"][0]["message"]["content"].as_str().expect(
                    format!("content should be a string\nfull response: \n\n{:?}", v).as_str(),
                );
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
                                    .footer(|f| {
                                        f.text("Powered by OpenAI | Created by @DuckyBlender")
                                    })
                            })
                        })
                        .await;

                    // Check if the message was sent successfully and handle any errors
                    if let Err(why) = embed_result {
                        error!("Error sending message: {:?}", why);
                        // send a reply to the user
                        msg.reply(
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
        typing.stop();
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    dotenv::dotenv().expect("Failed to load .env file");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix(".")) // unused prefix
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
