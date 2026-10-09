use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::prelude::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize, ToSchema)]
pub(crate) struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,

    pub expires_at: DateTime<Utc>,

    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}
