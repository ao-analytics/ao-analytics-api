#[macro_use]
extern crate dotenv_codegen;

use std::collections::HashMap;

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{StatusCode, Method},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use tower_http::cors::{Any, CorsLayer};

mod models;
use models::db;

mod utils;

#[tokio::main]
async fn main() {
    let db_url = match std::env::var("ENV") {
        Ok(env) => match env.as_str() {
            "PROD" => dotenv!("PROD_DATABASE_URL"),
            "DEV" => dotenv!("DATABASE_URL"),
            _ => dotenv!("DATABASE_URL"),
        },
        Err(_) => dotenv!("DATABASE_URL"),
    };

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(db_url)
        .await
        .unwrap();

    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any);

    let item_routes = Router::new()
        .route("/:id", get(search_items))
        .route("/:id/localizations", get(get_item_localizations))
        .route("/:id/orders", get(get_item_market_orders));

    let statistics_routes = Router::new()
        .route("/orders/item", get(get_item_statistics))
        .route("/orders/location", get(get_location_statistics))
        .route("/orders/auction_type", get(get_auction_type_statistics))
        .route("/orders/hourly", get(get_market_order_statistics))
        .route("/orders/count", get(get_market_order_count));

    let order_routes = Router::new()
        .route("/", get(get_market_orders));

    let routes = Router::new()
        .nest("/item", item_routes)
        .nest("/statistics", statistics_routes)
        .nest("/orders", order_routes);

    let app = Router::new()
        .nest("/api", routes)
        .layer(cors)
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", dotenv!("PORT")))
        .await
        .unwrap();

    println!("Server running on port {}", dotenv!("PORT"));

    axum::serve(listener, app).await.unwrap();
}

async fn search_items(
    Path(item): Path<String>,
    Query(query): Query<HashMap<String, String>>,
    State(pool): State<Pool<Postgres>>,
) -> Response<Body> {

    let lang = match query.get("lang") {
        Some(lang) => lang,
        None => "en_us",
    };

    let result = utils::db::search_items_by_localized_name(&pool, lang, &item).await;

    let result = match result {
        Ok(result) => result,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };

    Json(result).into_response()
}

async fn get_item_statistics(State(pool): State<Pool<Postgres>>) -> Response<Body> {
    let market_order_count_by_item = utils::db::get_market_orders_count_by_item(&pool).await.unwrap();

    Json(market_order_count_by_item).into_response()
}

async fn get_location_statistics(State(pool): State<Pool<Postgres>>) -> Response<Body> {
    let market_order_count_by_location = utils::db::get_market_orders_count_by_location(&pool).await.unwrap();

    Json(market_order_count_by_location).into_response()
}

async fn get_auction_type_statistics(State(pool): State<Pool<Postgres>>) -> Response<Body> {
    let market_order_count_by_auction_type = utils::db::get_market_orders_count_by_auction_type(&pool).await.unwrap();

    Json(market_order_count_by_auction_type).into_response()
}

async fn get_market_order_statistics(State(pool): State<Pool<Postgres>>) -> Response<Body> {
    let market_order_count_by_updated_at = utils::db::get_market_orders_count_by_updated_at(&pool).await.unwrap();

    Json(market_order_count_by_updated_at).into_response()
}

async fn get_market_order_count(State(pool): State<Pool<Postgres>>) -> Response<Body> {
    let market_order_count = utils::db::get_market_orders_count(&pool).await.unwrap();

    Json(market_order_count).into_response()
}

async fn get_item_market_orders(
    Path(unique_name): Path<String>,
    Query(query): Query<HashMap<String, String>>,
    State(pool): State<Pool<Postgres>>,
) -> Response<Body> {

    let location_id: Option<String> = match query.get("location_id") {
        Some(location_id) => Some(location_id.to_string()),
        None => None,
    };

    let auction_type: Option<String> = match query.get("auction_type")  {
        Some(auction_type) => Some(auction_type.to_string()),
        None => None,
    };

    let quality_level: Option<i32> = match query.get("quality_level") {
        Some(quality_level) => match quality_level.parse::<i32>() {
            Ok(quality_level) => Some(quality_level),
            Err(_) => return StatusCode::BAD_REQUEST.into_response(),
        },
        None => None,
    };

    let enchantment_level: Option<i32> = match query.get("enchantment_level") {
        Some(enchantment_level) => match enchantment_level.parse::<i32>() {
            Ok(enchantment_level) => Some(enchantment_level),
            Err(_) => return StatusCode::BAD_REQUEST.into_response(),
        },
        None => None,
    };

    let limit = match query.get("limit") {
        Some(limit) => match limit.parse::<i64>() {
            Ok(limit) => limit,
            Err(_) => return StatusCode::BAD_REQUEST.into_response(),
        },
        None => 100,
    };

    let offset = match query.get("offset") {
        Some(offset) => match offset.parse::<i64>() {
            Ok(offset) => offset,
            Err(_) => return StatusCode::BAD_REQUEST.into_response(),
        },
        None => 0,
    };

    let result = utils::db::query_market_orders(&pool, &unique_name, location_id, auction_type, quality_level, enchantment_level, limit, offset).await;

    let orders = match result {
        Ok(orders) => orders,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };

    Json(orders).into_response()
}

async fn get_item_localizations(
    Path(unique_name): Path<String>,
    State(pool): State<Pool<Postgres>>,
) -> Response<Body> {

    let result = utils::db::get_localized_names_by_unique_name(&pool, &unique_name).await;

    let name = match result {
        Ok(item) => item,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };

    let result = utils::db::get_localized_descriptions_by_unique_name(&pool, &unique_name).await;

    let description = match result {
        Ok(item) => item,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };

    let item = db::Localizations {
        unique_name: unique_name.to_string(),
        name,
        description,
    };

    Json(item).into_response()
}

async fn get_market_orders(State(pool): State<Pool<Postgres>>) -> Response<Body> {
    let result = utils::db::get_market_orders(&pool).await;

    let orders = match result {
        Ok(orders) => orders,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };

    Json(orders).into_response()
}