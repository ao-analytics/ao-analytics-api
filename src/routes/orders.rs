use std::collections::HashMap;

use sqlx::{Pool, Postgres};

use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Json;
use axum::Router;

use tracing::warn;

use crate::utils;

pub fn get_router() -> Router<Pool<Postgres>> {
    Router::new().route("/", get(get_market_orders))
}

async fn get_market_orders(
    Query(query): Query<HashMap<String, String>>,
    State(pool): State<Pool<Postgres>>,
) -> Response<Body> {
    let localized_name = query.get("item_name").map(|localized_name| localized_name.to_string());

    let lang = match query.get("lang") {
        Some(lang) => lang.to_string(),
        None => "EN-US".to_string(),
    };

    let location_id: Option<String> = query.get("location_id").map(|location_id| location_id.to_string());

    let auction_type: Option<String> = query.get("auction_type").map(|auction_type| auction_type.to_string());

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

    let tier: Option<i32> = match query.get("tier") {
        Some(tier) => match tier.parse::<i32>() {
            Ok(tier) => Some(tier),
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

    let result = utils::db::query_market_orders_with_localized_name(
        &pool,
        localized_name,
        lang,
        location_id,
        auction_type,
        quality_level,
        enchantment_level,
        tier,
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
