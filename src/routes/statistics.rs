use serde::Deserialize;
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
        .route("/histories/count", get(get_market_history_count))
        .route("/items/:id", get(get_item_stats))
}

#[derive(Deserialize)]
struct MarketOrderStatsQuery {
    group_by: String,
}

async fn get_market_order_statistics(
    Query(query): Query<MarketOrderStatsQuery>,
    State(pool): State<Pool<Postgres>>,
) -> Response<Body> {
    let interval = match &query.group_by {
        group_by if group_by.contains("hour") => "1 hour",
        group_by if group_by.contains("day") => "1 day",
        group_by if group_by.contains("week") => "1 week",
        group_by if group_by.contains("month") => "1 month",
        _ => return StatusCode::BAD_REQUEST.into_response(),
    };

    let result = match query.group_by.contains("location") {
        true => utils::db::get_market_orders_count_by_date_and_location(&pool, interval)
            .await
            .map(|d| Json(d).into_response()),
        false => utils::db::get_market_orders_count_by_date(&pool, interval)
            .await
            .map(|d| Json(d).into_response()),
    };

    match result {
        Ok(stats) => stats,
        Err(e) => {
            warn!("{:?}", e);
            StatusCode::NOT_FOUND.into_response()
        }
    }
}

#[derive(Deserialize)]
struct MarketOrderCountQuery {
    aution_type: Option<String>,
}

async fn get_market_order_count(
    Query(query): Query<MarketOrderCountQuery>,
    State(pool): State<Pool<Postgres>>,
) -> Response<Body> {
    let result = utils::db::get_market_orders_count(query.aution_type, &pool).await;

    match result {
        Ok(count) => Json(count).into_response(),
        Err(e) => {
            warn!("{:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn get_market_history_count(State(pool): State<Pool<Postgres>>) -> Response<Body> {
    let result = utils::db::get_market_histories_count(&pool).await;

    match result {
        Ok(count) => Json(count).into_response(),
        Err(e) => {
            warn!("{:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[derive(Deserialize)]
struct ItemStatsQuery {
    group_by: String,
}

async fn get_item_stats(
    Path(id): Path<String>,
    Query(query): Query<ItemStatsQuery>,
    State(pool): State<Pool<Postgres>>,
) -> Response<Body> {
    let interval = match &query.group_by {
        group_by if group_by.contains("hour") => "1 hour",
        group_by if group_by.contains("day") => "1 day",
        group_by if group_by.contains("week") => "1 week",
        group_by if group_by.contains("month") => "1 month",
        _ => return StatusCode::BAD_REQUEST.into_response(),
    };

    let result = match query.group_by.contains("location") {
        true => utils::db::get_item_stats_by_date_and_location(&pool, &id, interval)
            .await
            .map(|d| Json(d).into_response()),
        false => utils::db::get_item_stats_by_date(&pool, &id, interval)
            .await
            .map(|d| Json(d).into_response()),
    };

    match result {
        Ok(stats) => stats,
        Err(e) => {
            warn!("{:?}", e);
            StatusCode::NOT_FOUND.into_response()
        }
    }
}
