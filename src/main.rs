use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::Utc;
use log::{info, LevelFilter};
use ollama_rs::{
    generation::{completion::request::GenerationRequest, images::Image, parameters::KeepAlive},
    Ollama,
};
use poise::{
    serenity_prelude::{self as serenity, CreateAttachment, CreateEmbed, CreateEmbedFooter},
    CreateReply,
};

pub struct Data {} // User data, which is stored and accessible in all command invocations
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

mod models;
use crate::models::*;

mod utils;
use crate::utils::*;

/// Generates a text reponse using the specified model and prompt
#[poise::command(slash_command, prefix_command, user_cooldown = 10)]
async fn llm(
    ctx: Context<'_>,
    #[description = "Model"] model: LLMModels,
    #[description = "Prompt"] prompt: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    info!("Generating response for model `{model}` and prompt `{prompt}`...");
    let ollama = Ollama::default();
    let response = ollama
        .generate(GenerationRequest::new(model.to_string(), prompt.clone()))
        .await;

    if let Err(e) = response {
        handle_error(&ctx, e.into()).await?;
        return Ok(());
    }
    let response = response.unwrap();

    let footer = CreateEmbedFooter::new("Made by @DuckyBlender | Generated with Ollama");
    // token count / duration
    let response_speed = f32::from(response.final_data.clone().unwrap().eval_count)
        / (response.final_data.clone().unwrap().eval_duration as f32 / 1_000_000_000.0);
    let res = response.response.trim();

    let description = format!("**Prompt:**\n{}\n\n**Response:**\n{}", prompt, res);

    let embed = CreateEmbed::default()
        .title(format!("Generated by `{model}`"))
        .color(0x00ff00)
        .description(description)
        .field(
            "Duration",
            format!(
                "`{:.2}s`",
                // total_durations is in nanoseconds
                response.final_data.clone().unwrap().total_duration as f32 / 1_000_000_000.0
            ),
            true,
        )
        .field("Speed", format!("`{response_speed:.2} tokens/s`"), true)
        .footer(footer)
        .timestamp(Utc::now());
    let message = CreateReply::default().embed(embed);
    ctx.send(message).await?;
    info!("Response sent successfully");

    Ok(())
}

#[poise::command(slash_command, prefix_command, user_cooldown = 10)]
async fn clone_image(
    ctx: Context<'_>,
    #[description = "Image to clone"] image_url: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    // Get the image and save it to a buffer
    let image = reqwest::get(&image_url).await?.bytes().await?;
    // Convert image to base64
    let image = STANDARD.encode(image);

    info!("Getting prompt from image...");
    let start_time = std::time::Instant::now();
    let ollama = Ollama::default();
    let res = ollama
        .generate(
            GenerationRequest::new(
                "knoopx/llava-phi-2:3b-q8_0".to_string(),
                "Describe this image in one sentence.".to_string(),
            )
            .images(vec![Image::from_base64(&image)])
            .keep_alive(KeepAlive::UnloadOnCompletion), // unload after completion to give more resources to the image generation
        )
        .await;
    let text_elapsed = start_time.elapsed();

    // Response
    if let Err(e) = res {
        handle_error(&ctx, e.into()).await?;
        return Ok(());
    }
    let res = res.unwrap();
    let res = res.response.trim();
    // Generate the image using the llava response
    let images = process_image_generation(&res, &ImageModels::SDXLTurbo) // quality doesn't matter as much here
        .await;
    if let Err(e) = images {
        handle_error(&ctx, e.into()).await?;
        return Ok(());
    }
    let images = images.unwrap();
    let img_elapsed = text_elapsed + start_time.elapsed();

    // Send this as an attachment
    // let attachment = CreateAttachment::bytes(image, "crong.png");
    let attachments = images
        .iter()
        .map(|(filename, bytes)| CreateAttachment::bytes(bytes.clone(), filename))
        .collect::<Vec<_>>();

    // For now just send the first image (because we're generating one image)
    // I'm not sure if it's even possible to send multiple images in a single message
    let footer = CreateEmbedFooter::new(format!(
        "Made by @DuckyBlender | Generated with LLaVA-Phi -> SDXL-Turbo"
    ));
    let message = CreateReply::default()
        .attachment(attachments[0].clone())
        .embed(
            CreateEmbed::default()
                .title("SDXL-Turbo")
                .fields(vec![
                    ("Prompt", format!("`{res}`"), true),
                    (
                        "Duration",
                        format!(
                            "`{:.2}s + {:.2}s = {:.2}s`",
                            text_elapsed.as_secs_f32(),
                            img_elapsed.as_secs_f32(),
                            img_elapsed.as_secs_f32() + text_elapsed.as_secs_f32()
                        ),
                        true,
                    ),
                    // ("Steps", format!("`{steps}`"), true),
                ])
                .color(0x00ff00)
                .footer(footer)
                .timestamp(Utc::now()),
        );
    ctx.send(message).await?;
    info!("Image sent successfully");

    Ok(())
}

/// Generates an image using the specified model and prompt
#[poise::command(slash_command, prefix_command, user_cooldown = 10)]
async fn img(
    ctx: Context<'_>,
    #[description = "Model"] model: ImageModels,
    #[description = "Prompt"] prompt: String,
) -> Result<(), Error> {
    info!("Generating image for prompt `{prompt}`...");
    ctx.defer().await?;
    let before = std::time::Instant::now();
    let imgs = process_image_generation(&prompt, &model).await;
    if let Err(e) = imgs {
        handle_error(&ctx, e.into()).await?;
        return Ok(());
    }
    let imgs = imgs.unwrap();
    let after = std::time::Instant::now();
    info!(
        "Image generated successfully in {elapsed}ms",
        elapsed = (after - before).as_millis()
    );
    let attachments = imgs
        .iter()
        .map(|(filename, bytes)| CreateAttachment::bytes(bytes.clone(), filename))
        .collect::<Vec<_>>();
    let footer = CreateEmbedFooter::new(format!("Generated by @DuckyBlender | Model: {model}"));
    let message = CreateReply::default()
        .attachment(attachments[0].clone())
        .embed(
            CreateEmbed::default()
                .title(format!("Generated by `{model}`"))
                .color(0x00ff00)
                .description(format!("**Prompt:**\n{}", prompt))
                .field(
                    "Duration",
                    format!("`{:.2}s`", (after - before).as_secs_f32()),
                    true,
                )
                .footer(footer)
                .timestamp(Utc::now()),
        );
    ctx.send(message).await?;
    info!("Image sent successfully");
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    // Initialize env_logger with a custom configuration
    let mut builder = env_logger::Builder::new();
    builder.filter(None, LevelFilter::Warn); // Set the default level to Warn for all modules
    builder.filter(Some("duckgpt"), LevelFilter::Info);
    builder.filter(Some("comfyui_rs"), LevelFilter::Info);
    builder.init();

    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![llm(), img(), clone_image()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                let guild_id = serenity::GuildId::new(1175184892225671258);
                poise::builtins::register_in_guild(ctx, &framework.options().commands, guild_id)
                    .await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
