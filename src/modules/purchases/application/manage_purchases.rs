use std::sync::Arc;
use uuid::Uuid;

use crate::shared::errors::AppError;
use crate::modules::purchases::domain::entities::{CreatePurchaseDto, Purchase, PurchaseWithItems};
use crate::modules::purchases::domain::repositories::PurchaseRepository;

pub async fn list_purchases(
    repo: &Arc<dyn PurchaseRepository>,
) -> Result<Vec<Purchase>, AppError> {
    repo.find_all().await
}

pub async fn get_purchase(
    repo: &Arc<dyn PurchaseRepository>,
    id: Uuid,
) -> Result<PurchaseWithItems, AppError> {
    repo.find_by_id_with_items(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Compra {id} no encontrada")))
}

/// Crear compra e insertar stock en inventario automáticamente.
pub async fn create_purchase(
    repo: &Arc<dyn PurchaseRepository>,
    dto: CreatePurchaseDto,
    created_by: Uuid,
) -> Result<PurchaseWithItems, AppError> {
    // Validar payment_status
    if dto.payment_status != "paid" && dto.payment_status != "credit" {
        return Err(AppError::BadRequest(
            "payment_status debe ser 'paid' o 'credit'".into(),
        ));
    }
    repo.create(&dto, created_by).await
}
