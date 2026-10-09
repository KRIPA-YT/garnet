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
    lists::repository::{DeleteResult, ListInsertResult, ListUpdateResult, PatchListParams},
    model::{Item, List},
};

#[utoipa::path(

    get,
    path = "/api/lists",
    tag = "Lists",
    security(
        ("bearer_auth" = [])
    ),
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
pub(crate) async fn get_lists(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let lists = state.lists.get_lists(&user_id).await;
    lists.map_or_else(
        |_| StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        |lists| Json(lists).into_response(),
    )
}

#[utoipa::path(
    get,
    path = "/api/list/{id}",
    tag = "Lists",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = Uuid, Path, description = "List UUID")
    ),
    responses(
        (status = 200, description = "Items belonging to the list", body = [Item]),
        (status = 500, description = "Internal server error")
    )
)]

pub(crate) async fn get_list(
    Path(list_id): Path<Uuid>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let items = state.items.get_items(&list_id, &user_id).await;
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
    security(
        ("bearer_auth" = [])
    ),
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
    Path(list_id): Path<Uuid>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(state): State<AppState>,
    Json(list): Json<CreateListParams>,
) -> impl IntoResponse {
    let list = List {
        id: list_id,
        title: list.title,
        pinned: false,
    };
    let res = state.lists.insert_list(list, &user_id).await;
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
    security(
        ("bearer_auth" = [])
    ),
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
    Path(list_id): Path<Uuid>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let res = state.lists.delete_list(&list_id, &user_id).await;
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
    security(
        ("bearer_auth" = [])
    ),
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
    Path(list_id): Path<Uuid>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(state): State<AppState>,
    Json(list): Json<PatchListParams>,
) -> impl IntoResponse {
    let res = state.lists.update_list(&list_id, &user_id, &list).await;
    match res {
        ListUpdateResult::Updated => StatusCode::NO_CONTENT,
        ListUpdateResult::NotFound => StatusCode::NOT_FOUND,
        ListUpdateResult::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
