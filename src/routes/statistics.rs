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

async fn get_item_stats(
    Path(id): Path<String>,
    Query(query): Query<HashMap<String, String>>,
    State(pool): State<Pool<Postgres>>,
) -> Response<Body> {

    let group_by = match query.get("group_by") {
        Some(group_by) => group_by,
        None => return StatusCode::BAD_REQUEST.into_response(),
    };

    if group_by == "updated_at" {
        let result = utils::db::get_item_stats_by_updated_at(&pool, &id).await;

        return match result {
            Ok(item_stats) => Json(item_stats).into_response(),
            Err(e) => {
                warn!("{:?}", e);
                return StatusCode::NOT_FOUND.into_response();
            }
        }
    }

    if group_by.contains("updated_at") && group_by.contains("location") {
        let result = utils::db::get_item_stats_by_updated_at_and_location(&pool, &id).await;

        return match result {
            Ok(item_stats) => Json(item_stats).into_response(),
            Err(e) => {
                warn!("{:?}", e);
                return StatusCode::NOT_FOUND.into_response();
            }
        }
    }

    return StatusCode::BAD_REQUEST.into_response();
}
