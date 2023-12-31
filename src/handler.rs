use std::env;
use std::sync::Arc;

use serenity::http::{Http, Typing};
// handler.rs
use crate::constants::TESTER_ROLE_ID;
use crate::events::{message, ready};
use serenity::model::channel::{Message, MessageType};
use serenity::model::gateway::Ready;
use serenity::{async_trait, prelude::*};
use tracing::{info, warn};

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

        // Ignore system messages
        if msg.kind != MessageType::Regular {
            info!("Ignoring system message (type: {:?})", msg.kind);
            return;
        }

        // Ignore messages from users without the role
        let member = msg.member(&ctx).await.unwrap();
        if !member.roles.contains(&TESTER_ROLE_ID.into()) {
            warn!("User {} doesn't have the role", member.user.name); // this should never happen as the discord is setup so that only people with the role can send messages
            return;
        }

        // Typing indicator
        let http = Http::new(&env::var("DISCORD_TOKEN").expect("Token not set!"));
        let typing = Typing::start(Arc::new(http), msg.channel_id.into());
        message::handle(&ctx, msg).await;
        typing.stop();
    }
}
