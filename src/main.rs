pub mod auth;
pub mod db;
pub mod error;
pub mod items;
pub mod lists;
pub mod model;
pub mod openapi;
pub mod routes;
pub mod sessions;
pub mod users;

use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
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

pub(crate) type AppState = Arc<InnerAppState>;

pub(crate) struct InnerAppState {
    pub auth: AuthService,
    pub items: ItemRepository,
    pub lists: ListRepository,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    println!("Connecting to database...");

    #[allow(clippy::expect_used)]
    let db_url =
        std::env::var("DATABASE_URL").expect("Need to set MIGRATOR_DATABASE_URL env variable");
    #[allow(clippy::expect_used)]
    let migrator_pool = db::establish_connection(&db_url)
        .await
        .expect("Could not connect to database");
    #[allow(clippy::expect_used)]
    sqlx::migrate!("./migrations")
        .run(&migrator_pool)
        .await
        .expect("Could not run migration!");
    drop(migrator_pool);

    #[allow(clippy::expect_used)]
    let db_url = std::env::var("APP_DATABASE_URL").expect("Need to set DATABASE_URL env variable");
    #[allow(clippy::expect_used)]
    let pool = db::establish_connection(&db_url)
        .await
        .expect("Could not connect to database");
    let auth = AuthService::new(pool.clone());
    let items = ItemRepository::new(pool.clone());
    let lists = ListRepository::new(pool.clone());
    let inner = InnerAppState { auth, items, lists };
    let app_state = Arc::new(inner);

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
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
        .route("/auth/refresh", post(refresh))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(app_state);

    // run our app with hyper, listening globally on port 3000
    #[allow(clippy::expect_used)]
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Couldn't bind to port 3000");
    println!("Listening on 0.0.0.0:3000....");
    let _ = axum::serve(listener, app).await;
}
