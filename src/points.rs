use crate::{commands::add_role_to_user, db::DbContext, RANKS_NAMES, RANK_COUNT, RANK_UP_CHANNEL};

use serenity::all::{Context, CreateMessage, UserId};

pub async fn get_point_cutoffs(db: &DbContext) -> anyhow::Result<[i64; RANK_COUNT]> {
    let mut max_score = db.get_users_by_points(1)
        .await?
        .first()
        .unwrap()
        .points;

    let mut cutoffs = [0; RANK_COUNT];

    for i in (0..RANK_COUNT).rev() {
        cutoffs[i] = max_score;
        max_score = (max_score as f64 * 0.75) as i64;
    }

    Ok(cutoffs)
}

pub async fn check_rank_up(context: &Context, db: &DbContext, user_id: UserId, old_points: i64, new_points: i64) -> anyhow::Result<()> {
    let cutoffs = get_point_cutoffs(db).await?;

    let old_rank = match cutoffs.binary_search(&old_points) {
        Ok(n) => n,
        Err(n) => n,
    };

    let new_rank = match cutoffs.binary_search(&new_points) {
        Ok(n) => n,
        Err(n) => n,
    };

    if new_rank > old_rank {
        let user = user_id.to_user(context).await?;

        // TODO: reanable later
        // add_role_to_user(context, user_id, RANKS_NAMES[new_rank]).await?;
        // TODO: remove old role

        let message = CreateMessage::new()
            .content(format!("{} has reached the rank {}!", user.name, RANKS_NAMES[new_rank]));

        RANK_UP_CHANNEL.send_message(context, message).await?;
    }

    Ok(())
}

pub async fn give_points(context: &Context, db: &DbContext, user_id: UserId, points: i64) -> anyhow::Result<()> {
    let new_points = db.give_user_points(user_id, points).await?;
    let old_points = new_points - points;

    check_rank_up(context, db, user_id, old_points, new_points).await?;

    Ok(())
}