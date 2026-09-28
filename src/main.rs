pub mod db;
pub mod model;
pub mod openapi;
pub mod routes;
pub mod user;

use std::sync::Arc;

use axum::{Router, routing::get};
use utoipa::OpenApi as _;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    openapi::ApiDoc,
    routes::{
        item::{create_item, delete_item, get_item, patch_item},
        list::{create_list, delete_list, get_list, get_lists, patch_list},
    },
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
