use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Registro de historial de precios (Temporal Data Pattern).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct PriceHistory {
    pub id: Uuid,
    pub product_id: Uuid,
    pub flavor_id: Uuid,
    pub provider_id: Uuid,
    pub cost_price: Decimal,
    pub price_base: Decimal,
    pub price_route: Decimal,
    pub price_local: Decimal,
    pub commission: Decimal,
    pub effective_date: DateTime<Utc>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

/// DTO para crear un nuevo precio (NO actualizar — Temporal Data).
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreatePriceDto {
    pub product_id: Uuid,
    pub flavor_id: Uuid,
    pub provider_id: Uuid,
    pub cost_price: Decimal,
    pub price_base: Decimal,
    pub price_route: Decimal,
    pub price_local: Decimal,
}
