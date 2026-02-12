use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::shared::errors::AppError;
use super::entities::{CreatePriceDto, PriceHistory};

/// Puerto de salida: persistencia de precios.
#[async_trait]
pub trait PriceRepository: Send + Sync {
    /// Obtener el precio actual (más reciente) de un combo producto+sabor+proveedor.
    async fn get_current_price(
        &self,
        product_id: Uuid,
        flavor_id: Uuid,
        provider_id: Uuid,
    ) -> Result<Option<PriceHistory>, AppError>;

    /// Obtener precio vigente en una fecha específica (para reportes históricos).
    async fn get_price_at(
        &self,
        product_id: Uuid,
        flavor_id: Uuid,
        provider_id: Uuid,
        date: DateTime<Utc>,
    ) -> Result<Option<PriceHistory>, AppError>;

    /// Crear un nuevo registro de precio (nunca se actualizan los existentes).
    async fn create(&self, dto: &CreatePriceDto, created_by: Uuid) -> Result<PriceHistory, AppError>;

    /// Obtener historial completo de precios de un combo.
    async fn get_history(
        &self,
        product_id: Uuid,
        flavor_id: Uuid,
        provider_id: Uuid,
    ) -> Result<Vec<PriceHistory>, AppError>;

    /// Listar todos los precios actuales (último de cada combo).
    async fn list_current_prices(&self) -> Result<Vec<PriceHistory>, AppError>;
}
