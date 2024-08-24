use crate::{db::DbContext, RANK_COUNT};

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