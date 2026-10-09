use serde::Deserialize;
use sqlx::{PgPool, query, query_as};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{error::repository::RepositoryError, model::Item, users::repository::UserTxExt as _};

pub(crate) struct ItemRepository {
    pool: PgPool,
}

#[derive(Deserialize, ToSchema)]

pub(crate) struct PatchItemParams {
    list_id: Option<Uuid>,
    title: Option<String>,
    pinned: Option<bool>,
    checked: Option<bool>,
}

impl ItemRepository {
    pub(crate) async fn get_items(
        &self,
        list_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<Vec<Item>, RepositoryError> {
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
    ) -> Result<Item, RepositoryError> {
        let mut tx = self.pool.user_tx(user_id).await?;
        let item = query_as!(
            Item,
            r#"
                SELECT id, list_id, title, checked, pinned
                FROM items WHERE id=$1
            "#,
            item_id
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(RepositoryError::ItemNotFound)?;

        tx.commit().await?;
        Ok(item)
    }

    pub(crate) async fn insert_item(
        &self,
        item: Item,
        user_id: &Uuid,
    ) -> Result<(), RepositoryError> {
        let mut tx = self.pool.user_tx(user_id).await?;
        query!(
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
        .await
        .map_err(RepositoryError::map_insert)?;

        tx.commit().await?;

        Ok(())
    }

    pub(crate) async fn delete_item(
        &self,
        id: &Uuid,
        user_id: &Uuid,
    ) -> Result<(), RepositoryError> {
        let mut tx = self.pool.user_tx(user_id).await?;
        let res = query!(
            r#"
                DELETE FROM items WHERE id=$1
            "#,
            id
        )
        .execute(&mut *tx)
        .await?;

        if res.rows_affected() == 0 {
            // If the entry did not exist, the rows affected will be 0
            return Err(RepositoryError::ItemNotFound);
        }

        tx.commit().await?;

        Ok(())
    }

    pub(crate) async fn update_item(
        &self,
        id: &Uuid,
        user_id: &Uuid,
        params: PatchItemParams,
    ) -> Result<(), RepositoryError> {
        let mut tx = self.pool.user_tx(user_id).await?;
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
        .await?;
        if res.rows_affected() == 0 {
            return Err(RepositoryError::ItemNotFound);
        }

        tx.commit().await?;

        Ok(())
    }

    pub(crate) const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
