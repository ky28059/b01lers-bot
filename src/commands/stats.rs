use poise::CreateReply;
use serde::{Deserialize, Serialize};
use serenity::all::{CreateEmbed, GetMessages, Mentionable};
use strum::IntoEnumIterator;

use crate::config::config;
use crate::points::{get_point_cutoffs, points_to_string};
use crate::db::ChallengeType;

use super::{role_id_for_role_name, CmdContext, Error};

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
            _ => format!("{}. ", i + 1),
        };

        users.push_str(&format!("{position}{}\n", user.id.mention()));
        points.push_str(&format!("{}\n", points_to_string(user.points)));
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
        .field("Point Total", points_to_string(user.points), true);

    let cutoffs = get_point_cutoffs(&ctx.data().db).await?;
    let rank_names = &config().ranks.rank_names;
    for (i, (rank, points)) in rank_names.iter().zip(cutoffs).enumerate() {
        embed = embed.field(rank, format!("Rank #{i} @ {} points.", points_to_string(points)), true);
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

    let data = ron::to_string(&messages)?;
    //let data = format!("{messages:#?}");
    tokio::fs::write("solve_messages.ron", data.as_bytes()).await?;

    ctx.say("saved messages").await?;

    Ok(())
}

use serenity::all::{ChannelId, UserId};
const BOT_ID: UserId = UserId::new(690299165162471622);

#[derive(Debug, Clone, Serialize)]
struct Solve {
    name: String,
    flag: String,
    category: Category,
    ctf: ChannelId,
    ctf_name: String,
    solvers: Vec<UserId>,
}


#[derive(Debug, Clone, Copy, Serialize)]
enum Category {
    Rev,
    Pwn,
    Web,
    Crypto,
    Forensics,
    Misc,
}

impl Category {
    fn from_name(name: &str) -> Self {
        match name {
            "re" => Self::Rev,
            "pwn" => Self::Pwn,
            "web" => Self::Web,
            "crypto" => Self::Crypto,
            "forensics" => Self::Forensics,
            "other" => Self::Misc,
            _ => panic!("unknown category"),
        }
    }
}

fn update_time(message: &Message) -> i64 {
    message.edited_timestamp.unwrap().unix_timestamp()
}

#[poise::command(slash_command)]
pub async fn save_solves_channel_processed(ctx: CmdContext<'_>) -> Result<(), Error> {
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

    let mut solves = Vec::new();

    let messages = messages.into_iter()
        // posted by b01lers bot
        .filter(|message| message.author.id == BOT_ID)
        .filter(|message| message.content.starts_with("This request is approved by"));

    let mut n = 0;
    for message in messages {
        if message.embeds.len() != 1 {
            continue;
        }

        let embed = &message.embeds[0];
        let mut challenge = None;
        let mut category = None;
        let mut ctf = None;
        let mut flag = None;
        let mut participants = Vec::new();

        /*let desc = &embed.description.clone().unwrap();
        // look for thing before mention
        let start_index = desc.find('@').unwrap() + 1;
        let end_index = desc.find('>').unwrap();
        participants.push(desc[start_index..end_index].parse::<u64>().unwrap());*/

        for field in embed.fields.iter() {
            match field.name.as_str() {
                "Challenge" => challenge = Some(field.value.clone()),
                "Category" => category = Some(Category::from_name(&field.value)),
                "CTF" => ctf = Some(ChannelId::new(field.value[2..field.value.len() - 1].parse::<u64>().unwrap())),
                // remove backticks from code block
                "Flag" => flag = Some(field.value[3..field.value.len() - 3].to_string()),
                "Participants" => {
                    for participant in field.value.split(", ") {
                        participants.push(UserId::new(participant[2..participant.len() - 1].parse::<u64>().unwrap()));
                    }
                },
                _ => (),
            }
        }

        let (Some(chall), Some(category), Some(ctf), Some(flag)) = (challenge, category, ctf, flag) else {
            println!("invalid embed");
            continue;
        };

        let ctf_name = ctf.to_channel(ctx).await.unwrap().guild().unwrap().name;

        /*println!("challenge: {chall:?}");
        println!("category: {category:?}");
        println!("ctf: {ctf:?}");
        println!("flag: {flag:?}");
        println!("participants: {participants:?}");*/

        solves.push(Solve {
            name: chall,
            flag,
            category,
            ctf,
            ctf_name,
            solvers: participants,
        })
    }

    ctx.say("extracted solve data").await?;

    /*let messages: Vec<Message> = messages
        .into_iter()
        .filter_map(|item| match item {
            Ok(message) => Some(message),
            Err(_) => None,
        })
        .collect();*/

    let data = ron::to_string(&solves)?;
    //let data = format!("{messages:#?}");
    tokio::fs::write("solve_messages.ron", data.as_bytes()).await?;

    ctx.say("saved messages").await?;

    Ok(())
}

use serenity::all::Member;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct UserRank {
    user_id: UserId,
    rank: Option<usize>,
}

#[poise::command(slash_command)]
pub async fn save_user_ranks(ctx: CmdContext<'_>) -> Result<(), Error> {
    let members = config().server.guild_id.members_iter(ctx)
        .collect::<Vec<serenity::Result<Member>>>()
        .await
        .into_iter()
        .map(|member| member.unwrap());

    ctx.say("collected members").await?;

    let mut role_map = HashMap::new();

    for (i, role) in config().ranks.rank_names.iter().enumerate() {
        let role_id = role_id_for_role_name(ctx.serenity_context(), role).await?.unwrap();
        role_map.insert(role_id, i);
    }

    ctx.say("got role type ids").await?;

    let mut user_ranks = Vec::new();

    for member in members {
        let mut role = None;
        for role_id in member.roles.iter() {
            if let Some(new_role) = role_map.get(&role_id) {
                if let Some(old_role) = role {
                    if *new_role > old_role {
                        role = Some(*new_role);
                    }
                } else {
                    role = Some(*new_role);
                }
            }
        }

        user_ranks.push(UserRank {
            user_id: member.user.id,
            rank: role,
        });
    }

    ctx.say("got everyones rank").await?;

    let data = ron::to_string(&user_ranks)?;
    //let data = format!("{messages:#?}");
    tokio::fs::write("ranks.ron", data.as_bytes()).await?;

    Ok(())
}

#[poise::command(slash_command)]
pub async fn save_channels(ctx: CmdContext<'_>) -> Result<(), Error> {
    let channels: HashMap<ChannelId, String> = config().server.guild_id.channels(ctx).await?
        .into_iter()
        .map(|(id, channel)| (id, channel.name))
        .collect();

    let data = ron::to_string(&channels)?;
    tokio::fs::write("channels.ron", data.as_bytes()).await?;

    Ok(())
}