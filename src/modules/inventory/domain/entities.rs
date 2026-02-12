use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Item de inventario — una "pila" homogénea por congelador.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct InventoryItem {
    pub id: Uuid,
    pub freezer_id: Uuid,
    pub product_id: Uuid,
    pub flavor_id: Uuid,
    pub provider_id: Uuid,
    pub quantity: i32,
    pub min_stock_alert: i32,
    pub is_deformed: bool,
    pub assigned_worker_id: Option<Uuid>,
    pub last_updated: DateTime<Utc>,
    pub updated_by: Uuid,
}

/// DTO para agregar stock (compra a proveedor).
#[derive(Debug, Deserialize)]
pub struct AddStockDto {
    pub freezer_id: Uuid,
    pub product_id: Uuid,
    pub flavor_id: Uuid,
    pub provider_id: Uuid,
    pub quantity: i32,
}

/// DTO para actualizar alerta de stock mínimo.
#[derive(Debug, Deserialize)]
pub struct UpdateAlertDto {
    pub min_stock_alert: i32,
}
