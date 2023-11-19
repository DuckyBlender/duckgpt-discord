use serenity::prelude::*;
use serenity::model::channel::Message;
use serenity::framework::standard::{CommandResult, macros::command};

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;
    Ok(())
}