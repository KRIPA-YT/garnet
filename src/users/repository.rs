use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    auth::password::Password,
    error::auth::AuthError,
    users::model::{Discriminator, Email, User, Username},
};

pub(crate) struct UserRepository {
    pool: PgPool,
}

pub(crate) struct UserLoginRow {
    pub id: Uuid,
    pub password_hash: String,
}

pub(crate) trait UserTxExt<DB: sqlx::Database> {
    async fn user_tx(&self, user_id: &Uuid) -> Result<Transaction<'_, DB>, sqlx::Error>;
}

impl UserTxExt<Postgres> for sqlx::Pool<Postgres> {
    async fn user_tx(&self, user_id: &Uuid) -> Result<Transaction<'_, Postgres>, sqlx::Error> {
        let mut tx = self.begin().await?;

        sqlx::query!(
            "SELECT set_config('app.user_id', $1, true)",
            user_id.to_string()
        )
        .fetch_one(&mut *tx)
        .await?;

        Ok(tx)
    }
}

impl UserRepository {
    pub(crate) const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub(crate) async fn create(
        &self,
        email: Email,
        password: Password,
        username: Username,
        discriminator: Discriminator,
    ) -> Result<User, AuthError> {
        let row = sqlx::query!(
            r#"
                INSERT INTO users (username, discriminator, email, password_hash) VALUES ($1, $2, $3, $4) RETURNING id, created_at
            "#,
            &username.get(),
            &discriminator.get(),
            &email.get(),
            &password.hash().await.ok_or(AuthError::Internal)?
        ).fetch_one(&self.pool).await
        .map_err(|err| {
            match err {
                sqlx::Error::Database(database_err) if database_err.is_unique_violation() => {
                    AuthError::Conflict
                }
                _ => AuthError::Internal,
            }
        })?;
        Ok(User {
            id: row.id,
            username,
            discriminator,
            email,
            created_at: row.created_at,
        })
    }

    pub(crate) async fn find_for_login(
        &self,
        email: &Email,
    ) -> Result<Option<UserLoginRow>, AuthError> {
        sqlx::query_as!(
            UserLoginRow,
            r#"
                SELECT id, password_hash
                FROM users
                WHERE email = $1
            "#,
            email.get(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(AuthError::Database)
    }
}
