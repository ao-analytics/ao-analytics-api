use std::collections::HashMap;

use sqlx::{Pool, Postgres};

use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Json;
use axum::Router;

use tracing::warn;

use crate::{models, utils};

pub fn get_router() -> Router<Pool<Postgres>> {
    Router::new()
        .route("/", get(search_items))
        .route("/:id/localizations", get(get_item_localizations))
        .route("/:id/orders", get(get_item_market_orders))
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
            warn!("{:?}", error);
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    Json(result).into_response()
}

async fn get_item_localizations(
    Path(unique_name): Path<String>,
    State(pool): State<Pool<Postgres>>,
) -> Response<Body> {
    let result = utils::db::get_localized_names_by_unique_name(&pool, &unique_name).await;

    let name = match result {
        Ok(item) => item,
        Err(e) => {
            warn!("{:?}", e);
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    let result = utils::db::get_localized_descriptions_by_unique_name(&pool, &unique_name).await;

    let description = match result {
        Ok(item) => item,
        Err(e) => {
            warn!("{:?}", e);
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    let item = models::db::Localizations {
        unique_name: unique_name.to_string(),
        name,
        description,
    };

    Json(item).into_response()
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
            Err(e) => {
                warn!("{:?}", e);
                return StatusCode::BAD_REQUEST.into_response();
            }
        },
        None => None,
    };

    let enchantment_level: Option<i32> = match query.get("enchantment_level") {
        Some(enchantment_level) => match enchantment_level.parse::<i32>() {
            Ok(enchantment_level) => Some(enchantment_level),
            Err(e) => {
                warn!("{:?}", e);
                return StatusCode::BAD_REQUEST.into_response();
            }
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
            Err(e) => {
                warn!("{:?}", e);
                return StatusCode::BAD_REQUEST.into_response();
            }
        },
        None => 100,
    };

    let offset = match query.get("offset") {
        Some(offset) => match offset.parse::<i64>() {
            Ok(offset) => offset,
            Err(e) => {
                warn!("{:?}", e);
                return StatusCode::BAD_REQUEST.into_response();
            }
        },
        None => 0,
    };

    let date_from: Option<chrono::NaiveDate> = match query.get("from") {
        Some(date_from) => match chrono::NaiveDate::parse_from_str(date_from, "%Y-%m-%d") {
            Ok(date_from) => Some(date_from),
            Err(e) => {
                warn!("{:?}", e);
                return StatusCode::BAD_REQUEST.into_response();
            }
        },
        None => None,
    };

    let date_to: Option<chrono::NaiveDate> = match query.get("to") {
        Some(date_to) => match chrono::NaiveDate::parse_from_str(date_to, "%Y-%m-%d") {
            Ok(date_to) => Some(date_to),
            Err(e) => {
                warn!("{:?}", e);
                return StatusCode::BAD_REQUEST.into_response();
            }
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
        Err(e) => {
            warn!("{:?}", e);
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    Json(orders).into_response()
}
