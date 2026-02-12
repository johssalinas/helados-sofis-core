use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OwnerSale {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub departure_time: DateTime<Utc>,
    pub return_time: Option<DateTime<Utc>>,
    pub route_id: Option<Uuid>,
    pub sold_quantity: i32,
    pub total_amount: Decimal,
    pub auto_withdrawal: Decimal,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OwnerSaleLoadedItem {
    pub id: Uuid,
    pub sale_id: Uuid,
    pub inventory_id: Uuid,
    pub product_id: Uuid,
    pub flavor_id: Uuid,
    pub freezer_id: Uuid,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub is_deformed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OwnerSaleReturnedItem {
    pub id: Uuid,
    pub sale_id: Uuid,
    pub product_id: Uuid,
    pub flavor_id: Uuid,
    pub quantity: i32,
    pub is_deformed: bool,
    pub destination_freezer_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CreateOwnerSaleDto {
    pub departure_time: DateTime<Utc>,
    pub route_id: Option<Uuid>,
    pub loaded_items: Vec<OwnerLoadedItemDto>,
}

#[derive(Debug, Deserialize)]
pub struct OwnerLoadedItemDto {
    pub inventory_id: Uuid,
    pub product_id: Uuid,
    pub flavor_id: Uuid,
    pub freezer_id: Uuid,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub is_deformed: bool,
}

#[derive(Debug, Deserialize)]
pub struct CompleteOwnerSaleDto {
    pub returned_items: Vec<OwnerReturnedItemDto>,
}

#[derive(Debug, Deserialize)]
pub struct OwnerReturnedItemDto {
    pub product_id: Uuid,
    pub flavor_id: Uuid,
    pub quantity: i32,
    pub is_deformed: bool,
    pub destination_freezer_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct OwnerSaleWithItems {
    #[serde(flatten)]
    pub sale: OwnerSale,
    pub loaded_items: Vec<OwnerSaleLoadedItem>,
    pub returned_items: Vec<OwnerSaleReturnedItem>,
}
