use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use sqlx::PgPool;
use utoipa::OpenApi as _;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    auth::service::AuthService,
    items::repository::ItemRepository,
    lists::repository::ListRepository,
    openapi::ApiDoc,
    routes::{
        auth::{login, logout, refresh, register},
        item::{create_item, delete_item, get_item, patch_item},
        list::{create_list, delete_list, get_list, get_lists, patch_list},
    },
};

pub type AppState = Arc<InnerAppState>;

pub struct InnerAppState {
    pub auth: AuthService,
    pub items: ItemRepository,
    pub lists: ListRepository,
}

impl InnerAppState {
    pub fn new(pool: PgPool) -> Self {
        Self {
            auth: AuthService::new(pool.clone()),
            items: ItemRepository::new(pool.clone()),
            lists: ListRepository::new(pool),
        }
    }
}

pub fn create_app(state: AppState) -> Router {
    Router::new()
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
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
        .route("/auth/refresh", post(refresh))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state)
}
