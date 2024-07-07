use serenity::all::{ChannelId, UserId};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

pub use competition::{Competition, BingoSquare};
pub use user::User;
use competition::CompetitionRaw;
use user::UserRaw;

mod competition;
mod user;

pub struct DbContext {
    pool: SqlitePool,
}

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
}