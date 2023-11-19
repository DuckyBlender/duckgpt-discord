use std::env;
use std::sync::Arc;

use serenity::http::{Http, Typing};
// handler.rs
use serenity::{prelude::*, async_trait};
use serenity::model::gateway::Ready;
use serenity::model::channel::Message;
use tracing::info;
use crate::constants::TESTER_ROLE_ID;
use crate::events::{ready, message};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        ready::handle(&ctx, ready).await;
    }

    async fn message(&self, ctx: Context, msg: Message) {
        // Ignore bot messages
        if msg.author.bot {
            return;
        }
        // Ignore messages from users without the role
        let member = msg.member(&ctx).await.unwrap();
        if !member.roles.contains(&serenity::model::id::RoleId(TESTER_ROLE_ID)) {
            info!("User {} doesn't have the role", member.user.name); // this should never happen as the discord is setup so that only people with the role can send messages
            return;
        }
        // Typing indicator
        let http = Http::new(&env::var("DISCORD_TOKEN").expect("Token not set!"));
        let typing = Typing::start(Arc::new(http), msg.channel_id.into()).unwrap();
        message::handle(&ctx, msg).await;
        typing.stop().unwrap();
    }
}