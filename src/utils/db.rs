use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;

use crate::models::queries;

pub async fn search_items_by_localized_name(
    pool: &PgPool,
    lang: String,
    item: String,
) -> Result<Vec<queries::LocalizedName>, sqlx::Error> {
    sqlx::query_as!(
        queries::LocalizedName,
        "
SELECT
    item_unique_name,
    lang,
    name
FROM
    localized_name
WHERE
    lang = $1
ORDER BY
    SIMILARITY(name, $2) DESC
LIMIT 10",
        lang,
        item
    )
    .fetch_all(pool)
    .await
}

pub async fn get_item_data_by_item_group_name(
    pool: &PgPool,
    item_group_name: &String,
) -> Result<Value, sqlx::Error> {
    sqlx::query!(
        "
SELECT
    data
FROM
    item_data
WHERE
    item_group_name = $1",
        item_group_name
    )
    .fetch_one(pool)
    .await
    .map(|x| x.data)
}

pub async fn query_market_orders(
    pool: &PgPool,
    unique_name: String,
    location_id: Option<i16>,
    auction_type: Option<String>,
    quality_level: Option<i16>,
    enchantment_level: Option<i16>,
    tier: Option<i16>,
    limit: i64,
    offset: i64,
) -> Result<Vec<queries::MarketOrder>, sqlx::Error> {
    sqlx::query_as!(
        queries::MarketOrder,
        "
SELECT
    market_order.id,
    location_id,
    market_order.item_unique_name,
    (item_data.data->>'@tier')::SMALLINT as tier,
    enchantment_level,
    quality_level,
    unit_price_silver,
    amount,
    auction_type,
    expires_at,
    updated_at
FROM
    market_order
    JOIN item ON market_order.item_unique_name = item.unique_name
    JOIN item_data ON item_data.item_group_name = item.item_group_name
WHERE
    expires_at > NOW()
    AND market_order.item_unique_name = $1
    AND ( $2::SMALLINT IS NULL OR location_id = $2 )
    AND ( $3::TEXT IS NULL OR auction_type = $3 )
    AND ( $4::SMALLINT IS NULL OR quality_level = $4 )
    AND ( $5::SMALLINT IS NULL OR (item_data.data->>'@tier')::SMALLINT = $5 )
    AND ( $6::SMALLINT IS NULL OR item.enchantment_level::SMALLINT = $6 )
ORDER BY unit_price_silver ASC
OFFSET $7
LIMIT $8",
        unique_name,
        location_id,
        auction_type,
        quality_level,
        tier,
        enchantment_level,
        offset,
        limit,
    )
    .fetch_all(pool)
    .await
}

pub async fn query_market_orders_with_localized_name(
    pool: &PgPool,
    localized_name: Option<String>,
    lang: String,
    location_id: Option<i16>,
    auction_type: Option<String>,
    quality_level: Option<i16>,
    enchantment_level: Option<i16>,
    tier: Option<i16>,
    limit: i64,
    offset: i64,
) -> Result<Vec<queries::MarketOrder>, sqlx::Error> {
    sqlx::query_as!(
        queries::MarketOrder,
        "
SELECT
    market_order.id,
    location.id as location_id,
    market_order.item_unique_name,
    (item_data.data->>'@tier')::SMALLINT as tier,
    enchantment_level,
    quality_level,
    unit_price_silver,
    amount,
    auction_type,
    expires_at,
    updated_at
FROM
    market_order
    JOIN item ON market_order.item_unique_name = item.unique_name
    JOIN item_data ON item_data.item_group_name = item.item_group_name
    JOIN location ON location_id = location.id
    JOIN localized_name ON market_order.item_unique_name = localized_name.item_unique_name
WHERE
    expires_at > NOW()
    AND lang = $1
    AND ( $3::SMALLINT IS NULL OR location.id = $3 )
    AND ( $4::TEXT IS NULL OR auction_type = $4 )
    AND ( $5::SMALLINT IS NULL OR quality_level = $5 )
    AND ( $6::SMALLINT IS NULL OR (item_data.data->>'@tier')::SMALLINT = $6 )
    AND ( $7::SMALLINT IS NULL OR enchantment_level = $7 )
ORDER BY
    SIMILARITY(localized_name.name, $2) DESC,
    unit_price_silver ASC
OFFSET $8
LIMIT $9",
        lang,
        localized_name,
        location_id,
        auction_type,
        quality_level,
        tier,
        enchantment_level,
        offset,
        limit,
    )
    .fetch_all(pool)
    .await
}

