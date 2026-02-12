use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct WorkerPayment {
    pub id: Uuid,
    pub worker_id: Uuid,
    pub trip_id: Uuid,
    pub amount: Decimal,
    pub previous_debt: Decimal,
    pub new_debt: Decimal,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreatePaymentDto {
    pub trip_id: Uuid,
}
