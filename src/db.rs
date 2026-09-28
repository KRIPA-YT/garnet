use std::time::Duration;

use serde::Deserialize;
use sqlx::{PgPool, postgres::PgPoolOptions, query, query_as};
use utoipa::IntoParams;
use uuid::Uuid;

use crate::model::{Item, List};

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
pub(crate) async fn establish_connection(db_url: &str) -> anyhow::Result<PgPool> {
    // Production-ready pool configuration
    let pool = PgPoolOptions::new()
        .max_connections(50)
        .acquire_timeout(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(10))
        .connect(db_url)
        .await?;

    Ok(pool)
}

pub(crate) async fn get_lists(pool: &PgPool) -> anyhow::Result<Vec<List>> {
    let lists = query_as!(
        List,
        r#"
        SELECT * FROM lists 
    "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(lists)
}

pub(crate) async fn insert_list(pool: &PgPool, list: List) -> ListInsertResult {
    let res = query!(
        r#"
        INSERT INTO lists (id, title, pinned) VALUES ($1, $2, $3)
    "#,
        list.id,
        list.title,
        list.pinned
    )
    .execute(pool)
    .await;
    match res {
        Ok(_) => ListInsertResult::Inserted,
        Err(err) => {
            #[allow(clippy::unwrap_used)]
            let Some(database_err) = err.into_database_error() else {
                return ListInsertResult::InternalError;
            };
            if !database_err.is_unique_violation() {
                return ListInsertResult::InternalError;
            }
            ListInsertResult::Duplicate
        }
    }
}

pub(crate) async fn delete_list(pool: &PgPool, id: Uuid) -> DeleteResult {
    let res = query!(
        r#"
        DELETE FROM lists WHERE id=$1
    "#,
        id
    )
    .execute(pool)
    .await;
    let Ok(res) = res else {
        return DeleteResult::InternalError;
    };

    if res.rows_affected() == 0 {
        // If the entry did not exist, the rows affected will be 0
        DeleteResult::NotFound
    } else {
        DeleteResult::Deleted
    }
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub(crate) struct PatchListParams {
    title: Option<String>,
    pinned: Option<bool>,
}
pub(crate) async fn update_list(
    pool: &PgPool,
    id: Uuid,
    params: PatchListParams,
) -> ListUpdateResult {
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
        id,
    )
    .execute(pool)
    .await;
    match res {
        Ok(result) if result.rows_affected() > 0 => ListUpdateResult::Updated,
        Ok(_) => ListUpdateResult::NotFound,
        Err(_) => ListUpdateResult::InternalError,
    }
}

#[utoipa::path(
    get,
    path = "/api/item/{id}",
    tag = "Items",
    params(
        ("id" = Uuid, Path, description = "UUID of the item")
    ),
    responses(
        (status = 200, description = "Item found", body = Item),
        (status = 404, description = "Item not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub(crate) async fn get_items(pool: &PgPool, list_id: Uuid) -> anyhow::Result<Vec<Item>> {
    let items = query_as!(
        Item,
        r#"
        SELECT * FROM items WHERE list_id=$1
    "#,
        list_id
    )
    .fetch_all(pool)
    .await?;
    Ok(items)
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum GetItemError {
    #[error("Item not found")]
    NotFound,
    #[error("Sqlx error: $1")]
    Sqlx(#[from] sqlx::Error),
}

pub(crate) async fn get_item(pool: &PgPool, id: Uuid) -> Result<Item, GetItemError> {
    let item = query_as!(
        Item,
        r#"
        SELECT * FROM items WHERE id=$1
    "#,
        id
    )
    .fetch_one(pool)
    .await;
    match item {
        Ok(item) => Ok(item),
        Err(sqlx::Error::RowNotFound) => Err(GetItemError::NotFound),
        Err(err) => Err(GetItemError::Sqlx(err)),
    }
}

pub(crate) enum ItemInsertResult {
    Inserted,
    Duplicate,
    ListNotFound,
    InternalError,
}

pub(crate) async fn insert_item(pool: &PgPool, item: Item) -> ItemInsertResult {
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
    .execute(pool)
    .await;
    match res {
        Ok(_) => ItemInsertResult::Inserted,
        Err(err) => {
            #[allow(clippy::unwrap_used)]
            let Some(database_err) = err.into_database_error() else {
                return ItemInsertResult::InternalError;
            };
            if database_err.is_unique_violation() {
                return ItemInsertResult::Duplicate;
            }
            if database_err.is_foreign_key_violation() {
                return ItemInsertResult::ListNotFound;
            }
            ItemInsertResult::InternalError
        }
    }
}

pub(crate) async fn delete_item(pool: &PgPool, id: Uuid) -> DeleteResult {
    let res = query!(
        r#"
        DELETE FROM items WHERE id=$1
    "#,
        id
    )
    .execute(pool)
    .await;
    let Ok(res) = res else {
        return DeleteResult::InternalError;
    };

    if res.rows_affected() == 0 {
        // If the entry did not exist, the rows affected will be 0
        DeleteResult::NotFound
    } else {
        DeleteResult::Deleted
    }
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in=Query)]
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

pub(crate) async fn update_item(
    pool: &PgPool,
    id: Uuid,
    params: PatchItemParams,
) -> ItemUpdateResult {
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
    .execute(pool)
    .await;
    match res {
        Ok(result) if result.rows_affected() > 0 => ItemUpdateResult::NotFound,
        Ok(_) => ItemUpdateResult::Updated,
        Err(err) => match err.into_database_error() {
            Some(database_err) if database_err.is_foreign_key_violation() => {
                ItemUpdateResult::ListNotFound
            }
            _ => ItemUpdateResult::InternalError,
        },
    }
}
