use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Entidades ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkerTrip {
    pub id: Uuid,
    pub worker_id: Uuid,
    pub departure_time: DateTime<Utc>,
    pub return_time: Option<DateTime<Utc>>,
    pub route_id: Option<Uuid>,
    pub status: String,
    pub sold_quantity: i32,
    pub amount_due: Decimal,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LoadedItem {
    pub id: Uuid,
    pub trip_id: Uuid,
    pub inventory_id: Uuid,
    pub product_id: Uuid,
    pub flavor_id: Uuid,
    pub freezer_id: Uuid,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub is_deformed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReturnedItem {
    pub id: Uuid,
    pub trip_id: Uuid,
    pub product_id: Uuid,
    pub flavor_id: Uuid,
    pub quantity: i32,
    pub is_deformed: bool,
    pub destination_freezer_id: Uuid,
}

// ─── DTOs ───────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateTripDto {
    pub worker_id: Uuid,
    pub departure_time: DateTime<Utc>,
    pub route_id: Option<Uuid>,
    pub loaded_items: Vec<LoadedItemDto>,
}

#[derive(Debug, Deserialize)]
pub struct LoadedItemDto {
    pub inventory_id: Uuid,
    pub product_id: Uuid,
    pub flavor_id: Uuid,
    pub freezer_id: Uuid,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub is_deformed: bool,
}

#[derive(Debug, Deserialize)]
pub struct CompleteTripDto {
    pub returned_items: Vec<ReturnedItemDto>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ReturnedItemDto {
    pub product_id: Uuid,
    pub flavor_id: Uuid,
    pub quantity: i32,
    pub is_deformed: bool,
    pub destination_freezer_id: Uuid,
}

// ─── Respuesta compuesta ────────────────────────────────

#[derive(Debug, Serialize)]
pub struct TripWithItems {
    #[serde(flatten)]
    pub trip: WorkerTrip,
    pub loaded_items: Vec<LoadedItem>,
    pub returned_items: Vec<ReturnedItem>,
}
