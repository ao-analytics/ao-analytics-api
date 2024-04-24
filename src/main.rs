use std::time::Duration;

use axum::http::Method;
use axum::Router;

use sqlx::postgres::PgPoolOptions;

use tokio::net::TcpListener;

use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use tracing::info;

mod models;
mod routes;
mod utils;

#[tokio::main]
async fn main() {
    let config = utils::config::Config::from_env();

    let config = match config {
        Some(config) => config,
        None => {
            panic!("Failed to initialize config");
        }
    };

    tracing_subscriber::fmt().init();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&config.db_url)
        .await
        .unwrap();

    let cors = CorsLayer::new()
        .allow_methods([Method::GET])
        .allow_origin(Any);

    let routes = Router::new()
        .nest("/items", routes::items::get_router())
        .nest("/statistics", routes::statistics::get_router())
        .nest("/orders", routes::orders::get_router())
        .nest("/locations", routes::locations::get_router());

    let app = Router::new()
        .nest("/api", routes)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(pool);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", config.port))
        .await
        .unwrap();

    info!("Server running on port {}", config.port);

    axum::serve(listener, app).await.unwrap();
}
