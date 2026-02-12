use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct FreezerTransfer {
    pub id: Uuid,
    pub from_freezer_id: Uuid,
    pub to_freezer_id: Uuid,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct FreezerTransferItem {
    pub id: Uuid,
    pub transfer_id: Uuid,
    pub product_id: Uuid,
    pub flavor_id: Uuid,
    pub quantity: i32,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateTransferDto {
    pub from_freezer_id: Uuid,
    pub to_freezer_id: Uuid,
    pub reason: Option<String>,
    pub items: Vec<TransferItemDto>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct TransferItemDto {
    pub product_id: Uuid,
    pub flavor_id: Uuid,
    pub quantity: i32,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TransferWithItems {
    #[serde(flatten)]
    pub transfer: FreezerTransfer,
    pub items: Vec<FreezerTransferItem>,
}
