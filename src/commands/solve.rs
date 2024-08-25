use serenity::all::{ButtonStyle, CreateButton, CreateEmbed, CreateMessage, Mentionable, Context, ComponentInteraction, ComponentInteractionDataKind, EditMessage};

use crate::config::config;
use crate::db::{ApprovalStatus, ChallengeType, Solve};
use crate::points::check_rank_up;

use super::{CmdContext, CommandContext, Error, competition::get_competition_from_ctx};

#[poise::command(slash_command)]
pub async fn solve(
    ctx: CmdContext<'_>,
    #[description = "Name of the challenge that was solved"] challenge_name: String,
    #[description = "Type of the challenge that was solved"] challenge_type: ChallengeType,
    #[description = "Flag of the challenge that was solved"] flag: String,
    // TODO: figure out how to specify teammates
) -> Result<(), Error> {
    let competition = get_competition_from_ctx(&ctx).await?;
    let solver_ids = vec![ctx.author().id];

    let approval_embed = CreateEmbed::new()
        .title(format!("New Solve Request"))
        .description(format!("Here is a new CTF solve request submitted by {}", ctx.author().id.mention()))
        .color(0xc22026)
        .thumbnail("https://pbs.twimg.com/profile_images/568451513295441921/9Hm60msK_400x400.png")
        .field("Challenge", &challenge_name, true)
        .field("Category", challenge_type.to_string(), true)
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
        competition_id: competition.channel_id,
        approval_message_id: approval_message.id,
        challenge_name: challenge_name.clone(),
        challenge_type,
        flag,
        approved: ApprovalStatus::Pending,
    };

    let solve_id = ctx.data().db.create_solve(solve, &solver_ids).await?;

    ctx.say(format!("Your solve for {challenge_name} has been recorded with request ID {solve_id}.")).await?;

    Ok(())
}

/// Recieves Component Interaction events and updates solve status if they are an approval button
pub async fn handle_approval_button(context: &Context, cmd_context: &CommandContext, interaction: &ComponentInteraction) -> anyhow::Result<()> {
    if matches!(interaction.data.kind, ComponentInteractionDataKind::Button) {
        let mut message = interaction.message.clone();
        let mut solve = cmd_context.db.get_solve_by_approval_message_id(message.id).await?;

        if solve.approved != ApprovalStatus::Pending {
            message.reply(context, format!("solve is alredy {}", solve.approved)).await?;
        } else if interaction.data.custom_id == "accept" {
            solve.approved = ApprovalStatus::Approved;

            let edit = EditMessage::new()
                .content(format!("This request is approved by {}", interaction.user.id.mention()))
                .components(Vec::new());

            message.edit(context, edit).await?;
        } else if interaction.data.custom_id == "reject" {
            solve.approved = ApprovalStatus::Declined;

            let edit = EditMessage::new()
                .content(format!("This request is declined by {}", interaction.user.id.mention()))
                .components(Vec::new());

            message.edit(context, edit).await?;
        }

        // give participants points for solving
        let points_updates = cmd_context.db.give_points_for_solve(solve.id, config().ranks.points_per_solve).await?;

        // save updated approval status
        cmd_context.db.update_solve(solve).await?;

        // rank people up as necassary
        for points_update in points_updates {
            check_rank_up(context, &cmd_context.db, points_update).await?;
        }

        // acknowledge interaction
        interaction.defer(context).await?;
    }

    Ok(())
}