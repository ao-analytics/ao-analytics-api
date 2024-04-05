use sqlx::PgPool;

use crate::models::db;

pub async fn search_items_by_localized_name(
    pool: &PgPool,
    lang: &str,
    item: &str,
) -> Result<Vec<db::LocalizedName>, sqlx::Error> {
    return sqlx::query_as!(
        db::LocalizedName,
        "SELECT 
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
    .await;
}

pub async fn get_item_data_by_unique_name(
    pool: &PgPool,
    unique_name: &String,
) -> Result<db::ItemData, sqlx::Error> {
    return sqlx::query_as!(
        db::ItemData,
        "SELECT 
            unique_name,
            enchantment_level,
            tier,
            shop_sub_category.id as shop_sub_category,
            weight
        FROM
            item_data
            JOIN item ON item.unique_name = item_data.item_unique_name
            JOIN shop_sub_category ON shop_sub_category_id = shop_sub_category.id
            JOIN shop_category ON shop_sub_category.shop_category_id = shop_category.id
        WHERE 
            item_unique_name = $1",
        unique_name
    )
    .fetch_one(pool)
    .await;
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
) -> Result<Vec<db::MarketOrder>, sqlx::Error> {
    return sqlx::query_as!(
        db::MarketOrder,
        "SELECT
            market_order.id,
            location.id as location_id,
            market_order.item_unique_name,
            quality_level,
            unit_price_silver, 
            amount,
            auction_type, 
            expires_at, 
            updated_at
        FROM
            market_order
            JOIN location ON location_id = location.id
            JOIN item_data ON market_order.item_unique_name = item_data.item_unique_name
            JOIN localized_name ON market_order.item_unique_name = localized_name.item_unique_name
        WHERE
            expires_at > NOW()
            AND market_order.item_unique_name = $1
            AND ( $2::TEXT IS NULL OR location.id = $2 )
            AND ( $3::TEXT IS NULL OR auction_type = $3 )
            AND ( $4::INT IS NULL OR quality_level = $4 )
            AND ( $5::INT IS NULL OR tier = $4 )
            AND ( $6::INT IS NULL OR enchantment_level = $5 )
        ORDER BY unit_price_silver ASC
        OFFSET $7
        LIMIT $8",
        unique_name,
        location_id,
        auction_type,
        quality_level,
        enchantment_level,
        tier,
        offset,
        limit,
    )
    .fetch_all(pool)
    .await;
}

pub async fn query_market_orders_with_localized_name(
    pool: &PgPool,
    localized_name: String,
    lang: String,
    location_id: Option<String>,
    auction_type: Option<String>,
    quality_level: Option<i32>,
    enchantment_level: Option<i32>,
    tier: Option<i32>,
    limit: i64,
    offset: i64,
) -> Result<Vec<db::MarketOrder>, sqlx::Error> {
    return sqlx::query_as!(
        db::MarketOrder,
        "SELECT
            market_order.id,
            location.id as location_id,
            market_order.item_unique_name,
            quality_level,
            unit_price_silver, 
            amount, 
            auction_type, 
            expires_at, 
            updated_at
        FROM 
            market_order
            JOIN item_data ON market_order.item_unique_name = item_data.item_unique_name
            JOIN location ON location_id = location.id
            JOIN localized_name ON market_order.item_unique_name = localized_name.item_unique_name
        WHERE 
            expires_at > NOW()
            AND lang = $1
            AND ( $3::TEXT IS NULL OR location.id = $3 )
            AND ( $4::TEXT IS NULL OR auction_type = $4 )
            AND ( $5::INT IS NULL OR quality_level = $5 )
            AND ( $6::INT IS NULL OR enchantment_level = $6 )
            AND ( $7::INT IS NULL OR tier = $7 )
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
        enchantment_level,
        tier,
        offset,
        limit,
    )
    .fetch_all(pool)
    .await;
}

pub async fn get_localized_names_by_unique_name(
    pool: &PgPool,
    unique_name: &String,
) -> Result<Vec<db::LocalizedName>, sqlx::Error> {
    return sqlx::query_as!(
        db::LocalizedName,
        "SELECT 
            item_unique_name, 
            lang,
            name
        FROM localized_name 
            WHERE item_unique_name = $1",
        unique_name
    )
    .fetch_all(pool)
    .await;
}

pub async fn get_localized_descriptions_by_unique_name(
    pool: &PgPool,
    unique_name: &String,
) -> Result<Vec<db::LocalizedDescription>, sqlx::Error> {
    return sqlx::query_as!(
        db::LocalizedDescription,
        "SELECT 
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
    .await;
}

pub async fn get_market_orders_count(
    auction_type: Option<String>,
    pool: &PgPool,
) -> Result<db::MarketOrderCount, sqlx::Error> {
    match auction_type {
        Some(auction_type) => {
            return sqlx::query_as!(
                db::MarketOrderCount,
                "SELECT 
                    COUNT(*) as count
                FROM 
                    market_order
                WHERE
                    auction_type = $1",
                auction_type
            )
            .fetch_one(pool)
            .await;
        }
        None => {
            return sqlx::query_as!(
                db::MarketOrderCount,
                "SELECT 
                    COUNT(*) as count
                FROM 
                    market_order"
            )
            .fetch_one(pool)
            .await;
        }
    }
}

pub async fn get_market_orders_count_by_updated_at(
    pool: &PgPool,
) -> Result<Vec<db::MarketOrderCountByUpdatedAt>, sqlx::Error> {
    return sqlx::query_as!(
        db::MarketOrderCountByUpdatedAt,
        "SELECT 
        date, 
            count 
        FROM 
            market_orders_count_by_updated_at
        ORDER BY
            date DESC"
    )
    .fetch_all(pool)
    .await;
}

pub async fn get_market_orders_count_by_updated_at_and_location(
    pool: &PgPool,
) -> Result<Vec<db::MarketOrderCountByUpdatedAtAndLocation>, sqlx::Error> {
    return sqlx::query_as!(
        db::MarketOrderCountByUpdatedAtAndLocation,
        "SELECT 
            date,
            location.name as location,
            count
        FROM
            market_orders_count_by_updated_at_and_location
            JOIN location ON location.id = market_orders_count_by_updated_at_and_location.location_id
        ORDER BY
            date DESC"
    )
    .fetch_all(pool)
    .await;
}

pub async fn query_locations(
    pool: &PgPool,
    min_market_orders: Option<i32>,
) -> Result<Vec<db::Location>, sqlx::Error> {
    return sqlx::query_as!(
        db::Location,
        "SELECT 
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
    .await;
}

pub async fn get_locations_by_id(
    pool: &PgPool,
    location_id: &String,
) -> Result<db::Location, sqlx::Error> {
    return sqlx::query_as!(
        db::Location,
        "SELECT 
            location.id, 
            location.name
        FROM
            location
        WHERE 
            location.id = $1",
        location_id
    )
    .fetch_one(pool)
    .await;
}

pub async fn get_item_stats_by_updated_at_and_location(
    pool: &PgPool,
    unique_name: &String,
) -> Result<Vec<db::ItemStatsByDayAndLocation>, sqlx::Error> {
    return sqlx::query_as!(
        db::ItemStatsByDayAndLocation,
        "SELECT 
            date,
            item_unique_name,
            location_id,
            total_count,
            max_unit_price_silver_offer,
            min_unit_price_silver_offer,
            avg_unit_price_silver_offer,
            sum_amount_offer,
            max_unit_price_silver_request,
            min_unit_price_silver_request,
            avg_unit_price_silver_request,
            sum_amount_request
        FROM 
            item_prices_by_updated_at_and_location
        WHERE 
            item_unique_name = $1
        ORDER BY
            date DESC",
        unique_name
    )
    .fetch_all(pool)
    .await;
}

pub async fn get_item_stats_by_updated_at(
    pool: &PgPool,
    unique_name: &String,
) -> Result<Vec<db::ItemStatsByDay>, sqlx::Error> {
    return sqlx::query_as!(
        db::ItemStatsByDay,
        "SELECT 
            date,
            item_unique_name,
            total_count,
            max_unit_price_silver_offer,
            min_unit_price_silver_offer,
            avg_unit_price_silver_offer,
            sum_amount_offer,
            max_unit_price_silver_request,
            min_unit_price_silver_request,
            avg_unit_price_silver_request,
            sum_amount_request
        FROM 
            item_prices_by_updated_at
        WHERE 
            item_unique_name = $1
        ORDER BY
            date DESC",
        unique_name
    )
    .fetch_all(pool)
    .await;
}