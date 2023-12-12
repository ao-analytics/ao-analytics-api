use sqlx::PgPool;

use crate::models::db;

pub async fn search_items_by_localized_name(
    pool: &PgPool,
    lang: &str,
    item: &str,
) -> Result<Vec<db::LocalizedName>, sqlx::Error> {
    let item = format!("%{}%", item);

    return sqlx::query_as!(
        db::LocalizedName,
        "SELECT 
            item_unique_name, 
            en_us,
            de_de,
            fr_fr,
            ru_ru,
            pl_pl,
            es_es,
            pt_br,
            it_it,
            zh_cn,
            ko_kr,
            ja_jp,
            zh_tw,
            id_id
        FROM 
            localized_name
        WHERE 
            ( $1 = 'en_us' AND en_us ILIKE $2 )
            OR ( $1 = 'de_de' AND de_de ILIKE $2 )
            OR ( $1 = 'fr_fr' AND fr_fr ILIKE $2 )
            OR ( $1 = 'ru_ru' AND ru_ru ILIKE $2 )
            OR ( $1 = 'pl_pl' AND pl_pl ILIKE $2 )
            OR ( $1 = 'es_es' AND es_es ILIKE $2 )
            OR ( $1 = 'pt_br' AND pt_br ILIKE $2 )
            OR ( $1 = 'it_it' AND it_it ILIKE $2 )
            OR ( $1 = 'zh_cn' AND zh_cn ILIKE $2 )
            OR ( $1 = 'ko_kr' AND ko_kr ILIKE $2 )
            OR ( $1 = 'ja_jp' AND ja_jp ILIKE $2 )
            OR ( $1 = 'zh_tw' AND zh_tw ILIKE $2 )
            OR ( $1 = 'id_id' AND id_id ILIKE $2 )
        ORDER BY 
            en_us ASC
        LIMIT 10",
        lang,
        item
    )
    .fetch_all(pool)
    .await;
}

pub async fn query_market_orders(
    pool: &PgPool,
    unique_name: Option<String>,
    location_id: Option<String>,
    auction_type: Option<String>,
    quality_level: Option<i32>,
    enchantment_level: Option<i32>,
    limit: i64,
    offset: i64,
) -> Result<Vec<db::MarketOrder>, sqlx::Error> {
    return sqlx::query_as!(
        db::MarketOrder,
        "SELECT 
            market_order.id,
            location.id as location_id,
            item_unique_name,
            quality_level, 
            enchantment_level, 
            unit_price_silver, 
            amount, 
            auction_type, 
            expires_at, 
            updated_at,
            created_at
        FROM 
            market_order, location 
        WHERE 
            location_id = location.id
            AND expires_at > NOW()
            AND ( $1::TEXT IS NULL OR item_unique_name = $1 )
            AND ( $2::TEXT IS NULL OR location.id = $2 )
            AND ( $3::TEXT IS NULL OR auction_type = $3 )
            AND ( $4::INT IS NULL OR quality_level = $4 )
            AND ( $5::INT IS NULL OR enchantment_level = $5 )
        ORDER BY 
            unit_price_silver ASC
        LIMIT $6 
        OFFSET $7",
        unique_name,
        location_id,
        auction_type,
        quality_level,
        enchantment_level,
        limit,
        offset
    )
    .fetch_all(pool)
    .await;
}

pub async fn get_localized_names_by_unique_name(
    pool: &PgPool,
    unique_name: &String,
) -> Result<db::LocalizedName, sqlx::Error> {
    return sqlx::query_as!(
        db::LocalizedName,
        "SELECT 
            item_unique_name, 
            en_us, 
            de_de, 
            fr_fr, 
            ru_ru, 
            pl_pl, 
            es_es, 
            pt_br, 
            it_it, 
            zh_cn, 
            ko_kr, 
            ja_jp, 
            zh_tw, 
            id_id 
        FROM localized_name 
            WHERE item_unique_name = $1",
        unique_name
    )
    .fetch_one(pool)
    .await;
}

pub async fn get_localized_descriptions_by_unique_name(
    pool: &PgPool,
    unique_name: &String,
) -> Result<db::LocalizedDescription, sqlx::Error> {
    return sqlx::query_as!(
        db::LocalizedDescription,
        "SELECT 
            item_unique_name, 
            en_us, 
            de_de, 
            fr_fr, 
            ru_ru, 
            pl_pl, 
            es_es, 
            pt_br, 
            it_it, 
            zh_cn, 
            ko_kr, 
            ja_jp, 
            zh_tw, 
            id_id 
        FROM 
            localized_description 
        WHERE 
            item_unique_name = $1",
        unique_name
    )
    .fetch_one(pool)
    .await;
}

pub async fn get_market_orders_count(pool: &PgPool) -> Result<db::MarketOrderCount, sqlx::Error> {
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

pub async fn get_market_orders_count_by_location(
    pool: &PgPool,
) -> Result<Vec<db::MarketOrderCountByLocation>, sqlx::Error> {
    return sqlx::query_as!(
        db::MarketOrderCountByLocation,
        "SELECT 
            location.name as location, 
            COUNT(*) as count 
        FROM 
            market_order, 
            location 
        WHERE 
            location_id = location.id 
        GROUP BY 
            location.name 
        ORDER BY
            count DESC"
    )
    .fetch_all(pool)
    .await;
}

pub async fn get_market_orders_count_by_auction_type(
    pool: &PgPool,
) -> Result<Vec<db::MarketOrderCountByAuctionType>, sqlx::Error> {
    return sqlx::query_as!(
        db::MarketOrderCountByAuctionType,
        "SELECT 
            auction_type, 
            COUNT(*) as count 
        FROM 
            market_order 
        GROUP BY 
            auction_type 
        ORDER BY 
            count DESC"
    )
    .fetch_all(pool)
    .await;
}

pub async fn get_market_orders_count_by_updated_at(
    pool: &PgPool,
) -> Result<Vec<db::MarketOrderCountByUpdatedAt>, sqlx::Error> {
    return sqlx::query_as!(
        db::MarketOrderCountByUpdatedAt,
        "SELECT 
            DATE_TRUNC('hour', updated_at)  AS updated_at,
            COUNT(*) as count
        FROM
            market_order
        WHERE 
            expires_at > NOW()
        GROUP BY
            DATE_TRUNC('hour', updated_at)
            
        ORDER BY
            updated_at DESC"
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
            DATE_TRUNC('hour', updated_at) AS updated_at,
            location.name as location,
            COUNT(*) as count
        FROM
            market_order,   
            location
        WHERE
            location_id = location.id
            AND expires_at > NOW()
        GROUP BY
            DATE_TRUNC('hour', updated_at),
            location
        ORDER BY
            updated_at DESC"
    )
    .fetch_all(pool)
    .await;
}

pub async fn get_market_orders_count_by_created_at(
    pool: &PgPool,
) -> Result<Vec<db::MarketOrderCountByCreatedAt>, sqlx::Error> {
    return sqlx::query_as!(
        db::MarketOrderCountByCreatedAt,
        "SELECT 
            DATE_TRUNC('hour', created_at) AS created_at,
            COUNT(*) as count
        FROM
            market_order
        WHERE 
            expires_at > NOW()
        GROUP BY
            DATE_TRUNC('hour', created_at)
        ORDER BY
            created_at DESC"
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
        ) AS market_order_count ON market_order_count.location_id = location.id
        WHERE 
            ( $1::INT IS NULL OR $1 <= COALESCE(market_order_count.count, 0) )",
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
