use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    AppState,
    auth::extractor::AuthenticatedUser,
    items::repository::{DeleteResult, ItemInsertResult, ItemUpdateResult, PatchItemParams},
    model::Item,
};

#[utoipa::path(
    get,
    path = "/api/item/{id}",
    tag = "Items",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = Uuid, Path, description = "UUID of the item")
    ),
    responses(
        (status = 200, description = "Item found", body = Item),
        (status = 404, description = "Item not found"),
        (status = 500, description = "Internal server error")
    )
)]

pub(crate) async fn get_item(
    Path(item_id): Path<Uuid>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let item = state.items.get_item(&item_id, &user_id).await;
    item.map_or_else(
        |_| StatusCode::NOT_FOUND.into_response(),
        |item| (StatusCode::OK, Json(item)).into_response(),
    )
}

#[derive(Deserialize, ToSchema)]

pub(crate) struct CreateItemParams {
    list_id: Uuid,
    title: String,
}

#[utoipa::path(
    put,
    path = "/api/item/{id}",
    tag = "Items",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = Uuid, Path, description = "UUID for the new item"),
    ),
    request_body = CreateItemParams,
    responses(
        (status = 201, description = "Item created"),
        (status = 204, description = "Item ID already exists"),
        (status = 400, description = "Parent list does not exist"),
        (status = 500, description = "Internal server error")
    )
)]

pub(crate) async fn create_item(
    Path(item_id): Path<Uuid>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(state): State<AppState>,
    Json(item): Json<CreateItemParams>,
) -> impl IntoResponse {
    let item = Item {
        id: item_id,
        list_id: item.list_id,
        title: item.title,
        pinned: false,
        checked: false,
    };
    let res = state.items.insert_item(item, &user_id).await;
    match res {
        ItemInsertResult::Inserted => StatusCode::CREATED,
        ItemInsertResult::Duplicate => StatusCode::NO_CONTENT,
        ItemInsertResult::ListNotFound => StatusCode::BAD_REQUEST,
        ItemInsertResult::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[utoipa::path(
    delete,
    path = "/api/item/{id}",
    tag = "Items",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = Uuid, Path, description = "UUID of the item to delete")
    ),
    responses(
        (status = 204, description = "Item deleted"),
        (status = 404, description = "Item not found"),
        (status = 500, description = "Internal server error")
    )
)]

pub(crate) async fn delete_item(
    Path(item_id): Path<Uuid>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let res = state.items.delete_item(&item_id, &user_id).await;
    match res {
        DeleteResult::Deleted => StatusCode::NO_CONTENT,
        DeleteResult::NotFound => StatusCode::NOT_FOUND,
        DeleteResult::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[utoipa::path(
    patch,
    path = "/api/item/{id}",
    tag = "Items",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = Uuid, Path, description = "UUID of the item to update"),
    ),
    request_body = PatchItemParams,
    responses(
        (status = 204, description = "Item updated"),
        (status = 400, description = "Parent list does not exist"),
        (status = 404, description = "Item not found"),
        (status = 500, description = "Internal server error")
    )
)]

pub(crate) async fn patch_item(
    Path(item_id): Path<Uuid>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(state): State<AppState>,
    Json(item): Json<PatchItemParams>,
) -> impl IntoResponse {
    let res = state.items.update_item(&item_id, &user_id, item).await;
    match res {
        ItemUpdateResult::Updated => StatusCode::NO_CONTENT,
        ItemUpdateResult::NotFound => StatusCode::NOT_FOUND,
        ItemUpdateResult::ListNotFound => StatusCode::BAD_REQUEST,
        ItemUpdateResult::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
