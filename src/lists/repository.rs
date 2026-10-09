use serde::Deserialize;
use sqlx::{PgPool, query, query_as};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    error::repository::RepositoryError, lists::model::List, users::repository::UserTxExt as _,
};

pub(crate) struct ListRepository {
    pool: PgPool,
}

#[derive(Deserialize, ToSchema)]
pub(crate) struct PatchListParams {
    title: Option<String>,
    pinned: Option<bool>,
}

impl ListRepository {
    pub(crate) async fn get_lists(&self, user_id: &Uuid) -> Result<Vec<List>, RepositoryError> {
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

    pub(crate) async fn insert_list(
        &self,
        list: List,
        owner: &Uuid,
    ) -> Result<(), RepositoryError> {
        let mut tx = self.pool.user_tx(owner).await?;

        // Insert the list.
        query!(
            r#"
                INSERT INTO lists (id, title, pinned)
                VALUES ($1, $2, $3)
            "#,
            list.id,
            list.title,
            list.pinned,
        )
        .execute(&mut *tx)
        .await
        .map_err(RepositoryError::map_insert)?;

        // Insert the owner membership.
        query!(
            r#"
                INSERT INTO members (list_id, user_id, role)
                VALUES ($1, $2, 'owner')
            "#,
            list.id,
            *owner,
        )
        .execute(&mut *tx)
        .await
        .map_err(RepositoryError::map_insert)?;

        // Commit both inserts together.
        tx.commit().await?;

        Ok(())
    }

    pub(crate) async fn delete_list(
        &self,
        list_id: &Uuid,
        owner: &Uuid,
    ) -> Result<(), RepositoryError> {
        let mut tx = self.pool.user_tx(owner).await?;

        let res = query!(
            r#"
                DELETE FROM lists
                WHERE id = $1
            "#,
            list_id
        )
        .execute(&mut *tx)
        .await?;

        if res.rows_affected() == 0 {
            return Err(RepositoryError::ListNotFound);
        }

        tx.commit().await?;

        Ok(())
    }
    pub(crate) async fn update_list(
        &self,
        list_id: &Uuid,
        user_id: &Uuid,
        params: &PatchListParams,
    ) -> Result<(), RepositoryError> {
        let mut tx = self.pool.user_tx(user_id).await?;

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
        .await?;

        if res.rows_affected() == 0 {
            return Err(RepositoryError::ListNotFound);
        }

        tx.commit().await?;
        Ok(())
    }

    pub(crate) const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
