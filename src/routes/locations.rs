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
        .route("/", get(get_locations))
        .route("/:id", get(get_location_by_id))
}

async fn get_locations(
    Query(query): Query<HashMap<String, String>>,
    State(pool): State<Pool<Postgres>>,
) -> Response<Body> {
    let min_market_orders: Option<i32> = match query.get("min_market_orders") {
        Some(min_market_orders) => match min_market_orders.parse::<i32>() {
            Ok(min_market_orders) => Some(min_market_orders),
            Err(e) => {
                warn!("{:?}", e);
                return StatusCode::BAD_REQUEST.into_response();
            }
        },
        None => None,
    };

    let result = utils::db::query_locations(&pool, min_market_orders).await;

    let locations = match result {
        Ok(locations) => locations,
        Err(e) => {
            warn!("{:?}", e);
            return StatusCode::NOT_FOUND.into_response();
        }
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
        Err(e) => {
            warn!("{:?}", e);
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    Json(locations).into_response()
}
