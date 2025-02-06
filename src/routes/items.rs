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

use crate::models::queries::Localizations;
use crate::utils;

pub fn get_router() -> Router<Pool<Postgres>> {
    Router::new()
        .route("/", get(search_items))
        .route("/:id/localizations", get(get_item_localizations))
        .route("/:id/orders", get(get_item_market_orders))
        .route("/:id/history", get(get_item_market_history))
        .route("/:id/data", get(get_item_data))
}

#[derive(Deserialize)]
struct ItemsQuery {
    name: String,
    lang: Option<String>,
}

async fn search_items(
    Query(query): Query<ItemsQuery>,
    State(pool): State<Pool<Postgres>>,
) -> Response<Body> {
    let lang = query.lang.unwrap_or("EN-US".to_string());

    let result = utils::db::search_items_by_localized_name(&pool, lang, query.name).await;

    match result {
        Ok(item) => Json(item).into_response(),
        Err(error) => {
            warn!("{:?}", error);
            StatusCode::NOT_FOUND.into_response()
        }
    }
}

async fn get_item_data(
    Path(item_group_name): Path<String>,
    State(pool): State<Pool<Postgres>>,
) -> Response<Body> {
    let result = utils::db::get_item_data_by_item_group_name(&pool, &item_group_name).await;

    match result {
        Ok(item) => Json(item).into_response(),
        Err(e) => {
            warn!("{:?}", e);
            StatusCode::NOT_FOUND.into_response()
        }
    }
}

async fn get_item_localizations(
    Path(unique_name): Path<String>,
    State(pool): State<Pool<Postgres>>,
) -> Response<Body> {
    let result = utils::db::get_localized_names_by_unique_name(&pool, &unique_name)
        .await
        .map(|names| {
            names
                .iter()
                .map(|name| (name.lang.clone(), name.name.clone()))
                .collect()
        });

    let names = match result {
        Ok(item) => item,
        Err(e) => {
            warn!("{:?}", e);
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    let result = utils::db::get_localized_descriptions_by_unique_name(&pool, &unique_name)
        .await
        .map(|descriptions| {
            descriptions
                .iter()
                .map(|description| (description.lang.clone(), description.description.clone()))
                .collect()
        });

    let descriptions = match result {
        Ok(item) => item,
        Err(e) => {
            warn!("{:?}", e);
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    Json(Localizations {
        unique_name,
        names,
        descriptions,
    })
    .into_response()
}

#[derive(Deserialize)]
struct ItemMarketHistorQuery {
    timescale: i16,
    location_id: Option<i16>,
    quality_level: Option<i16>,
}

async fn get_item_market_history(
    Path(unique_name): Path<String>,
    Query(query): Query<ItemMarketHistorQuery>,
    State(pool): State<Pool<Postgres>>,
) -> Response<Body> {
    let result = utils::db::get_item_market_history(
        &pool,
        unique_name,
        query.timescale,
        query.location_id,
        query.quality_level,
    )
    .await;

    match result {
        Ok(history) => Json(history).into_response(),
        Err(e) => {
            warn!("{:?}", e);
            StatusCode::NOT_FOUND.into_response()
        }
    }
}

#[derive(Deserialize)]
struct ItemMarketOrderQuery {
    location_id: Option<i16>,
    auction_type: Option<String>,
    quality_level: Option<i16>,
    enchantment_level: Option<i16>,
    tier: Option<i16>,
    limit: Option<i64>,
    offset: Option<i64>,
}

async fn get_item_market_orders(
    Path(unique_name): Path<String>,
    Query(query): Query<ItemMarketOrderQuery>,
    State(pool): State<Pool<Postgres>>,
) -> Response<Body> {
    let limit = query
        .limit
        .map_or(100, |limit| if limit > 100 { 100 } else { limit });

    let offset = query.offset.unwrap_or(0);

    let result = utils::db::query_market_orders(
        &pool,
        unique_name,
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
