use serde::Deserialize;
use sqlx::{PgPool, query, query_as};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{model::Item, users::repository::UserTxExt as _};

pub(crate) struct ItemRepository {
    pool: PgPool,
}

pub(crate) enum DeleteResult {
    Deleted,
    NotFound,
    InternalError,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum GetItemError {
    #[error("Item not found")]
    NotFound,
    #[error("Sqlx error: $1")]
    Sqlx(#[from] sqlx::Error),
}

pub(crate) enum ItemInsertResult {
    Inserted,
    Duplicate,
    ListNotFound,
    InternalError,
}

#[derive(Deserialize, ToSchema)]

pub(crate) struct PatchItemParams {
    list_id: Option<Uuid>,
    title: Option<String>,
    pinned: Option<bool>,
    checked: Option<bool>,
}

pub(crate) enum ItemUpdateResult {
    Updated,
    NotFound,
    ListNotFound,
    InternalError,
}
impl ItemRepository {
    pub(crate) async fn get_items(
        &self,
        list_id: &Uuid,
        user_id: &Uuid,
    ) -> anyhow::Result<Vec<Item>> {
        let mut tx = self.pool.user_tx(user_id).await?;
        let items = query_as!(
            Item,
            r#"
                SELECT id, list_id, title, checked, pinned
                FROM items 
                WHERE list_id=$1
            "#,
            list_id
        )
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(items)
    }

    pub(crate) async fn get_item(
        &self,
        item_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<Item, GetItemError> {
        let mut tx = self.pool.user_tx(user_id).await?;
        let item = query_as!(
            Item,
            r#"
                SELECT id, list_id, title, checked, pinned
                FROM items WHERE id=$1
            "#,
            item_id
        )
        .fetch_one(&mut *tx)
        .await;
        match item {
            Ok(item) => {
                tx.commit().await?;
                Ok(item)
            }
            Err(sqlx::Error::RowNotFound) => Err(GetItemError::NotFound),
            Err(err) => Err(GetItemError::Sqlx(err)),
        }
    }

    pub(crate) async fn insert_item(&self, item: Item, user_id: &Uuid) -> ItemInsertResult {
        let mut tx = match self.pool.user_tx(user_id).await {
            Ok(tx) => tx,
            Err(_) => return ItemInsertResult::InternalError,
        };
        let res = query!(
            r#"
                INSERT INTO items (id, list_id, title, pinned, checked) VALUES ($1, $2, $3, $4, $5)
            "#,
            item.id,
            item.list_id,
            item.title,
            item.pinned,
            item.checked,
        )
        .execute(&mut *tx)
        .await;
        if let Err(err) = res {
            let Some(database_err) = err.into_database_error() else {
                return ItemInsertResult::InternalError;
            };
            if database_err.is_unique_violation() {
                return ItemInsertResult::Duplicate;
            }
            if database_err.is_foreign_key_violation() {
                return ItemInsertResult::ListNotFound;
            }
            return ItemInsertResult::InternalError;
        }

        match tx.commit().await {
            Ok(()) => ItemInsertResult::Inserted,
            Err(_) => ItemInsertResult::InternalError,
        }
    }

    pub(crate) async fn delete_item(&self, id: &Uuid, user_id: &Uuid) -> DeleteResult {
        let mut tx = match self.pool.user_tx(user_id).await {
            Ok(tx) => tx,
            Err(_) => return DeleteResult::InternalError,
        };
        let res = query!(
            r#"
                DELETE FROM items WHERE id=$1
            "#,
            id
        )
        .execute(&mut *tx)
        .await;
        let Ok(res) = res else {
            return DeleteResult::InternalError;
        };

        if res.rows_affected() == 0 {
            // If the entry did not exist, the rows affected will be 0
            return DeleteResult::NotFound;
        }

        match tx.commit().await {
            Ok(()) => DeleteResult::Deleted,
            Err(_) => DeleteResult::InternalError,
        }
    }

    pub(crate) async fn update_item(
        &self,
        id: &Uuid,
        user_id: &Uuid,
        params: PatchItemParams,
    ) -> ItemUpdateResult {
        let mut tx = match self.pool.user_tx(user_id).await {
            Ok(tx) => tx,
            Err(_) => return ItemUpdateResult::InternalError,
        };
        let res = query!(
            r#"
            UPDATE items
            SET
                list_id = COALESCE($1, list_id),
                title = COALESCE($2, title),
                pinned = COALESCE($3, pinned),
                checked = COALESCE($4, checked)
            WHERE id = $5
            "#,
            params.list_id,
            params.title,
            params.pinned,
            params.checked,
            id,
        )
        .execute(&mut *tx)
        .await;
        match res {
            Ok(result) if result.rows_affected() > 0 => return ItemUpdateResult::NotFound,
            Ok(_) => {}
            Err(err) => match err.into_database_error() {
                Some(database_err) if database_err.is_foreign_key_violation() => {
                    return ItemUpdateResult::ListNotFound;
                }
                _ => return ItemUpdateResult::InternalError,
            },
        }

        match tx.commit().await {
            Ok(()) => ItemUpdateResult::Updated,
            Err(_) => ItemUpdateResult::InternalError,
        }
    }

    pub(crate) const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
