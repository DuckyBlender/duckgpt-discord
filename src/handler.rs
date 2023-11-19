// handler.rs
use serenity::{prelude::*, async_trait};
use serenity::model::gateway::Ready;
use serenity::model::channel::Message;
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
        message::handle(&ctx, msg).await;
    }
}