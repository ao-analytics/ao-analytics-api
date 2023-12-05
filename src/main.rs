#[macro_use]
extern crate dotenv_codegen;

use std::collections::HashMap;

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

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

    let app = Router::new()
        .route("/items/:id", get(search_items))
        .route("/items/:id/localizations", get(get_item_localizations))
        .route("/items/:id/orders", get(get_item_market_orders))
        .route("/statistics", get(get_statistics))
        .route("/orders", get(get_market_orders))
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

async fn get_statistics(State(pool): State<Pool<Postgres>>) -> Response<Body> {
    let market_order_count = utils::db::get_market_orders_count(&pool).await;
    let market_order_count_by_item = utils::db::get_market_orders_count_by_item(&pool).await.unwrap();
    let market_order_count_by_location = utils::db::get_market_orders_count_by_location(&pool).await.unwrap();
    let market_order_count_by_auction_type = utils::db::get_market_orders_count_by_auction_type(&pool).await.unwrap();
    let market_order_count_by_quality_level = utils::db::get_market_orders_count_by_quality_level(&pool).await.unwrap();
    let market_order_count_by_enchantment_level = utils::db::get_market_orders_count_by_enchantment_level(&pool).await.unwrap();
    let market_order_count_by_created_at = utils::db::get_market_orders_count_by_created_at(&pool).await.unwrap();

    let statistics = db::Statistics {
        market_order_count,
        market_order_count_by_item,
        market_order_count_by_location,
        market_order_count_by_auction_type,
        market_order_count_by_quality_level,
        market_order_count_by_enchantment_level,
        market_order_count_by_created_at,
    };

    Json(statistics).into_response()
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