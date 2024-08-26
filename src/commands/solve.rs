use serenity::all::{ButtonStyle, ComponentInteraction, ComponentInteractionDataKind, Context, CreateButton, CreateEmbed, CreateMessage, EditMessage, EditThread, Mentionable, UserId};

use crate::config::config;
use crate::db::{ApprovalStatus, Solve};
use crate::points::check_rank_up;

use super::{CmdContext, CommandContext, Error, competition::{get_competition_from_ctx, get_challenge_from_ctx}};

#[poise::command(slash_command)]
pub async fn solve(
    ctx: CmdContext<'_>,
    #[description = "Flag of the challenge that was solved"] flag: String,
    #[description = "First Teammate"] teammate1: Option<UserId>,
    #[description = "Second Teammate"] teammate2: Option<UserId>,
    #[description = "Third Teammate"] teammate3: Option<UserId>,
    #[description = "Fourth Teammate"] teammate4: Option<UserId>,
    #[description = "Fifth Teammate"] teammate5: Option<UserId>,
    #[description = "Sixth Teammate"] teammate6: Option<UserId>,
    // TODO: figure out how to specify teammates
) -> Result<(), Error> {
    let competition = get_competition_from_ctx(&ctx).await?;
    let challenge = get_challenge_from_ctx(&ctx).await?;

    let mut challenge_channel = ctx.guild_channel().await
        .ok_or_else(|| anyhow::anyhow!("You are not inside a challenge channel."))?;

    let competition_forum_channel = challenge_channel.parent_id
        .ok_or_else(|| anyhow::anyhow!("You are not inside a challenge channel."))?
        .to_channel(ctx)
        .await
        .or_else(|_| Err(anyhow::anyhow!("You are not inside a challenge channel.")))?
        .guild()
        .ok_or_else(|| anyhow::anyhow!("You are not inside a challenge channel."))?;

    let mut solver_ids = vec![ctx.author().id];

    if let Some(teammate) = teammate1 {
        solver_ids.push(teammate);
    }

    if let Some(teammate) = teammate2 {
        solver_ids.push(teammate);
    }

    if let Some(teammate) = teammate3 {
        solver_ids.push(teammate);
    }

    if let Some(teammate) = teammate4 {
        solver_ids.push(teammate);
    }

    if let Some(teammate) = teammate5 {
        solver_ids.push(teammate);
    }

    if let Some(teammate) = teammate6 {
        solver_ids.push(teammate);
    }

    let approval_embed = CreateEmbed::new()
        .title(format!("New Solve Request"))
        .description(format!("Here is a new CTF solve request submitted by {}", ctx.author().id.mention()))
        .color(0xc22026)
        .thumbnail("https://pbs.twimg.com/profile_images/568451513295441921/9Hm60msK_400x400.png")
        .field("Challenge", &challenge.name, true)
        .field("Category", challenge.category.to_string(), true)
        .field("CTF", competition.channel_id.mention().to_string(), true)
        .field("Flag", format!("```{flag}```"), true);

    let accept_button = CreateButton::new("accept")
        .label("Accept")
        .emoji('✅')
        .style(ButtonStyle::Success);

    let reject_button = CreateButton::new("reject")
        .label("Reject")
        .emoji('❎')
        .style(ButtonStyle::Danger);

    let approval_message = CreateMessage::new()
        .add_embed(approval_embed)
        .button(accept_button)
        .button(reject_button);

    let approval_message = config().server.solve_approvals_channel_id
        .send_message(ctx, approval_message).await?;

    let solve = Solve {
        id: 0,
        challenge_id: challenge.id,
        approval_message_id: approval_message.id,
        flag,
        approval_status: ApprovalStatus::Pending,
    };

    let solve_id = ctx.data().db.create_solve(solve, &solver_ids).await?;

    // mark challenge channel as solved
    let tag_ids = competition_forum_channel.available_tags
        .iter()
        .filter(|t| t.name == challenge.category.to_string() || t.name == "solved")
        .map(|t| t.id);

    challenge_channel.edit_thread(ctx, EditThread::new().applied_tags(tag_ids)).await?;

    ctx.say(format!("Your solve for {} has been recorded with request ID {solve_id}.", challenge.name)).await?;

    Ok(())
}

/// Recieves Component Interaction events and updates solve status if they are an approval button
pub async fn handle_approval_button(context: &Context, cmd_context: &CommandContext, interaction: &ComponentInteraction) -> anyhow::Result<()> {
    if matches!(interaction.data.kind, ComponentInteractionDataKind::Button) {
        let mut message = interaction.message.clone();
        let mut solve = cmd_context.db.get_solve_by_approval_message_id(message.id).await?;

        if solve.approval_status != ApprovalStatus::Pending {
            message.reply(context, format!("solve is alredy {}", solve.approval_status)).await?;
        } else if interaction.data.custom_id == "accept" {
            solve.approval_status = ApprovalStatus::Approved;

            // give participants points for solving
            let points_updates = cmd_context.db.give_points_for_solve(solve.id, config().ranks.points_per_solve).await?;

            // rank people up as necassary
            for points_update in points_updates {
                check_rank_up(context, &cmd_context.db, points_update).await?;
            }

            let edit = EditMessage::new()
                .content(format!("This request is approved by {}", interaction.user.id.mention()))
                .components(Vec::new());

            message.edit(context, edit).await?;


        } else if interaction.data.custom_id == "reject" {
            solve.approval_status = ApprovalStatus::Declined;

            let edit = EditMessage::new()
                .content(format!("This request is declined by {}", interaction.user.id.mention()))
                .components(Vec::new());

            message.edit(context, edit).await?;
        }

        // save updated approval status
        cmd_context.db.update_solve(solve).await?;

        // acknowledge interaction
        interaction.defer(context).await?;
    }

    Ok(())
}