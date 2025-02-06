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
        .route("/", get(get_locations))
        .route("/:id", get(get_location_by_id))
}

#[derive(Deserialize)]
struct LocationsQuery {
    min_market_orders: Option<i32>,
}

async fn get_locations(
    Query(query): Query<LocationsQuery>,
    State(pool): State<Pool<Postgres>>,
) -> Response<Body> {
    let result = utils::db::query_locations(&pool, query.min_market_orders).await;

    match result {
        Ok(locations) => Json(locations).into_response(),
        Err(e) => {
            warn!("{:?}", e);
            return StatusCode::NOT_FOUND.into_response();
        }
    }
}

async fn get_location_by_id(
    Path(id): Path<i16>,
    State(pool): State<Pool<Postgres>>,
) -> Response<Body> {
    let result = utils::db::get_locations_by_id(&pool, &id).await;

    match result {
        Ok(locations) => Json(locations).into_response(),
        Err(e) => {
            warn!("{:?}", e);
            return StatusCode::NOT_FOUND.into_response();
        }
    }
}
