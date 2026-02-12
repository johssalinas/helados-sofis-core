use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Products ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub modified_at: Option<DateTime<Utc>>,
    pub modified_by: Option<Uuid>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateProductDto {
    pub name: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateProductDto {
    pub name: Option<String>,
    pub active: Option<bool>,
}

// ─── Flavors ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct Flavor {
    pub id: Uuid,
    pub name: String,
    pub product_id: Uuid,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateFlavorDto {
    pub name: String,
    pub product_id: Uuid,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateFlavorDto {
    pub name: Option<String>,
    pub active: Option<bool>,
}

// ─── Providers ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct Provider {
    pub id: Uuid,
    pub name: String,
    pub contact_info: Option<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateProviderDto {
    pub name: String,
    pub contact_info: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateProviderDto {
    pub name: Option<String>,
    pub contact_info: Option<String>,
    pub active: Option<bool>,
}

// ─── Workers ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct Worker {
    pub id: Uuid,
    pub name: String,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub active: bool,
    pub current_debt: Decimal,
    pub total_sales: i32,
    pub last_sale: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateWorkerDto {
    pub name: String,
    pub phone: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateWorkerDto {
    pub name: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub active: Option<bool>,
}

// ─── Routes ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct Route {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub usage_count: i32,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateRouteDto {
    pub name: String,
}

// ─── Freezers ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct Freezer {
    pub id: Uuid,
    pub number: i32,
    #[schema(value_type = Object)]
    pub max_capacity: serde_json::Value,
    pub is_on: bool,
    pub last_toggle: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateFreezerDto {
    pub number: i32,
    #[schema(value_type = Option<Object>)]
    pub max_capacity: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateFreezerDto {
    #[schema(value_type = Option<Object>)]
    pub max_capacity: Option<serde_json::Value>,
    pub is_on: Option<bool>,
}
