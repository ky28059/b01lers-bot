use serenity::all::{ChannelId, MessageId, UserId};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

pub use competition::{Competition, BingoSquare};
pub use user::User;
pub use solve::{ChallengeType, ApprovalStatus, Solve};
use competition::CompetitionRaw;
use user::UserRaw;
use solve::SolveRaw;

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

    pub async fn create_user(&self, user: User) -> Result<(), anyhow::Error> {
        let user_raw: UserRaw = user.into();
        sqlx::query!(
            "INSERT INTO users (id, email) VALUES (?, ?)",
            user_raw.id,
            user_raw.email,
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
}