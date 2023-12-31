mod commands;
mod constants;
mod events;
mod handler;
mod structs;
mod utils;

use handler::Handler;
use serenity::framework::standard::macros::group;
use serenity::framework::standard::Configuration;
use serenity::framework::StandardFramework;
use serenity::prelude::*;
use serenity::Client;
use std::env;
use tracing::error;

#[group]
// #[commands(ping)]
struct General;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    dotenv::dotenv().expect("Failed to load .env file");

    // Create the framework
    let framework = StandardFramework::new().group(&GENERAL_GROUP);
    framework.configure(Configuration::new().prefix("."));

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
