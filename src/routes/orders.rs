use serde::Deserialize;
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

#[derive(Deserialize)]
struct MarketOrderQuery {
    item_name: Option<String>,
    lang: Option<String>,
    location_id: Option<String>,
    auction_type: Option<String>,
    quality_level: Option<i32>,
    enchantment_level: Option<i32>,
    tier: Option<i32>,
    limit: Option<i64>,
    offset: Option<i64>,
}

async fn get_market_orders(
    Query(query): Query<MarketOrderQuery>,
    State(pool): State<Pool<Postgres>>,
) -> Response<Body> {
    let lang = query.lang.unwrap_or("EN-US".to_string());

    let limit = query
        .limit
        .map_or(100, |limit| if limit > 100 { 100 } else { limit });

    let offset = query.offset.unwrap_or(0);

    let result = utils::db::query_market_orders_with_localized_name(
        &pool,
        query.item_name,
        lang,
        query.location_id,
        query.auction_type,
        query.quality_level,
        query.enchantment_level,
        query.tier,
        limit,
        offset,
    )
    .await;

    match result {
        Ok(orders) => Json(orders).into_response(),
        Err(e) => {
            warn!("{:?}", e);
            StatusCode::NOT_FOUND.into_response()
        }
    }
}