pub async fn get_localized_names_by_unique_name(
    pool: &PgPool,
    unique_name: &String,
) -> Result<Vec<queries::LocalizedName>, sqlx::Error> {
    sqlx::query_as!(
        queries::LocalizedName,
        "
SELECT
    item_unique_name,
    lang,
    name
FROM localized_name
    WHERE item_unique_name = $1",
        unique_name
    )
    .fetch_all(pool)
    .await
}

pub async fn get_localized_descriptions_by_unique_name(
    pool: &PgPool,
    unique_name: &String,
) -> Result<Vec<queries::LocalizedDescription>, sqlx::Error> {
    sqlx::query_as!(
        queries::LocalizedDescription,
        "
SELECT
    item_unique_name,
    lang,
    description
FROM
    localized_description
WHERE
    item_unique_name = $1",
        unique_name
    )
    .fetch_all(pool)
    .await
}

pub async fn get_market_orders_count(
    auction_type: Option<String>,
    pool: &PgPool,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query!(
        "
SELECT
	COUNT(*) as count
FROM
	market_order
WHERE
	($1::TEXT IS NULL OR auction_type = $1)",
        auction_type
    )
    .fetch_one(pool)
    .await
    .map(|x| x.count)
}

pub async fn get_market_histories_count(pool: &PgPool) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query!(
        "
SELECT APPROXIMATE_ROW_COUNT('market_history') as count"
    )
    .fetch_one(pool)
    .await
    .map(|x| x.count)
}

pub async fn get_market_orders_count_by_date(
    pool: &PgPool,
    interval: &str,
) -> Result<Vec<queries::MarketOrderCountByUpdatedAt>, sqlx::Error> {
    sqlx::query_as!(
        queries::MarketOrderCountByUpdatedAt,
        "
SELECT
    time_bucket($1::TEXT::INTERVAL, date) as date,
    SUM(count)::BIGINT as count
FROM
    market_orders_count_by_hour
GROUP BY
    time_bucket($1::TEXT::INTERVAL, date)
ORDER BY
    date DESC",
        interval
    )
    .fetch_all(pool)
    .await
}

pub async fn get_market_orders_count_by_date_and_location(
    pool: &PgPool,
    interval: &str,
) -> Result<Vec<queries::MarketOrderCountByUpdatedAtAndLocation>, sqlx::Error> {
    sqlx::query_as!(
        queries::MarketOrderCountByUpdatedAtAndLocation,
        "
SELECT
    time_bucket($1::TEXT::INTERVAL, date) as date,
    location_data.name as location,
    SUM(count)::BIGINT as count
FROM
    market_orders_count_by_hour_and_location
    JOIN location ON location.id = market_orders_count_by_hour_and_location.location_id
    JOIN location_data ON location_data.location_id = location.id
GROUP BY
    time_bucket($1::TEXT::INTERVAL, date),
    location_data.name
ORDER BY
    date DESC",
        interval
    )
    .fetch_all(pool)
    .await
}

pub async fn query_locations(
    pool: &PgPool,
    min_market_orders: Option<i32>,
) -> Result<Vec<queries::Location>, sqlx::Error> {
    sqlx::query_as!(
        queries::Location,
        "
SELECT
    location.id,
    location_data.name
FROM
    location
    JOIN location_data ON location_data.location_id = location.id
    LEFT JOIN market_order ON market_order.location_id = location.id
GROUP BY
    location.id, location_data.name
HAVING
    ($1::INT IS NULL OR COUNT(market_order.location_id) >= $1)
ORDER BY
    COUNT(market_order.location_id) DESC",
        min_market_orders
    )
    .fetch_all(pool)
    .await
}

pub async fn get_locations_by_id(
    pool: &PgPool,
    location_id: &i16,
) -> Result<queries::Location, sqlx::Error> {
    sqlx::query_as!(
        queries::Location,
        "
SELECT
    location.id,
    location_data.name
FROM
    location
    JOIN location_data ON location_data.location_id = location.id
WHERE
    location.id = $1",
        location_id
    )
    .fetch_one(pool)
    .await
}

