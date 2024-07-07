mod commands;
mod db;

use std::env;
use dotenvy::dotenv;
use serenity::all::{ChannelId, ClientBuilder, GatewayIntents, GuildId};

use commands::CommandContext;
use db::DbContext;

const B01LERS_GUILD_ID: GuildId = GuildId::new(511675552386777099);
const CTF_CATEGORY_ID: ChannelId = ChannelId::new(534524532799569950);
const ARCHIVED_CTF_CATEGORY_ID: ChannelId = ChannelId::new(877584240965984256);
const OFFICER_ROLE: &str = "officer";

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url =
        env::var("DATABASE_URL").expect("No `DATABASE_URL` environment variable specified");

    let discord_token =
        env::var("DISCORD_TOKEN").expect("No `DISCORD_TOKEN` environment variable specified");

    let db = DbContext::connect(&database_url)
        .await
        .expect("failed to connect to database");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::competition::competition(),
                commands::bingo::bingo(),
                commands::archive::archive()
            ],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                // register the bots commands with discord api on startup
                // poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                poise::builtins::register_in_guild(
                    ctx,
                    &framework.options().commands,
                    B01LERS_GUILD_ID,
                )
                .await?;
                Ok(CommandContext::new(db))
            })
        })
        .build();

    let mut client = ClientBuilder::new(discord_token, GatewayIntents::non_privileged())
        .framework(framework)
        .await
        .expect("Failed to create b01lers bot client");

    client.start().await.expect("Failed to run b01lers bot");
}
