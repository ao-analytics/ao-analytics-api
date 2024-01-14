#[macro_use]
extern crate dotenv_codegen;

use std::collections::HashMap;

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, 
    Router,
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
        .allow_methods([Method::GET])
        .allow_origin(Any);

    let item_routes = Router::new()
        .route("/", get(search_items))
        .route("/:id/localizations", get(get_item_localizations))
        .route("/:id/orders", get(get_item_market_orders));

    let statistics_routes = Router::new()
        .route("/orders", get(get_market_order_statistics))
        .route("/orders/count", get(get_market_order_count));

    let order_routes = Router::new().route("/", get(get_market_orders));

    let location_routes = Router::new()
        .route("/", get(get_locations))
        .route("/:id", get(get_location_by_id));

    let routes = Router::new()
        .nest("/items", item_routes)
        .nest("/statistics", statistics_routes)
        .nest("/orders", order_routes)
        .nest("/locations", location_routes);

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
    Query(query): Query<HashMap<String, String>>,
    State(pool): State<Pool<Postgres>>,
) -> Response<Body> {
    let item = match query.get("name") {
        Some(item) => item,
        None => return StatusCode::BAD_REQUEST.into_response(),
    };

    let lang = match query.get("lang") {
        Some(lang) => lang,
        None => "en_us",
    };

    let result = utils::db::search_items_by_localized_name(&pool, lang, &item).await;

    let result = match result {
        Ok(result) => result,
        Err(error) => {
            print!("{:?}", error);
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    Json(result).into_response()
}

async fn get_market_order_statistics(
    Query(query): Query<HashMap<String, String>>,
    State(pool): State<Pool<Postgres>>,
) -> Response<Body> {
    let group_by = match query.get("group_by") {
        Some(group_by) => group_by,
        None => return StatusCode::BAD_REQUEST.into_response(),
    };

    if group_by == "updated_at" {
        let market_order_count_by_updated_at =
            utils::db::get_market_orders_count_by_updated_at(&pool)
                .await
                .unwrap();

        return Json(market_order_count_by_updated_at).into_response();
    }

    if group_by.contains("updated_at") && group_by.contains("location") {
        let market_order_count_by_updated_at_and_location =
            utils::db::get_market_orders_count_by_updated_at_and_location(&pool)
                .await
                .unwrap();

        return Json(market_order_count_by_updated_at_and_location).into_response();
    }

    if group_by == "created_at" {
        let market_order_count_by_created_at =
            utils::db::get_market_orders_count_by_created_at(&pool)
                .await
                .unwrap();

        return Json(market_order_count_by_created_at).into_response();
    }

    if group_by.contains("created_at") && group_by.contains("location") {
        let market_order_count_by_created_at_and_location =
            utils::db::get_market_orders_count_by_created_at_and_location(&pool)
                .await
                .unwrap();

        return Json(market_order_count_by_created_at_and_location).into_response();
    }

    if group_by == "location" {
        let market_order_count_by_location = utils::db::get_market_orders_count_by_location(&pool)
            .await
            .unwrap();

        return Json(market_order_count_by_location).into_response();
    }

    return StatusCode::BAD_REQUEST.into_response();
}

async fn get_market_order_count(
    Query(query): Query<HashMap<String, String>>,
    State(pool): State<Pool<Postgres>>,
) -> Response<Body> {
    let auction_type: Option<String> = match query.get("auction_type") {
        Some(auction_type) => Some(auction_type.to_string()),
        None => None,
    };

    let market_order_count = utils::db::get_market_orders_count(auction_type, &pool)
        .await
        .unwrap();

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

    let auction_type: Option<String> = match query.get("auction_type") {
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
            Ok(limit) => {
                if limit > 100 {
                    100
                } else {
                    limit
                }
            }
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

    let date_from: Option<chrono::NaiveDate> = match query.get("from") {
        Some(date_from) => match chrono::NaiveDate::parse_from_str(date_from, "%Y-%m-%d") {
            Ok(date_from) => Some(date_from),
            Err(_) => return StatusCode::BAD_REQUEST.into_response(),
        },
        None => None,
    };

    let date_to: Option<chrono::NaiveDate> = match query.get("to") {
        Some(date_to) => match chrono::NaiveDate::parse_from_str(date_to, "%Y-%m-%d") {
            Ok(date_to) => Some(date_to),
            Err(_) => return StatusCode::BAD_REQUEST.into_response(),
        },
        None => None,
    };

    let result = utils::db::query_market_orders(
        &pool,
        Some(unique_name),
        location_id,
        auction_type,
        quality_level,
        enchantment_level,
        date_from,
        date_to,
        limit,
        offset,
    )
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

async fn get_market_orders(
    Query(query): Query<HashMap<String, String>>,
    State(pool): State<Pool<Postgres>>,
) -> Response<Body> {
    let unique_name: Option<String> = match query.get("item") {
        Some(unique_name) => Some(unique_name.to_string()),
        None => None,
    };

    let location_id: Option<String> = match query.get("location_id") {
        Some(location_id) => Some(location_id.to_string()),
        None => None,
    };

    let auction_type: Option<String> = match query.get("auction_type") {
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
            Ok(limit) => {
                if limit > 100 {
                    100
                } else {
                    limit
                }
            }
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

    let date_from: Option<chrono::NaiveDate> = match query.get("from") {
        Some(date_from) => match date_from.parse::<chrono::NaiveDate>() {
            Ok(date_from) => Some(date_from),
            Err(_) => return StatusCode::BAD_REQUEST.into_response(),
        },
        None => None,
    };

    let date_to: Option<chrono::NaiveDate> = match query.get("to") {
        Some(date_to) => match date_to.parse::<chrono::NaiveDate>() {
            Ok(date_to) => Some(date_to),
            Err(_) => return StatusCode::BAD_REQUEST.into_response(),
        },
        None => None,
    };

    let result = utils::db::query_market_orders(
        &pool,
        unique_name,
        location_id,
        auction_type,
        quality_level,
        enchantment_level,
        date_from,
        date_to,
        limit,
        offset,
    )
    .await;

    let orders = match result {
        Ok(orders) => orders,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };

    Json(orders).into_response()
}

async fn get_locations(
    Query(query): Query<HashMap<String, String>>,
    State(pool): State<Pool<Postgres>>,
) -> Response<Body> {
    let min_market_orders: Option<i32> = match query.get("min_market_orders") {
        Some(min_market_orders) => match min_market_orders.parse::<i32>() {
            Ok(min_market_orders) => Some(min_market_orders),
            Err(_) => return StatusCode::BAD_REQUEST.into_response(),
        },
        None => None,
    };

    let result = utils::db::query_locations(&pool, min_market_orders).await;

    let locations = match result {
        Ok(locations) => locations,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };

    Json(locations).into_response()
}

async fn get_location_by_id(
    Path(id): Path<String>,
    State(pool): State<Pool<Postgres>>,
) -> Response<Body> {
    let result = utils::db::get_locations_by_id(&pool, &id).await;

    let locations = match result {
        Ok(locations) => locations,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };

    Json(locations).into_response()
}