pub async fn get_item_stats_by_date(
    pool: &sqlx::Pool<sqlx::Postgres>,
    unique_name: &str,
    interval: &str,
) -> Result<Vec<queries::ItemStatsByDate>, sqlx::Error> {
    sqlx::query_as!(
        queries::ItemStatsByDate,
        "
SELECT
    time_bucket($2::TEXT::INTERVAL, date) as date,
    item_unique_name,
    SUM(total_count)::BIGINT as total_count,
    MAX(max_unit_price_silver_offer) as max_unit_price_silver_offer,
    MIN(min_unit_price_silver_offer) as min_unit_price_silver_offer,
    AVG(avg_unit_price_silver_offer)::BIGINT as avg_unit_price_silver_offer,
    SUM(sum_amount_offer)::BIGINT as sum_amount_offer,
    MAX(max_unit_price_silver_request) as max_unit_price_silver_request,
    MIN(min_unit_price_silver_request) as min_unit_price_silver_request,
    AVG(avg_unit_price_silver_request)::BIGINT as avg_unit_price_silver_request,
    SUM(sum_amount_request)::BIGINT as sum_amount_request
FROM
    item_prices_by_hour
WHERE
    item_unique_name = $1
GROUP BY
    time_bucket($2::TEXT::INTERVAL, date),
    item_unique_name
ORDER BY
    date DESC",
        unique_name,
        interval
    )
    .fetch_all(pool)
    .await
}

pub async fn get_item_stats_by_date_and_location(
    pool: &sqlx::Pool<sqlx::Postgres>,
    unique_name: &str,
    interval: &str,
) -> Result<Vec<queries::ItemStatsByDateAndLocation>, sqlx::Error> {
    sqlx::query_as!(
        queries::ItemStatsByDateAndLocation,
        "
SELECT
    time_bucket($2::TEXT::INTERVAL, date) as date,
    item_unique_name,
    location_id,
    SUM(total_count)::BIGINT as total_count,
    MAX(max_unit_price_silver_offer) as max_unit_price_silver_offer,
    MIN(min_unit_price_silver_offer) as min_unit_price_silver_offer,
    AVG(avg_unit_price_silver_offer)::BIGINT as avg_unit_price_silver_offer,
    SUM(sum_amount_offer)::BIGINT as sum_amount_offer,
    MAX(max_unit_price_silver_request) as max_unit_price_silver_request,
    MIN(min_unit_price_silver_request) as min_unit_price_silver_request,
    AVG(avg_unit_price_silver_request)::BIGINT as avg_unit_price_silver_request,
    SUM(sum_amount_request)::BIGINT as sum_amount_request
FROM
    item_prices_by_hour_and_location
WHERE
    item_unique_name = $1
GROUP BY
    time_bucket($2::TEXT::INTERVAL, date),
    item_unique_name,
    location_id
ORDER BY
    date DESC
    ",
        unique_name,
        interval
    )
    .fetch_all(pool)
    .await
}

pub async fn get_item_market_history(
    pool: &sqlx::Pool<sqlx::Postgres>,
    unique_name: String,
    location_id: Option<i16>,
    quality_level: Option<i16>,
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
) -> Result<Vec<queries::ItemMarketHistory>, sqlx::Error> {
    sqlx::query_as!(
        queries::ItemMarketHistory,
        "
SELECT DISTINCT ON (timestamp, quality_level)
	time_bucket('1 day', timestamp) as timestamp,
    item_unique_name,
    location_id,
    quality_level,
    SUM(item_amount)::BIGINT as item_amount,
    SUM(silver_amount)::BIGINT as silver_amount,
    updated_at
FROM
    market_history
WHERE
    item_unique_name = $1
    AND timescale != 0
    AND ( $2::SMALLINT IS NULL OR location_id = $2 )
    AND ( $3::SMALLINT IS NULL OR quality_level = $3 )
    AND ( $4::TIMESTAMPTZ IS NULL OR timestamp > $4 )
    AND ( $5::TIMESTAMPTZ IS NULL OR timestamp < $5 )
GROUP BY
	time_bucket('1 day', timestamp),
	timescale,
	item_unique_name,
    location_id,
    quality_level,
    updated_at
ORDER BY
    timestamp DESC,
    quality_level DESC,
    updated_at DESC
    ",
        unique_name,
        location_id,
        quality_level,
        from,
        to
    )
    .fetch_all(pool)
    .await
}
