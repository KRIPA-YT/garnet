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
    db::{DeleteResult, ListInsertResult, ListUpdateResult, PatchListParams},
    model::{Item, List},
};

#[utoipa::path(

    get,
    path = "/api/lists",
    tag = "Lists",
    responses(
        (
            status = 200,
            description = "Returns all lists",
            body = [List]
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub(crate) async fn get_lists(State(state): State<AppState>) -> impl IntoResponse {
    let lists = crate::db::get_lists(&state.pool).await;
    lists.map_or_else(
        |_| StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        |lists| Json(lists).into_response(),
    )
}

#[utoipa::path(
    get,
    path = "/api/list/{id}",
    tag = "Lists",
    params(
        ("id" = Uuid, Path, description = "List UUID")
    ),
    responses(
        (status = 200, description = "Items belonging to the list", body = [Item]),
        (status = 500, description = "Internal server error")
    )
)]

pub(crate) async fn get_list(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let items = crate::db::get_items(&state.pool, id).await;
    items.map_or_else(
        |_| StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        |items| Json(items).into_response(),
    )
}

#[derive(Deserialize, ToSchema)]
pub(crate) struct CreateListParams {
    title: String,
}

#[utoipa::path(
    put,
    path = "/api/list/{id}",
    tag = "Lists",
    params(
        ("id" = Uuid, Path, description = "UUID for the new list"),
    ),
    request_body = CreateListParams,
    responses(
        (status = 201, description = "List created"),
        (status = 204, description = "List ID already exists"),
        (status = 500, description = "Internal server error")
    )
)]

pub(crate) async fn create_list(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(list): Json<CreateListParams>,
) -> impl IntoResponse {
    let list = List {
        id,
        title: list.title,
        pinned: false,
    };
    let res = crate::db::insert_list(&state.pool, list).await;
    match res {
        ListInsertResult::Inserted => StatusCode::CREATED,
        ListInsertResult::Duplicate => StatusCode::NO_CONTENT,
        ListInsertResult::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[utoipa::path(
    delete,
    path = "/api/list/{id}",
    tag = "Lists",
    params(
        ("id" = Uuid, Path, description = "UUID of the list to delete")
    ),
    responses(
        (status = 204, description = "List deleted"),
        (status = 404, description = "List not found"),
        (status = 500, description = "Internal server error")
    )
)]

pub(crate) async fn delete_list(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let res = crate::db::delete_list(&state.pool, id).await;
    match res {
        DeleteResult::Deleted => StatusCode::NO_CONTENT,
        DeleteResult::NotFound => StatusCode::NOT_FOUND,
        DeleteResult::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[utoipa::path(
    patch,
    path = "/api/list/{id}",
    tag = "Lists",
    params(
        ("id" = Uuid, Path, description = "UUID of the list to update"),
    ),
    request_body = PatchListParams,
    responses(
        (status = 204, description = "List updated"),
        (status = 404, description = "List not found"),
        (status = 500, description = "Internal server error")
    )
)]

pub(crate) async fn patch_list(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(list): Json<PatchListParams>,
) -> impl IntoResponse {
    let res = crate::db::update_list(&state.pool, id, list).await;
    match res {
        ListUpdateResult::Updated => StatusCode::NO_CONTENT,
        ListUpdateResult::NotFound => StatusCode::NOT_FOUND,
        ListUpdateResult::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
