use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Purchase {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub total: Decimal,
    pub payment_status: String,
    pub paid_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PurchaseItem {
    pub id: Uuid,
    pub purchase_id: Uuid,
    pub product_id: Uuid,
    pub flavor_id: Uuid,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub freezer_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CreatePurchaseItemDto {
    pub product_id: Uuid,
    pub flavor_id: Uuid,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub freezer_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CreatePurchaseDto {
    pub provider_id: Uuid,
    pub payment_status: String, // "paid" o "credit"
    pub items: Vec<CreatePurchaseItemDto>,
}

#[derive(Debug, Serialize)]
pub struct PurchaseWithItems {
    #[serde(flatten)]
    pub purchase: Purchase,
    pub items: Vec<PurchaseItem>,
}
