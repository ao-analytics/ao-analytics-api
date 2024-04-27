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

use crate::utils;

pub fn get_router() -> Router<Pool<Postgres>> {
    Router::new()
        .route("/orders", get(get_market_order_statistics))
        .route("/orders/count", get(get_market_order_count))
        .route("/items/:id", get(get_item_stats))
}

async fn get_market_order_statistics(
    Query(query): Query<HashMap<String, String>>,
    State(pool): State<Pool<Postgres>>,
) -> Response<Body> {
    let group_by = match query.get("group_by") {
        Some(group_by) => group_by,
        None => return StatusCode::BAD_REQUEST.into_response(),
    };

    let interval = match group_by {
        group_by if group_by.contains("hour") => "1 hour",
        group_by if group_by.contains("day") => "1 day",
        group_by if group_by.contains("week") => "1 week",
        group_by if group_by.contains("month") => "1 month",
        _ => return StatusCode::BAD_REQUEST.into_response(),
    };

    if group_by.contains("location") {
        let market_order_count_by_location =
            utils::db::get_market_orders_count_by_date_and_location(&pool, interval)
                .await
                .unwrap();

        return Json(market_order_count_by_location).into_response();
    }

    let market_order_count_by_updated_at =
        utils::db::get_market_orders_count_by_date(&pool, interval)
            .await
            .unwrap();

    Json(market_order_count_by_updated_at).into_response()
}

async fn get_market_order_count(
    Query(query): Query<HashMap<String, String>>,
    State(pool): State<Pool<Postgres>>,
) -> Response<Body> {
    let auction_type: Option<String> = query.get("auction_type").map(|auction_type| auction_type.to_string());

    let market_order_count = utils::db::get_market_orders_count(auction_type, &pool)
        .await
        .unwrap();

    Json(market_order_count).into_response()
}

async fn get_item_stats(
    Path(id): Path<String>,
    Query(query): Query<HashMap<String, String>>,
    State(pool): State<Pool<Postgres>>,
) -> Response<Body> {
    let group_by = match query.get("group_by") {
        Some(group_by) => group_by.as_str(),
        None => return StatusCode::BAD_REQUEST.into_response(),
    };

    let interval = match group_by {
        group_by if group_by.contains("hour") => "1 hour",
        group_by if group_by.contains("day") => "1 day",
        group_by if group_by.contains("week") => "1 week",
        group_by if group_by.contains("month") => "1 month",
        _ => return StatusCode::BAD_REQUEST.into_response(),
    };

    if group_by.contains("location") {
        let result = utils::db::get_item_stats_by_date_and_location(&pool, &id, interval).await;

        return match result {
            Ok(item_stats) => Json(item_stats).into_response(),
            Err(e) => {
                warn!("{:?}", e);
                return StatusCode::NOT_FOUND.into_response();
            }
        };
    }

    let result = utils::db::get_item_stats_by_date(&pool, &id, interval).await;

    match result {
        Ok(item_stats) => Json(item_stats).into_response(),
        Err(e) => {
            warn!("{:?}", e);
            StatusCode::NOT_FOUND.into_response()
        }
    }
}
