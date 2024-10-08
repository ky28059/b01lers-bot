mod commands;
mod db;
mod logging;
mod email;
mod points;

use std::env;
use dotenvy::dotenv;
use email::EmailClient;
use logging::init_logging;
use poise::{BoxFuture, FrameworkContext, FrameworkError};
use serenity::all::{ChannelId, ClientBuilder, Context, FullEvent, GatewayIntents, GuildId, Interaction};
use tracing::{error, info};

use commands::CommandContext;
use db::DbContext;

const B01LERS_GUILD_ID: GuildId = GuildId::new(511675552386777099);
const CTF_CATEGORY_ID: ChannelId = ChannelId::new(534524532799569950);
const ARCHIVED_CTF_CATEGORY_ID: ChannelId = ChannelId::new(877584240965984256);
const SOLVE_APPROVALS_CHANNEL_ID: ChannelId = ChannelId::new(757358907034435686);
const BOT_LOG_CHANNEL: ChannelId = ChannelId::new(743238600329658459);
const OFFICER_ROLE: &str = "officer";
const MEMBER_ROLE: &str = "members";
const POINTS_PER_SOLVE: i64 = 100;
const POINTS_PER_MESSAGE: i64 = 1;
const RANK_COUNT: usize = 20;
const RANKS_NAMES: [&str; RANK_COUNT] = [
    "🐟 Phish Food",
    "📜 Script Kiddie",
    "⌨️ /r/masterhacker",
    "</> Inspector of Elements",
    "❌ Cross-Site Scripter",
    "💫 Path Explorer",
    "🎩 White Hat",
    "🛠️ Pwn Tool",
    "🤫 Bad Actor",
    "🐣 Fuzzer Duckling",
    "🦦 ShellMammal",
    "👾 Anti-Anti Virus",
    "💻 Not an Enigma",
    "🐚 Shell Popper",
    "💸 Bounty Hunter",
    "💼 Edge Case",
    "🖥️ Gibson Crasher",
    "🔥 Intrusion Creation System",
    "🍦 Zero Cool",
    "🌈 1337",
];

/// Runs for every serenity event
///
/// Currently needed for solve approve / reject buttons to work
fn event_handler<'a>(
    context: &'a Context,
    event: &'a FullEvent,
    framework_context: FrameworkContext<'a, CommandContext, anyhow::Error>,
    user_data: &'a CommandContext,
) -> BoxFuture<'a, Result<(), anyhow::Error>> {
    Box::pin(async move {
        if let FullEvent::InteractionCreate {
            interaction: Interaction::Component(component_interaction)
        } = event {
            commands::solve::handle_approval_button(context, user_data, component_interaction).await?;
        }

        if let FullEvent::Message { new_message } = event {
            framework_context.user_data().await.db.give_user_points(new_message.author.id, POINTS_PER_MESSAGE).await?;
        }

        Ok(())
    })
}

fn pre_command_handler<'a>(context: poise::Context<'a, CommandContext, anyhow::Error>) -> BoxFuture<'a, ()> {
    Box::pin(async move {
        info!("Running command `{}`", context.invocation_string());
    })
}

/// Runs on every error, logs the error in the error channel
fn error_handler<'a>(
    error: FrameworkError<'a, CommandContext, anyhow::Error>,
) -> BoxFuture<'a, ()> {
    Box::pin(async move {
        // first report error in discord channel
        match &error {
            FrameworkError::Command { error, ctx, .. } => {
                error!("error in `{}` command: {}", ctx.invoked_command_name(), error);
            },
            FrameworkError::CommandPanic { payload: Some(payload), ctx, .. } => {
                error!("command `{}` has paniced: {}", ctx.invoked_command_name(), payload);
            },
            FrameworkError::CommandPanic { payload: None, ctx, .. } => {
                error!("command `{}` has paniced", ctx.invoked_command_name());
            },
            // TODO: handle other type of errors
            _ => (),
        }

        // then report error to user
        if let Err(e) = poise::builtins::on_error(error).await {
            error!("Error while handling error: {}", e);
        }
    })
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url =
        env::var("DATABASE_URL").expect("No `DATABASE_URL` environment variable specified");

    let discord_token =
        env::var("DISCORD_TOKEN").expect("No `DISCORD_TOKEN` environment variable specified");
    
    let mailgun_token =
        env::var("MAILGUN_TOKEN").expect("No `MAILGUN_TOKEN` environment variable specified");

    let db = DbContext::connect(&database_url)
        .await
        .expect("failed to connect to database");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::competition::competition(),
                commands::bingo::bingo(),
                commands::archive::archive(),
                commands::challenge::challenge(),
                commands::solve::solve(),
                commands::verify::verify(),
                commands::stats::stats(),
                commands::stats::save_solves_channel(),
            ],
            event_handler,
            pre_command: pre_command_handler,
            on_error: error_handler,
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                // set up logging to bot log channel
                init_logging(ctx.clone());

                // register the bots commands with discord api on startup
                // poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                poise::builtins::register_in_guild(
                    ctx,
                    &framework.options().commands,
                    B01LERS_GUILD_ID,
                )
                .await?;

                info!("the bot has logged on");

                let email_client = EmailClient::new(mailgun_token);
                Ok(CommandContext::new(db, email_client))
            })
        })
        .build();

    let mut client = ClientBuilder::new(discord_token, GatewayIntents::non_privileged())
        .framework(framework)
        .await
        .expect("Failed to create b01lers bot client");

    client.start().await.expect("Failed to run b01lers bot");
}
