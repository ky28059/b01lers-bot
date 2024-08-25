use crate::{commands::{add_role_to_user, remove_role_from_user}, config::config, db::{DbContext, PointsUpdate}};

use serenity::all::{Context, CreateMessage, UserId};

pub async fn get_point_cutoffs(db: &DbContext) -> anyhow::Result<Vec<i64>> {
    let mut max_score = db.get_users_by_points(1)
        .await?
        .first()
        .unwrap()
        .points;

    let rank_count = config().ranks.rank_count();

    let mut cutoffs = vec![0; rank_count];

    for i in (0..rank_count).rev() {
        cutoffs[i] = max_score;
        max_score = (max_score as f64 * 0.75) as i64;
    }

    Ok(cutoffs)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Rank {
    Unranked,
    Rank(usize),
}

impl Rank {
    fn from_points(points: i64, cutoffs: &[i64]) -> Rank {
        match cutoffs.binary_search(&points) {
            Ok(i) => Rank::Rank(i),
            Err(i) => if i == 0 {
                // the points is less than the lowest rank
                Rank::Unranked
            } else {
                Rank::Rank(i - 1)
            }
        }
    }

    fn rank_name(&self) -> Option<&'static str> {
        match self {
            Self::Unranked => None,
            Self::Rank(i) => Some(&config().ranks.rank_names[*i])
        }
    }
}

pub async fn check_rank_up(context: &Context, db: &DbContext, points_update: PointsUpdate) -> anyhow::Result<()> {
    let cutoffs = get_point_cutoffs(db).await?;

    let old_rank = Rank::from_points(points_update.old_points, &cutoffs);
    let new_rank = Rank::from_points(points_update.new_points, &cutoffs);

    if new_rank > old_rank {
        let user = points_update.user_id.to_user(context).await?;

        if let Some(new_rank_name) = new_rank.rank_name() {
            add_role_to_user(context, points_update.user_id, new_rank_name).await?;

            // create rank up message
            let message = CreateMessage::new()
                .content(format!("{} has reached the rank {}!", user.name, new_rank_name));

            config().server.rank_up_channel.send_message(context, message).await?;
        }

        if let Some(old_rank_name) = old_rank.rank_name() {
            remove_role_from_user(context, points_update.user_id, old_rank_name).await?; 
        }
    }

    Ok(())
}

pub async fn give_points(context: &Context, db: &DbContext, user_id: UserId, points: i64) -> anyhow::Result<()> {
    let points_update = db.give_user_points(user_id, points).await?;

    check_rank_up(context, db, points_update).await?;

    Ok(())
}