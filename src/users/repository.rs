use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::ToSchema;
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
        #[derive(Serialize, ToSchema)]
        struct UserResponse {
            pub id: Uuid,
            pub created_at: DateTime<Utc>,
        }

        let user = sqlx::query_as!(
            UserResponse,
            r#"
                INSERT INTO users (username, discriminator, email, password_hash) VALUES ($1, $2, $3, $4) RETURNING id, created_at
            "#,
            &username.get(),
            &discriminator.get(),
            &email.get(),
            &password.hash().await.ok_or(AuthError::Internal)?
        ).fetch_one(&self.pool).await;
        match user {
            Ok(row) => Ok(User {
                id: row.id,
                username,
                discriminator,
                email,
                created_at: row.created_at,
            }),
            Err(err) => {
                if let Some(database_err) = err.into_database_error()
                    && database_err.is_unique_violation()
                {
                    Err(AuthError::Conflict)
                } else {
                    Err(AuthError::Internal)
                }
            }
        }
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
