use chrono::{DateTime, Duration, Utc};
use rand::{RngExt, rng};
use sha2::{Digest as _, Sha256};
use sqlx::{PgPool, postgres::types::PgInterval, prelude::FromRow, query_as};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub(crate) enum SessionError {
    #[error("Sqlx error: $1")]
    Sqlx(#[from] sqlx::error::Error),
    #[error("Params duration overflow")]
    DurationOverflow,
    #[error("Refresh token expired or revoked")]
    ExpiredOrRevoked,
}

#[derive(Debug, FromRow)]
pub(crate) struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,

    pub access_token_hash: Vec<u8>,
    pub access_expires_at: DateTime<Utc>,

    pub refresh_token_hash: Vec<u8>,
    pub refresh_expires_at: DateTime<Utc>,
    pub absolute_expires_at: DateTime<Utc>,

    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

#[allow(clippy::struct_field_names)]
pub(crate) struct SessionParams {
    pub access_validity: Duration,
    pub refresh_validity: Duration,
    pub absolute_validity: Duration,
}

pub(crate) struct TokenPair {
    pub access: String,
    pub refresh: String,
}

impl SessionParams {
    pub(crate) const fn default() -> Self {
        Self {
            access_validity: Duration::hours(1),
            refresh_validity: Duration::days(14),
            absolute_validity: Duration::days(90),
        }
    }
}

impl Session {
    pub(crate) async fn create(
        pool: &PgPool,
        user_id: Uuid,
        title: String,
        session_params: SessionParams,
    ) -> Result<(Self, TokenPair), SessionError> {
        let access_token = random_ascii(25);
        let refresh_token = random_ascii(25);
        let session = query_as!(
            Session,
            r#"
            INSERT INTO sessions (user_id, title,
            access_token_hash, access_expires_at,
            refresh_token_hash, refresh_expires_at, absolute_expires_at,
            revoked_at) VALUES ($1, $2, $3, now() + $4, $5, now() + $6, now() + $7, NULL) RETURNING
            id, title, user_id, 
            access_token_hash, access_expires_at,
            refresh_token_hash, refresh_expires_at, absolute_expires_at,
            created_at, revoked_at;
            "#,
            user_id,
            title,
            Sha256::digest(access_token.as_bytes()).to_vec(),
            PgInterval::try_from(session_params.access_validity).map_err(|_| {
                println!("access");
                SessionError::DurationOverflow
            })?,
            Sha256::digest(refresh_token.as_bytes()).to_vec(),
            PgInterval::try_from(session_params.refresh_validity).map_err(|_| {
                println!("refresh");
                SessionError::DurationOverflow
            })?,
            PgInterval::try_from(session_params.absolute_validity).map_err(|_| {
                println!("absolute");
                SessionError::DurationOverflow
            })?,
        )
        .fetch_one(pool)
        .await?;
        Ok((
            session,
            TokenPair {
                access: access_token,
                refresh: refresh_token,
            },
        ))
    }

    pub(crate) async fn refresh(
        pool: &PgPool,
        user_id: Uuid,
        refresh_token: String,
    ) -> Result<(Self, TokenPair), SessionError> {
        let new_access_token = random_ascii(25);
        let new_refresh_token = random_ascii(25);
        let session = query_as!(
            Session,
            r#"
            UPDATE sessions
            SET
                access_token_hash = $1,
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
            id, user_id, title,
            access_token_hash, access_expires_at,
            refresh_token_hash, refresh_expires_at, absolute_expires_at,
            created_at, revoked_at;
        "#,
            Sha256::digest(new_access_token.as_bytes()).to_vec(),
            Sha256::digest(new_refresh_token.as_bytes()).to_vec(),
            user_id,
            Sha256::digest(refresh_token.as_bytes()).to_vec(),
        )
        .fetch_optional(pool)
        .await?
        .ok_or(SessionError::ExpiredOrRevoked)?;
        Ok((
            session,
            TokenPair {
                access: new_access_token,
                refresh: new_refresh_token,
            },
        ))
    }
}

const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                        abcdefghijklmnopqrstuvwxyz\
                        0123456789)(*&^%$#@!~";

fn random_ascii(len: i32) -> String {
    let mut rng = rng();
    (0..len)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            #[allow(clippy::indexing_slicing)]
            char::from(CHARSET[idx])
        })
        .collect()
}
