use crate::db::{ChallengeType, Solve};

use super::{CmdContext, Error, competition::get_competition_id_from_ctx};

#[poise::command(slash_command)]
pub async fn solve(
    ctx: CmdContext<'_>,
    #[description = "Name of the challenge that was solved"] challenge_name: String,
    #[description = "Type of the challenge that was solved"] challenge_type: ChallengeType,
    #[description = "Flag of the challenge that was solved"] flag: String,
    // TODO: figure out how to specify teammates
) -> Result<(), Error> {
    let competition_id = get_competition_id_from_ctx(&ctx).await?;
    let solver_ids = vec![ctx.author().id];

    let solve = Solve {
        id: 0,
        competition_id,
        challenge_name,
        challenge_type,
        flag,
        approved: false,
    };

    let solve_id = ctx.data().db.create_solve(solve, &solver_ids).await?;

    Ok(())
}