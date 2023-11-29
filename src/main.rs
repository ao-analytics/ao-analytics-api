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
use sqlx::{postgres::PgPoolOptions, types::chrono, Pool, Postgres};

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
        .route("/items/:id/localizations", get(get_item_localizations))
        .route("/items/:id/orders", get(get_item_market_orders))
        .route("/statistics", get(get_statistics))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", dotenv!("PORT")))
        .await
        .unwrap();

    println!("Server running on port {}", dotenv!("PORT"));

    axum::serve(listener, app).await.unwrap();
}

async fn get_statistics(State(pool): State<Pool<Postgres>>) -> Response<Body> {
    let market_order_count = sqlx::query!("SELECT COUNT(*) FROM market_order")
        .fetch_one(&pool)
        .await
        .unwrap()
        .count;

    let market_order_count_by_item = sqlx::query_as!(MarketOrderCountByItem, "SELECT item_unique_name, COUNT(*) as count FROM market_order GROUP BY item_unique_name ORDER BY count DESC")
        .fetch_all(&pool)
        .await
        .unwrap();

    let market_order_count_by_location = sqlx::query_as!(MarketOrderCountByLocation, "SELECT location.name as location, COUNT(*) as count FROM market_order, location WHERE location_id = location.id GROUP BY location.name ORDER BY count DESC")
        .fetch_all(&pool)
        .await
        .unwrap();

    let market_order_count_by_auction_type = sqlx::query_as!(MarketOrderCountByAuctionType, "SELECT auction_type, COUNT(*) as count FROM market_order GROUP BY auction_type ORDER BY count DESC")
        .fetch_all(&pool)
        .await
        .unwrap();

    let market_order_count_by_quality_level = sqlx::query_as!(MarketOrderCountByQualityLevel, "SELECT quality_level, COUNT(*) as count FROM market_order GROUP BY quality_level ORDER BY count DESC")
        .fetch_all(&pool)
        .await
        .unwrap();

    let market_order_count_by_enchantment_level = sqlx::query_as!(MarketOrderCountByEnchantmentLevel, "SELECT enchantment_level, COUNT(*) as count FROM market_order GROUP BY enchantment_level ORDER BY count DESC")
        .fetch_all(&pool)
        .await
        .unwrap();

    let statistics = Statistics {
        market_order_count,
        market_order_count_by_item,
        market_order_count_by_location,
        market_order_count_by_auction_type,
        market_order_count_by_quality_level,
        market_order_count_by_enchantment_level,
    };

    Json(statistics).into_response()
}

