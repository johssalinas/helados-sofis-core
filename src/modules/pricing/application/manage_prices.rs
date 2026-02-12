use std::sync::Arc;
use uuid::Uuid;

use crate::shared::errors::AppError;
use crate::modules::pricing::domain::entities::{CreatePriceDto, PriceHistory};
use crate::modules::pricing::domain::repositories::PriceRepository;

/// Caso de uso: Crear un nuevo precio (Temporal Data — NUNCA actualizar).
pub async fn create_price(
    repo: &Arc<dyn PriceRepository>,
    dto: CreatePriceDto,
    created_by: Uuid,
) -> Result<PriceHistory, AppError> {
    repo.create(&dto, created_by).await
}

/// Caso de uso: Obtener precio actual de un combo producto+sabor+proveedor.
pub async fn get_current_price(
    repo: &Arc<dyn PriceRepository>,
    product_id: Uuid,
    flavor_id: Uuid,
    provider_id: Uuid,
) -> Result<PriceHistory, AppError> {
    repo.get_current_price(product_id, flavor_id, provider_id)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(
                "No hay precio registrado para esta combinación producto/sabor/proveedor".into(),
            )
        })
}

/// Caso de uso: Listar todos los precios actuales.
pub async fn list_current_prices(
    repo: &Arc<dyn PriceRepository>,
) -> Result<Vec<PriceHistory>, AppError> {
    repo.list_current_prices().await
}

/// Caso de uso: Obtener historial de precios de un combo.
pub async fn get_price_history(
    repo: &Arc<dyn PriceRepository>,
    product_id: Uuid,
    flavor_id: Uuid,
    provider_id: Uuid,
) -> Result<Vec<PriceHistory>, AppError> {
    repo.get_history(product_id, flavor_id, provider_id).await
}
