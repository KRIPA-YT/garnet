use serde::Deserialize;
use sqlx::{PgPool, query, query_as};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{model::List, users::repository::UserTxExt as _};

pub(crate) struct ListRepository {
    pool: PgPool,
}

pub(crate) enum ListInsertResult {
    Inserted,
    Duplicate,
    InternalError,
}

pub(crate) enum DeleteResult {
    Deleted,
    NotFound,
    InternalError,
}

pub(crate) enum ListUpdateResult {
    Updated,
    NotFound,
    InternalError,
}

#[derive(Deserialize, ToSchema)]
pub(crate) struct PatchListParams {
    title: Option<String>,
    pinned: Option<bool>,
}

impl ListRepository {
    pub(crate) async fn get_lists(&self, user_id: &Uuid) -> anyhow::Result<Vec<List>> {
        let mut tx = self.pool.user_tx(user_id).await?;
        let lists = query_as!(
            List,
            r#"
                SELECT l.id, l.title, l.pinned FROM lists l;
            "#,
        )
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(lists)
    }

    pub(crate) async fn insert_list(&self, list: List, owner: &Uuid) -> ListInsertResult {
        let mut tx = match self.pool.user_tx(owner).await {
            Ok(tx) => tx,
            Err(_) => return ListInsertResult::InternalError,
        };

        // Insert the list.
        let res = sqlx::query!(
            r#"
                INSERT INTO lists (id, title, pinned)
                VALUES ($1, $2, $3)
            "#,
            list.id,
            list.title,
            list.pinned,
        )
        .execute(&mut *tx)
        .await;

        if let Err(err) = res {
            return match err.as_database_error() {
                Some(db_err) if db_err.is_unique_violation() => ListInsertResult::Duplicate,
                _ => ListInsertResult::InternalError,
            };
        }

        // Insert the owner membership.
        let res = sqlx::query!(
            r#"
                INSERT INTO members (list_id, user_id, role)
                VALUES ($1, $2, 'owner')
            "#,
            list.id,
            *owner,
        )
        .execute(&mut *tx)
        .await;

        if let Err(err) = res {
            return match err.as_database_error() {
                Some(db_err) if db_err.is_unique_violation() => ListInsertResult::Duplicate,
                _ => ListInsertResult::InternalError,
            };
        }

        // Commit both inserts together.
        match tx.commit().await {
            Ok(()) => ListInsertResult::Inserted,
            Err(_) => ListInsertResult::InternalError,
        }
    }

    pub(crate) async fn delete_list(&self, list_id: &Uuid, owner: &Uuid) -> DeleteResult {
        let mut tx = match self.pool.user_tx(owner).await {
            Ok(tx) => tx,
            Err(_) => return DeleteResult::InternalError,
        };

        let res = query!(
            r#"
                    DELETE FROM lists
                    WHERE id = $1
                "#,
            list_id
        )
        .execute(&mut *tx)
        .await;

        let Ok(res) = res else {
            return DeleteResult::InternalError;
        };

        let result = if res.rows_affected() == 0 {
            DeleteResult::NotFound
        } else {
            DeleteResult::Deleted
        };

        match tx.commit().await {
            Ok(()) => result,
            Err(_) => DeleteResult::InternalError,
        }
    }
    pub(crate) async fn update_list(
        &self,
        list_id: &Uuid,
        user_id: &Uuid,
        params: &PatchListParams,
    ) -> ListUpdateResult {
        let mut tx = match self.pool.user_tx(user_id).await {
            Ok(tx) => tx,
            Err(_) => return ListUpdateResult::InternalError,
        };

        let res = query!(
            r#"
                UPDATE lists
                SET
                    title = COALESCE($1, title),
                    pinned = COALESCE($2, pinned)
                WHERE id = $3
            "#,
            params.title,
            params.pinned,
            list_id,
        )
        .execute(&mut *tx)
        .await;

        let result = match res {
            Ok(result) if result.rows_affected() > 0 => ListUpdateResult::Updated,
            Ok(_) => ListUpdateResult::NotFound,
            Err(_) => return ListUpdateResult::InternalError,
        };

        match tx.commit().await {
            Ok(()) => result,
            Err(_) => ListUpdateResult::InternalError,
        }
    }

    pub(crate) const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
