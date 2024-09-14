use std::time::Duration;

use axum::http::Method;
use axum::Router;

use sqlx::postgres::PgPoolOptions;

use tokio::net::TcpListener;

use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod models;
mod routes;
mod utils;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tower_http=debug,axum::rejection=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .unwrap();

    let config = utils::config::Config::from_env();

    let config = match config {
        Some(config) => config,
        None => {
            panic!("Failed to initialize config");
        }
    };

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&config.database_url)
        .await
        .unwrap();

    let cors = CorsLayer::new()
        .allow_methods([Method::GET])
        .allow_origin(Any);

    let trace = TraceLayer::new_for_http();

    let routes = Router::new()
        .nest("/items", routes::items::get_router())
        .nest("/statistics", routes::statistics::get_router())
        .nest("/orders", routes::orders::get_router())
        .nest("/locations", routes::locations::get_router())
        .layer(cors)
        .layer(trace)
        .with_state(pool);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", config.port))
        .await
        .unwrap();

    info!("Server running on port {}", config.port);

    axum::serve(listener, routes).await.unwrap();
}
