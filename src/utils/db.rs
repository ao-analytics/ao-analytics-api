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

pub async fn get_item_data_by_unique_name(
    pool: &PgPool,
    unique_name: &String,
) -> Result<Value, sqlx::Error> {
    sqlx::query!(
        "
SELECT
    data
FROM
    item_data
WHERE
    item_unique_name = $1",
        unique_name
    )
    .fetch_one(pool)
    .await
    .map(|x| x.data)
}

pub async fn query_market_orders(
    pool: &PgPool,
    unique_name: String,
    location_id: Option<String>,
    auction_type: Option<String>,
    quality_level: Option<i32>,
    enchantment_level: Option<i32>,
    tier: Option<i32>,
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
    tier,
    enchantment_level,
    quality_level,
    unit_price_silver,
    amount,
    auction_type,
    expires_at,
    updated_at
FROM
    market_order
    JOIN item_data ON (market_order.item_unique_name = item_data.item_unique_name OR REGEXP_REPLACE(market_order.item_unique_name, '@.', '') = item_data.item_unique_name)
    JOIN location ON location_id = location.id
WHERE
    expires_at > NOW()
    AND market_order.item_unique_name = $1
    AND ( $2::TEXT IS NULL OR location.id = $2 )
    AND ( $3::TEXT IS NULL OR auction_type = $3 )
    AND ( $4::INT IS NULL OR quality_level = $4 )
    AND ( $5::INT IS NULL OR (item_data.data->>'@tier')::INT = $5 )
    AND ( $6::INT IS NULL OR COALESCE(NULLIF(item_data.data->>'@enchantmentlevel', ''), NULLIF(SPLIT_PART(market_order.item_unique_name, '@', 2), ''), '0')::INT = $6 )
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
    location_id: Option<String>,
    auction_type: Option<String>,
    quality_level: Option<i32>,
    enchantment_level: Option<i32>,
    tier: Option<i32>,
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
    tier,
    enchantment_level,
    quality_level,
    unit_price_silver,
    amount,
    auction_type,
    expires_at,
    updated_at
FROM
    market_order
    JOIN location ON location_id = location.id
    JOIN localized_name ON market_order.item_unique_name = localized_name.item_unique_name
WHERE
    expires_at > NOW()
    AND lang = $1
    AND ( $3::TEXT IS NULL OR location.id = $3 )
    AND ( $4::TEXT IS NULL OR auction_type = $4 )
    AND ( $5::INT IS NULL OR quality_level = $5 )
    AND ( $6::INT IS NULL OR tier = $6 )
    AND ( $7::INT IS NULL OR enchantment_level = $7 )
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
    location.name as location,
    SUM(count)::BIGINT as count
FROM
    market_orders_count_by_hour_and_location
    JOIN location ON location.id = market_orders_count_by_hour_and_location.location_id
GROUP BY
    time_bucket($1::TEXT::INTERVAL, date),
    location.name
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
    location.name
FROM
    location
LEFT JOIN (
    SELECT
        location_id,
        COUNT(*) as count
    FROM
        market_order
    GROUP BY
        location_id
) AS market_order_count
ON market_order_count.location_id = location.id
WHERE
    ( $1::INT IS NULL OR $1 <= COALESCE(market_order_count.count, 0) )
ORDER BY
    market_order_count.count DESC",
        min_market_orders
    )
    .fetch_all(pool)
    .await
}

pub async fn get_locations_by_id(
    pool: &PgPool,
    location_id: &String,
) -> Result<queries::Location, sqlx::Error> {
    sqlx::query_as!(
        queries::Location,
        "
SELECT
    location.id,
    location.name
FROM
    location
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
    AVG(avg_unit_price_silver_offer)::INTEGER as avg_unit_price_silver_offer,
    SUM(sum_amount_offer)::BIGINT as sum_amount_offer,
    MAX(max_unit_price_silver_request) as max_unit_price_silver_request,
    MIN(min_unit_price_silver_request) as min_unit_price_silver_request,
    AVG(avg_unit_price_silver_request)::INTEGER as avg_unit_price_silver_request,
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
    AVG(avg_unit_price_silver_offer)::INTEGER as avg_unit_price_silver_offer,
    SUM(sum_amount_offer)::BIGINT as sum_amount_offer,
    MAX(max_unit_price_silver_request) as max_unit_price_silver_request,
    MIN(min_unit_price_silver_request) as min_unit_price_silver_request,
    AVG(avg_unit_price_silver_request)::INTEGER as avg_unit_price_silver_request,
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
    timescale: i32,
    location_id: Option<String>,
    quality_level: Option<i32>,
) -> Result<Vec<queries::ItemMarketHistory>, sqlx::Error> {
    sqlx::query_as!(
        queries::ItemMarketHistory,
        "
SELECT
    item_unique_name,
    timestamp,
    location_id,
    quality_level,
    item_amount,
    silver_amount,
    updated_at
FROM
    market_history
WHERE
    item_unique_name = $1
    AND timescale = $2
    AND ( $3::TEXT IS NULL OR location_id = $3 )
    AND ( $4::INT IS NULL OR quality_level = $4 )
ORDER BY
    timestamp DESC
    ",
        unique_name,
        timescale,
        location_id,
        quality_level
    )
    .fetch_all(pool)
    .await
}
