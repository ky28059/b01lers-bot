use poise::CreateReply;
use serenity::all::{CreateEmbed, GetMessages, Mentionable};
use strum::IntoEnumIterator;

use crate::config::config;
use crate::points::get_point_cutoffs;
use crate::db::ChallengeType;

use super::{CmdContext, Error};

#[poise::command(slash_command, subcommands("solves", "leaderboard", "rank"))]
pub async fn stats(_ctx: CmdContext<'_>) -> Result<(), Error> {
    Ok(())
}

/// Gets statiscits about the challenges you have solved
#[poise::command(slash_command)]
pub async fn solves(ctx: CmdContext<'_>) -> Result<(), Error> {
    let user_id = ctx.author().id;
    let solves = ctx.data().db.get_solved_challenges_for_user(user_id).await?;

    let mut stats_embed = CreateEmbed::new()
        .title("CTF Solve Stats")
        .description("Number of challenges in each catagory you have solved")
        .color(0xc22026);

    for category in ChallengeType::iter() {
        let solve_count = solves.iter()
            .filter(|solve| solve.category == category)
            .count();

        stats_embed = stats_embed.field(category.to_string(), solve_count.to_string(), true);
    }

    let message = CreateReply::default()
        .embed(stats_embed);

    ctx.send(message).await?;

    Ok(())
}

/// Lists the top point leaders on the server
#[poise::command(slash_command)]
pub async fn leaderboard(ctx: CmdContext<'_>) -> Result<(), Error> {
    let mut embed = CreateEmbed::new()
        .title("b01lers Leaderboard")
        .description("Here is the current leaderboard on the server")
        .color(0xc22026)
        .thumbnail("https://pbs.twimg.com/profile_images/568451513295441921/9Hm60msK_400x400.png");

    let mut users = String::new();
    let mut points = String::new();

    for (i, user) in ctx.data().db.get_users_by_points(10).await?.iter().enumerate() {
        let position = match i {
            0 => "🥇".to_string(),
            1 => "🥈".to_string(),
            2 => "🥉".to_string(),
            _ => format!("{i}. "),
        };

        users.push_str(&format!("{position}{}\n", user.id.mention()));
        points.push_str(&format!("{}\n", user.points));
    }

    embed = embed.field("Users", users, true)
        .field("Points", points, true);

    let message = CreateReply::default()
        .embed(embed);

    ctx.send(message).await?;

    Ok(())
}

/// Lists your points and the point requirements of other ranks
#[poise::command(slash_command)]
pub async fn rank(ctx: CmdContext<'_>) -> Result<(), Error> {
    let user = ctx.data().db.get_user_by_id(ctx.author().id).await?;

    let mut embed = CreateEmbed::new()
        .title("Server Rank")
        .description("Points can be earned through participation in the server, like sending messages or solving CTF challenges.")
        .color(0xc22026)
        .field("Point Total", user.points.to_string(), true);

    let cutoffs = get_point_cutoffs(&ctx.data().db).await?;
    let rank_names = &config().ranks.rank_names;
    for (i, (rank, points)) in rank_names.iter().zip(cutoffs).enumerate() {
        embed = embed.field(rank, format!("Rank #{i} @ {points} points."), true);
    }

    let message = CreateReply::default()
        .embed(embed);

    ctx.send(message).await?;

    Ok(())
}

use serenity::futures::StreamExt;
use serenity::all::Message;

#[poise::command(slash_command)]
pub async fn save_solves_channel(ctx: CmdContext<'_>) -> Result<(), Error> {
    ctx.say("starting save").await?;
    //const SOLVE_APPROVALS_CHANNEL_ID: ChannelId = ChannelId::new(884853692367511623);
    /*let messages: Vec<Result<Message, serenity::Error>> = SOLVE_APPROVALS_CHANNEL_ID.messages_iter(ctx)
        .collect()
        .await;*/

    let mut messages = Vec::new();
    let mut last_id = None;

    loop {
        let filter = if let Some(last_id) = last_id {
            GetMessages::new().limit(50).before(last_id)
        } else {
            GetMessages::new().limit(50)
        };

        let mut new_messages = config().server.solve_approvals_channel_id
            .messages(ctx, filter).await?;

        if new_messages.len() == 0 {
            break;
        }
        ctx.say(format!("retrieved 50 message, last message: {:?}", new_messages.last().unwrap().timestamp)).await?;
        messages.append(&mut new_messages);

        last_id = Some(messages.last().unwrap().id);
    }

    ctx.say("collected messages").await?;

    /*let messages: Vec<Message> = messages
        .into_iter()
        .filter_map(|item| match item {
            Ok(message) => Some(message),
            Err(_) => None,
        })
        .collect();*/

    let data = format!("{messages:#?}");
    tokio::fs::write("solve_messages", data.as_bytes()).await?;

    ctx.say("saved messages").await?;

    Ok(())
}