use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    app::AppState, auth::extractor::AuthenticatedUser, items::model::Item,
    items::repository::PatchItemParams,
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
    state.items.get_item(&item_id, &user_id).await.map(Json)
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
    state.items.insert_item(item, &user_id).await
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
    state.items.delete_item(&item_id, &user_id).await
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
    state.items.update_item(&item_id, &user_id, item).await
}
