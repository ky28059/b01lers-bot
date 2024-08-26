use serenity::all::{ChannelId, MessageId, UserId};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

pub use competition::{Competition, BingoSquare};
pub use user::User;
pub use solve::{ChallengeType, ApprovalStatus, Solve};
use competition::CompetitionRaw;
use user::UserRaw;
use solve::SolveRaw;

use crate::points::Rank;

mod competition;
mod user;
mod solve;

pub struct DbContext {
    pool: SqlitePool,
}

struct OutputId { id: i64 }

impl DbContext {
    /// Connects to the database at `url`
    pub async fn connect(url: &str) -> Result<Self, anyhow::Error> {
        // TODO: idk what is a good value for max connections
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(url)
            .await?;

        Ok(DbContext { pool })
    }

    pub async fn create_competition(&self, competition: Competition) -> Result<(), anyhow::Error> {
        let competition_raw: CompetitionRaw = competition.into();
        sqlx::query!(
            "INSERT INTO competition (channel_id, name, bingo) VALUES (?, ?, ?)",
            competition_raw.channel_id,
            competition_raw.name,
            competition_raw.bingo
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_competition(
        &self,
        channel_id: ChannelId,
    ) -> Result<Competition, anyhow::Error> {
        let channel_id = channel_id.get() as i64;
        let competition_raw = sqlx::query_as!(
            CompetitionRaw,
            "SELECT * FROM competition WHERE channel_id = ?",
            channel_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(competition_raw.into())
    }

    pub async fn update_competition(&self, competition: Competition) -> Result<(), anyhow::Error> {
        let competition_raw: CompetitionRaw = competition.into();
        sqlx::query!(
            "UPDATE competition SET name = ?, bingo = ? WHERE channel_id = ?",
            competition_raw.name,
            competition_raw.bingo,
            competition_raw.channel_id,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn ensure_user_is_created(&self, user_id: UserId) {
        let user_id = user_id.get() as i64;
        // ignore error if user already exists
        let _ = sqlx::query!(
            "INSERT INTO users (id, email, points) VALUES (?, NULL, 0)",
            user_id,
        ).execute(&self.pool).await;
    }

    pub async fn verify_user(&self, user_id: UserId, email: &str) -> Result<(), anyhow::Error> {
        self.ensure_user_is_created(user_id).await;

        let user_id = user_id.get() as i64;
        sqlx::query!(
            "UPDATE users SET email = ? WHERE id = ?",
            email,
            user_id,
        ).execute(&self.pool).await?;

        Ok(())
    }

    /// Gives the given user points, creating them if they don't exist
    /// 
    /// # Returns
    /// 
    /// Returns the user's points
    pub async fn give_user_points(&self, user_id: UserId, points: i64) -> Result<PointsUpdate, anyhow::Error> {
        self.ensure_user_is_created(user_id).await;

        let user_id = user_id.get() as i64;
        let result = sqlx::query_as!(
            PointsUpdateRaw,
            "UPDATE users SET points = points + ? WHERE id = ? RETURNING id, points, rank",
            points,
            user_id,
        ).fetch_one(&self.pool).await?;

        Ok(PointsUpdate::from_raw(result, points))
    }

    pub async fn set_rank(&self, user_id: UserId, rank: Rank) -> Result<(), anyhow::Error> {
        let user_id = user_id.get() as i64;
        let rank: Option<i64> = rank.into();
        sqlx::query!(
            "UPDATE users SET rank = ? WHERE id = ?",
            rank,
            user_id,
        ).execute(&self.pool).await?;

        Ok(())
    }

    pub async fn get_user_by_id(&self, id: UserId) -> Result<User, anyhow::Error> {
        let id = id.get() as i64;
        let user_raw = sqlx::query_as!(
            UserRaw,
            "SELECT * FROM users WHERE id = ?",
            id,
        ).fetch_one(&self.pool).await?;

        Ok(user_raw.into())
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<User, anyhow::Error> {
        let user_raw = sqlx::query_as!(
            UserRaw,
            "SELECT * FROM users WHERE email = ?",
            email,
        ).fetch_one(&self.pool).await?;

        Ok(user_raw.into())
    }

    /// Gets the top `count` users with the highest points
    pub async fn get_users_by_points(&self, count: u32) -> Result<Vec<User>, anyhow::Error> {
        Ok(sqlx::query_as!(
            UserRaw,
            "SELECT * FROM users ORDER BY points DESC LIMIT ?",
            count,
        ).map(|user| User::from(user))
            .fetch_all(&self.pool).await?)
    }

    /// Creates a new solve solved by the given users and returns the solve id
    pub async fn create_solve(&self, solve: Solve, users: &[UserId]) -> Result<i64, anyhow::Error> {
        let solve_raw: SolveRaw = solve.into();

        let mut transaction = self.pool.begin().await?;

        let OutputId { id: solve_id } = sqlx::query_as!(
            OutputId,
            "INSERT INTO solves (competition_id, approval_message_id, challenge_name, challenge_type, flag, approved)
            VALUES (?, ?, ?, ?, ?, ?) RETURNING id",
            solve_raw.competition_id,
            solve_raw.approval_message_id,
            solve_raw.challenge_name,
            solve_raw.challenge_type,
            solve_raw.flag,
            solve_raw.approved,
        ).fetch_one(&mut *transaction).await?;

        for user_id in users {
            let user_id = user_id.get() as i64;
            sqlx::query!(
                "INSERT INTO user_solves (user_id, solve_id) VALUES (?, ?)",
                user_id,
                solve_id,
            ).execute(&mut *transaction).await?;
        }

        transaction.commit().await?;

        Ok(solve_id)
    }

    pub async fn get_solve_by_approval_message_id(&self, message_id: MessageId) -> Result<Solve, anyhow::Error> {
        let id = message_id.get() as i64;
        let solve_raw = sqlx::query_as!(
            SolveRaw,
            "SELECT * FROM solves WHERE approval_message_id = ?",
            id,
        ).fetch_one(&self.pool).await?;

        Ok(solve_raw.into())
    }

    pub async fn get_solves_for_user(&self, user_id: UserId) -> Result<Vec<Solve>, anyhow::Error> {
        let id = user_id.get() as i64;
        let solves = sqlx::query_as!(
            SolveRaw,
            "SELECT solves.* FROM user_solves
            INNER JOIN solves ON solves.id = user_solves.solve_id
            WHERE user_solves.user_id = ? AND solves.approved = ?",
            id,
            ApprovalStatus::Approved as i64,
        ).map(|solve| Solve::from(solve))
            .fetch_all(&self.pool).await?;

        Ok(solves)
    }

    /// Updates the challenge name, type, flag, and approval status of the given solve
    pub async fn update_solve(&self, solve: Solve) -> Result<(), anyhow::Error> {
        let solve_raw: SolveRaw = solve.into();

        sqlx::query!(
            "UPDATE solves SET challenge_name = ?, challenge_type = ?, flag = ?, approved = ? WHERE id = ?",
            solve_raw.challenge_name,
            solve_raw.challenge_type,
            solve_raw.flag,
            solve_raw.approved,
            solve_raw.id,
        ).execute(&self.pool).await?;

        Ok(())
    }

    /// Gives all the participants of this solve some points
    pub async fn give_points_for_solve(&self, solve_id: i64, points: i64) -> Result<Vec<PointsUpdate>, anyhow::Error> {
        let result = sqlx::query_as!(
            PointsUpdateRaw,
            "UPDATE users SET points = points + ? WHERE id IN
            (SELECT user_id FROM user_solves WHERE solve_id = ?)
            RETURNING id, points, rank",
            points,
            solve_id,
        ).map(|update| PointsUpdate::from_raw(update, points))
            .fetch_all(&self.pool).await?;

        Ok(result)
    }
}

struct PointsUpdateRaw {
    /// Id of user with changed points
    id: i64,
    /// New points for the user
    points: i64,
    /// Old rank for user
    rank: Option<i64>,
}

/// Represents a change in points that occured in a sql query
#[derive(Debug)]
pub struct PointsUpdate {
    pub user_id: UserId,
    pub old_points: i64,
    pub new_points: i64,
    pub old_rank: Rank,
}

impl PointsUpdate {
    fn from_raw(update: PointsUpdateRaw, point_increase: i64) -> Self {
        PointsUpdate {
            user_id: UserId::new(update.id as u64),
            old_points: update.points - point_increase,
            new_points: update.points,
            old_rank: update.rank.into(),
        }
    }
}