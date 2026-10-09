mod app;
pub mod auth;
mod config;
pub mod db;
pub mod error;
pub mod items;
pub mod lists;
pub mod openapi;
pub mod routes;
pub mod sessions;
pub mod users;

use std::sync::Arc;

use crate::app::InnerAppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = config::Config::from_env()?;

    println!("Connecting to database...");

    let migrator_pool = db::establish_connection(&config.migration_database_url).await?;
    sqlx::migrate!("./migrations").run(&migrator_pool).await?;
    drop(migrator_pool);

    let pool = db::establish_connection(&config.app_database_url).await?;
    let app_state = Arc::new(InnerAppState::new(pool));

    println!("Connected!");
    println!("Starting webapp...");

    let app = app::create_app(app_state);

    let listener = tokio::net::TcpListener::bind(&config.listen_addr).await?;
    println!("Listening ...");
    let _ = axum::serve(listener, app).await;
    Ok(())
}
