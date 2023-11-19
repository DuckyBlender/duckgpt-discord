use serenity::prelude::*;
use serenity::model::gateway::Ready;
use tracing::info;

pub async fn handle(_ctx: &Context, ready: Ready) {
    info!("{} is connected!", ready.user.name);
}