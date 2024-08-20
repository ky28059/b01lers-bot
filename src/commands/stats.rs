use anyhow::anyhow;
use poise::CreateReply;
use serenity::all::CreateEmbed;
use strum::IntoEnumIterator;

use crate::db::ChallengeType;

use super::{CmdContext, Error};

#[poise::command(slash_command, subcommands("solves"))]
pub async fn stats(_ctx: CmdContext<'_>) -> Result<(), Error> {
    Ok(())
}

/// Gets statiscits about the challenges you have solved
#[poise::command(slash_command)]
pub async fn solves(ctx: CmdContext<'_>) -> Result<(), Error> {
    let user_id = ctx.author().id;
    let solves = ctx.data().db.get_solves_for_user(user_id).await?;

    let mut stats_embed = CreateEmbed::new()
        .title("CTF Solve Stats")
        .description("Number of challenges in each catagory you have solved")
        .color(0xc22026);

    for category in ChallengeType::iter() {
        let solve_count = solves.iter()
            .filter(|solve| solve.challenge_type == category)
            .count();

        stats_embed = stats_embed.field(category.to_string(), solve_count.to_string(), true);
    }

    let message = CreateReply::default()
        .embed(stats_embed);

    ctx.send(message).await?;

    Ok(())
}

#[poise::command(slash_command)]
pub async fn error(ctx: CmdContext<'_>) -> Result<(), Error> {
    Err(anyhow!("A catastrophic error has occured"))
}