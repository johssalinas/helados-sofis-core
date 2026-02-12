use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct LocalSale {
    pub id: Uuid,
    pub total: Decimal,
    pub sale_type: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct LocalSaleItem {
    pub id: Uuid,
    pub sale_id: Uuid,
    pub inventory_id: Uuid,
    pub product_id: Uuid,
    pub flavor_id: Uuid,
    pub freezer_id: Uuid,
    pub quantity: i32,
    pub unit_price: Decimal,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateLocalSaleDto {
    pub sale_type: String,
    pub notes: Option<String>,
    pub items: Vec<LocalSaleItemDto>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct LocalSaleItemDto {
    pub inventory_id: Uuid,
    pub product_id: Uuid,
    pub flavor_id: Uuid,
    pub freezer_id: Uuid,
    pub quantity: i32,
    pub unit_price: Decimal,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct LocalSaleWithItems {
    #[serde(flatten)]
    pub sale: LocalSale,
    pub items: Vec<LocalSaleItem>,
}
