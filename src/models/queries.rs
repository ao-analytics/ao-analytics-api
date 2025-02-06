use std::collections::HashMap;

use chrono::Utc;

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct Item {
    pub unique_name: String,
}

#[derive(sqlx::FromRow, serde::Serialize, Clone)]
pub struct LocalizedName {
    pub item_unique_name: String,
    pub lang: String,
    pub name: String,
}

#[derive(sqlx::FromRow, serde::Serialize, Clone)]
pub struct LocalizedDescription {
    pub item_unique_name: String,
    pub lang: String,
    pub description: String,
}

#[derive(serde::Serialize)]
pub struct Localizations {
    pub unique_name: String,
    pub names: HashMap<String, String>,
    pub descriptions: HashMap<String, String>,
}

#[derive(serde::Serialize)]
pub struct LocalizationResponse {
    pub unique_name: String,
    pub name: Option<LocalizedName>,
    pub description: Option<LocalizedDescription>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct MarketOrder {
    pub id: i64,
    pub item_unique_name: String,
    pub location_id: i16,
    pub tier: Option<i16>,
    pub enchantment_level: i16,
    pub quality_level: i16,
    pub unit_price_silver: i64,
    pub amount: i64,
    pub auction_type: String,
    pub expires_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct MarketOrderCount {
    pub count: Option<i64>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct MarketHistoryCount {
    pub count: Option<i64>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct MarketOrderCountByItem {
    pub item_unique_name: String,
    pub count: Option<i64>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct MarketOrderCountByLocation {
    pub location: Option<String>,
    pub count: Option<i64>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct MarketOrderCountByAuctionType {
    pub auction_type: String,
    pub count: Option<i64>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct MarketOrderCountByQualityLevel {
    pub quality_level: i32,
    pub count: Option<i64>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct MarketOrderCountByEnchantmentLevel {
    pub enchantment_level: i32,
    pub count: Option<i64>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct MarketOrderCountByUpdatedAtAndLocation {
    pub date: Option<chrono::DateTime<Utc>>,
    pub location: Option<String>,
    pub count: Option<i64>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct MarketOrderCountByCreatedAtAndLocation {
    pub created_at: Option<chrono::DateTime<Utc>>,
    pub location: Option<String>,
    pub count: Option<i64>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct MarketOrderCountByUpdatedAt {
    pub date: Option<chrono::DateTime<Utc>>,
    pub count: Option<i64>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct MarketOrderCountByCreatedAt {
    pub created_at: Option<chrono::DateTime<Utc>>,
    pub count: Option<i64>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct Location {
    pub id: i16,
    pub name: String,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct SearchResult {
    pub item_unique_name: String,
    pub en_us: Option<String>,
    pub de_de: Option<String>,
    pub fr_fr: Option<String>,
    pub ru_ru: Option<String>,
    pub pl_pl: Option<String>,
    pub es_es: Option<String>,
    pub pt_br: Option<String>,
    pub it_it: Option<String>,
    pub zh_cn: Option<String>,
    pub ko_kr: Option<String>,
    pub ja_jp: Option<String>,
    pub zh_tw: Option<String>,
    pub id_id: Option<String>,
    pub tr_tr: Option<String>,
    pub ar_sa: Option<String>,
    pub rank: Option<f32>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct ItemStatsByDateAndLocation {
    pub date: Option<chrono::DateTime<Utc>>,
    pub item_unique_name: Option<String>,
    pub location_id: Option<i16>,
    pub total_count: Option<i64>,
    pub max_unit_price_silver_request: Option<i64>,
    pub min_unit_price_silver_request: Option<i64>,
    pub avg_unit_price_silver_request: Option<i64>,
    pub sum_amount_request: Option<i64>,
    pub max_unit_price_silver_offer: Option<i64>,
    pub min_unit_price_silver_offer: Option<i64>,
    pub avg_unit_price_silver_offer: Option<i64>,
    pub sum_amount_offer: Option<i64>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct ItemStatsByDate {
    pub date: Option<chrono::DateTime<Utc>>,
    pub item_unique_name: Option<String>,
    pub total_count: Option<i64>,
    pub max_unit_price_silver_request: Option<i64>,
    pub min_unit_price_silver_request: Option<i64>,
    pub avg_unit_price_silver_request: Option<i64>,
    pub sum_amount_request: Option<i64>,
    pub max_unit_price_silver_offer: Option<i64>,
    pub min_unit_price_silver_offer: Option<i64>,
    pub avg_unit_price_silver_offer: Option<i64>,
    pub sum_amount_offer: Option<i64>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct ItemMarketHistory {
    pub item_unique_name: String,
    pub timestamp: chrono::DateTime<Utc>,
    pub location_id: i16,
    pub quality_level: i16,
    pub item_amount: i64,
    pub silver_amount: i64,
    pub updated_at: chrono::DateTime<Utc>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct ItemMarketHistoryCountByUpdatedAt {
    pub date: Option<chrono::DateTime<Utc>>,
    pub count: Option<i64>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct ItemMarketHistoryCountByUpdatedAtAndLocation {
    pub date: Option<chrono::DateTime<Utc>>,
    pub location: Option<String>,
    pub count: Option<i64>,
}