async fn get_item_market_orders(
    Path(unique_name): Path<String>,
    Query(query): Query<HashMap<String, String>>,
    State(pool): State<Pool<Postgres>>,
) -> Response<Body> {
    let enchantment_level: Option<i32> = match query.get("enchantment_level") {
        Some(enchantment_level) => match enchantment_level.parse::<i32>() {
            Ok(enchantment_level) => Some(enchantment_level),
            Err(_) => return StatusCode::BAD_REQUEST.into_response(),
        },
        None => None,
    };

    let quality_level: Option<i32> = match query.get("quality_level") {
        Some(quality_level) => match quality_level.parse::<i32>() {
            Ok(quality_level) => Some(quality_level),
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

    let result = sqlx::query_as!(
        MarketOrder,
        "SELECT 
            location.name as location, 
            quality_level, 
            enchantment_level, 
            unit_price_silver, 
            amount, 
            auction_type, 
            expires_at, 
            updated_at 
        FROM market_order, location 
            WHERE location_id = location.id
            AND expires_at > NOW()
            AND item_unique_name = $1
            AND ( $2::TEXT IS NULL OR location.id = $2 )
            AND ( $3::TEXT IS NULL OR auction_type = $3 )
            AND ( $4::INT IS NULL OR quality_level = $4 )
            AND ( $5::INT IS NULL OR enchantment_level = $5 )
            ORDER BY unit_price_silver ASC
            LIMIT $6 OFFSET $7",
        unique_name,
        query.get("location"),
        query.get("auction_type"),
        quality_level,
        enchantment_level,
        limit,
        offset
    )
    .fetch_all(&pool)
    .await;

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
    let result = sqlx::query_as!(
        LocalizedName,
        "SELECT 
            item_unique_name, 
            en_us, 
            de_de, 
            fr_fr, 
            ru_ru, 
            pl_pl, 
            es_es, 
            pt_br, 
            it_it, 
            zh_cn, 
            ko_kr, 
            ja_jp, 
            zh_tw, 
            id_id 
        FROM localized_name 
            WHERE item_unique_name = $1",
        unique_name
    )
    .fetch_one(&pool)
    .await;

    let name = match result {
        Ok(item) => item,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };

    let result = sqlx::query_as!(
        LocalizedDescription,
        "SELECT 
            item_unique_name,
            en_us, 
            de_de,
            fr_fr, 
            ru_ru, 
            pl_pl, 
            es_es, 
            pt_br, 
            it_it, 
            zh_cn, 
            ko_kr, 
            ja_jp, 
            zh_tw, 
            id_id 
        FROM localized_description 
        WHERE item_unique_name = $1",
        unique_name
    )
    .fetch_one(&pool)
    .await;

    let description = match result {
        Ok(item) => item,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };

    let item = ItemResponse {
        unique_name,
        name,
        description,
    };

    Json(item).into_response()
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct Item {
    unique_name: String,
}

#[derive(sqlx::FromRow, serde::Serialize, Clone)]
struct LocalizedName {
    item_unique_name: String,
    en_us: String,
    de_de: String,
    fr_fr: String,
    ru_ru: String,
    pl_pl: String,
    es_es: String,
    pt_br: String,
    it_it: String,
    zh_cn: String,
    ko_kr: String,
    ja_jp: String,
    zh_tw: String,
    id_id: String,
}

#[derive(sqlx::FromRow, serde::Serialize, Clone)]
struct LocalizedDescription {
    item_unique_name: String,
    en_us: String,
    de_de: String,
    fr_fr: String,
    ru_ru: String,
    pl_pl: String,
    es_es: String,
    pt_br: String,
    it_it: String,
    zh_cn: String,
    ko_kr: String,
    ja_jp: String,
    zh_tw: String,
    id_id: String,
}

#[derive(serde::Serialize)]
struct ItemResponse {
    unique_name: String,
    name: LocalizedName,
    description: LocalizedDescription,
}

#[derive(serde::Serialize)]
struct LocalizationResponse {
    unique_name: String,
    name: Option<LocalizedName>,
    description: Option<LocalizedDescription>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct MarketOrder {
    location: String,
    quality_level: i32,
    enchantment_level: i32,
    unit_price_silver: i32,
    amount: i32,
    auction_type: String,
    expires_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
}

#[derive(serde::Serialize)]
struct Statistics {
    market_order_count: Option<i64>,
    market_order_count_by_item: Vec<MarketOrderCountByItem>,
    market_order_count_by_location: Vec<MarketOrderCountByLocation>,
    market_order_count_by_auction_type: Vec<MarketOrderCountByAuctionType>,
    market_order_count_by_quality_level: Vec<MarketOrderCountByQualityLevel>,
    market_order_count_by_enchantment_level: Vec<MarketOrderCountByEnchantmentLevel>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct MarketOrderCountByItem {
    item_unique_name: String,
    count: Option<i64>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct MarketOrderCountByLocation {
    location: String,
    count: Option<i64>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct MarketOrderCountByAuctionType {
    auction_type: String,
    count: Option<i64>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct MarketOrderCountByQualityLevel {
    quality_level: i32,
    count: Option<i64>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct MarketOrderCountByEnchantmentLevel {
    enchantment_level: i32,
    count: Option<i64>,
}
