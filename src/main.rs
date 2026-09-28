pub mod db;
pub mod model;
pub mod openapi;
pub mod user;

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{OpenApi as _, ToSchema};
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

use crate::{
    db::{
        DeleteResult, ItemInsertResult, ItemUpdateResult, ListInsertResult, ListUpdateResult,
        PatchItemParams, PatchListParams,
    },
    model::{Item, List},
    openapi::ApiDoc,
};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    println!("Connecting to database...");
    #[allow(clippy::expect_used)]
    let db_url = std::env::var("DATABASE_URL").expect("Need to set DATABASE_URL env variable");
    #[allow(clippy::expect_used)]
    let pg_pool = db::establish_connection(&db_url)
        .await
        .expect("Could not connect to database");
    let pg_pool = Arc::new(pg_pool);

    #[allow(clippy::expect_used)]
    sqlx::migrate!("./migrations")
        .run(pg_pool.as_ref())
        .await
        .expect("Could not run migration!");

    println!("Connected!");
    println!("Starting webapp...");

    let app = Router::new()
        .route("/api/lists", get(get_lists))
        .route(
            "/api/list/{id}",
            get(get_list)
                .put(create_list)
                .delete(delete_list)
                .patch(patch_list),
        )
        .route(
            "/api/item/{id}",
            get(get_item)
                .put(create_item)
                .delete(delete_item)
                .patch(patch_item),
        )
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(pg_pool);

    // run our app with hyper, listening globally on port 3000
    #[allow(clippy::expect_used)]
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Couldn't bind to port 3000");
    println!("Listening on 0.0.0.0:3000....");
    let _ = axum::serve(listener, app).await;
}

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
async fn get_lists(State(pg_pool): State<Arc<PgPool>>) -> impl IntoResponse {
    let lists = db::get_lists(pg_pool.as_ref()).await;
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

async fn get_list(Path(id): Path<Uuid>, State(pg_pool): State<Arc<PgPool>>) -> impl IntoResponse {
    let items = db::get_items(pg_pool.as_ref(), id).await;
    items.map_or_else(
        |_| StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        |items| Json(items).into_response(),
    )
}

#[derive(Deserialize, ToSchema)]

struct CreateListParams {
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

async fn create_list(
    Path(id): Path<Uuid>,
    State(pg_pool): State<Arc<PgPool>>,
    Json(list): Json<CreateListParams>,
) -> impl IntoResponse {
    let list = List {
        id,
        title: list.title,
        pinned: false,
    };
    let res = db::insert_list(pg_pool.as_ref(), list).await;
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

async fn delete_list(
    Path(id): Path<Uuid>,
    State(pg_pool): State<Arc<PgPool>>,
) -> impl IntoResponse {
    let res = db::delete_list(pg_pool.as_ref(), id).await;
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

async fn patch_list(
    Path(id): Path<Uuid>,
    State(pg_pool): State<Arc<PgPool>>,
    Json(list): Json<PatchListParams>,
) -> impl IntoResponse {
    let res = db::update_list(pg_pool.as_ref(), id, list).await;
    match res {
        ListUpdateResult::Updated => StatusCode::NO_CONTENT,
        ListUpdateResult::NotFound => StatusCode::NOT_FOUND,
        ListUpdateResult::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
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

async fn get_item(Path(id): Path<Uuid>, State(pg_pool): State<Arc<PgPool>>) -> impl IntoResponse {
    let item = db::get_item(pg_pool.as_ref(), id).await;
    item.map_or_else(
        |_| StatusCode::NOT_FOUND.into_response(),
        |item| (StatusCode::OK, Json(item)).into_response(),
    )
}

#[derive(Deserialize, ToSchema)]

struct CreateItemParams {
    list_id: Uuid,
    title: String,
}

#[utoipa::path(
    put,
    path = "/api/item/{id}",
    tag = "Items",
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

async fn create_item(
    Path(id): Path<Uuid>,
    State(pg_pool): State<Arc<PgPool>>,
    Json(item): Json<CreateItemParams>,
) -> impl IntoResponse {
    let item = Item {
        id,
        list_id: item.list_id,
        title: item.title,
        pinned: false,
        checked: false,
    };
    let res = db::insert_item(pg_pool.as_ref(), item).await;
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
    params(
        ("id" = Uuid, Path, description = "UUID of the item to delete")
    ),
    responses(
        (status = 204, description = "Item deleted"),
        (status = 404, description = "Item not found"),
        (status = 500, description = "Internal server error")
    )
)]

async fn delete_item(
    Path(id): Path<Uuid>,
    State(pg_pool): State<Arc<PgPool>>,
) -> impl IntoResponse {
    let res = db::delete_item(pg_pool.as_ref(), id).await;
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

async fn patch_item(
    Path(id): Path<Uuid>,
    State(pg_pool): State<Arc<PgPool>>,
    Json(item): Json<PatchItemParams>,
) -> impl IntoResponse {
    let res = db::update_item(pg_pool.as_ref(), id, item).await;
    match res {
        ItemUpdateResult::Updated => StatusCode::NO_CONTENT,
        ItemUpdateResult::NotFound => StatusCode::NOT_FOUND,
        ItemUpdateResult::ListNotFound => StatusCode::BAD_REQUEST,
        ItemUpdateResult::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
