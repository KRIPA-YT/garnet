use chrono::Duration;
use sqlx::{PgPool, postgres::types::PgInterval, query};
use uuid::Uuid;

use crate::{
    auth::token::{Expiry, Token, TokenPair},
    error::auth::AuthError,
    sessions::model::Session,
};
pub(crate) struct SessionRepository {
    pool: PgPool,
}

pub(crate) struct Validities {
    pub access: Duration,
    pub refresh: Duration,
    pub absolute: Duration,
}

impl Validities {
    pub(crate) const fn default() -> Self {
        Self {
            access: Duration::hours(1),
            refresh: Duration::days(14),
            absolute: Duration::days(90),
        }
    }
}

impl SessionRepository {
    pub(crate) const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub(crate) async fn create(
        &self,
        user_id: Uuid,
        title: &str,
        validities: Validities,
    ) -> Result<(Session, TokenPair), AuthError> {
        let access = Token::random();
        let refresh = Token::random();
        let row = query!(
            r#"
                INSERT INTO sessions (user_id, title,
                access_token_hash, access_expires_at,
                refresh_token_hash, refresh_expires_at, absolute_expires_at,
                revoked_at) VALUES ($1, $2, $3, now() + $4, $5, now() + $6, now() + $7, NULL) RETURNING
                id, title, user_id, 
                access_expires_at, refresh_expires_at, absolute_expires_at,
                created_at, revoked_at;
            "#,
            user_id,
            title,
            access.hash(),
            PgInterval::try_from(validities.access).map_err(|_| AuthError::Internal)?,
            refresh.hash(),
            PgInterval::try_from(validities.refresh).map_err(|_| AuthError::Internal)?,
            PgInterval::try_from(validities.absolute).map_err(|_| AuthError::Internal)?,
        )
        .fetch_one(&self.pool)
        .await?;
        let session = Session {
            id: row.id,
            title: row.title,
            user_id: row.user_id,
            absolute_expires_at: row.absolute_expires_at,
            created_at: row.created_at,
            revoked_at: row.revoked_at,
        };
        let access = access.limited(row.access_expires_at);
        let refresh = refresh.limited(row.refresh_expires_at);
        Ok((session, TokenPair { access, refresh }))
    }

    pub(crate) async fn refresh<E: Expiry>(
        &self,
        user_id: &Uuid,
        refresh_token: &Token<E>,
    ) -> Result<TokenPair, AuthError> {
        let new_access = Token::random();
        let new_refresh = Token::random();
        let row = query!(
            r#"
                UPDATE sessions
                SET
                    access_token_hash = $1,
                    access_expires_at = LEAST(
                        now() + INTERVAL '1 hour',
                        absolute_expires_at
                    ),
                    refresh_token_hash = $2,
                    refresh_expires_at = LEAST(
                        now() + INTERVAL '30 days',
                        absolute_expires_at
                    )
                WHERE user_id = $3
                AND refresh_token_hash = $4
                AND refresh_expires_at > now()
                AND absolute_expires_at > now()
                AND revoked_at IS NULL RETURNING
                access_expires_at, refresh_expires_at
            "#,
            new_access.hash(),
            new_refresh.hash(),
            user_id,
            refresh_token.hash(),
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AuthError::InvalidCredentials)?;
        Ok(TokenPair {
            access: new_access.limited(row.access_expires_at),
            refresh: new_refresh.limited(row.refresh_expires_at),
        })
    }

    pub(crate) async fn logout<E: Expiry>(
        &self,
        refresh_token: &Token<E>,
    ) -> Result<(), AuthError> {
        query!(
            r#"
                UPDATE sessions
                SET
                revoked_at = now()
                WHERE refresh_token_hash = $1
            "#,
            refresh_token.hash(),
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AuthError::InvalidCredentials)?;
        Ok(())
    }

    pub(crate) async fn authenticate_access<E: Expiry>(
        &self,
        access_token: &Token<E>,
    ) -> Result<Uuid, AuthError> {
        let row = query!(
            r#"
                SELECT user_id 
                FROM sessions 
                WHERE access_token_hash = $1 
                AND revoked_at IS NULL
            "#,
            access_token.hash()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AuthError::InvalidCredentials)?;
        Ok(row.user_id)
    }
}
