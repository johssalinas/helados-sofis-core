use std::sync::Arc;
use uuid::Uuid;

use crate::modules::inventory::domain::entities::{AddStockDto, InventoryItem};
use crate::modules::inventory::domain::repositories::InventoryRepository;
use crate::shared::errors::AppError;

pub async fn list_all(repo: &Arc<dyn InventoryRepository>) -> Result<Vec<InventoryItem>, AppError> {
    repo.find_all().await
}

pub async fn list_by_freezer(
    repo: &Arc<dyn InventoryRepository>,
    freezer_id: Uuid,
) -> Result<Vec<InventoryItem>, AppError> {
    repo.find_by_freezer(freezer_id).await
}

pub async fn list_sellable(
    repo: &Arc<dyn InventoryRepository>,
) -> Result<Vec<InventoryItem>, AppError> {
    repo.find_sellable().await
}

pub async fn list_low_stock(
    repo: &Arc<dyn InventoryRepository>,
) -> Result<Vec<InventoryItem>, AppError> {
    repo.find_low_stock().await
}

pub async fn list_worker_deformed(
    repo: &Arc<dyn InventoryRepository>,
    worker_id: Uuid,
) -> Result<Vec<InventoryItem>, AppError> {
    repo.find_worker_deformed(worker_id).await
}

pub async fn add_stock(
    repo: &Arc<dyn InventoryRepository>,
    dto: AddStockDto,
    updated_by: Uuid,
) -> Result<InventoryItem, AppError> {
    repo.add_stock(
        dto.freezer_id,
        dto.product_id,
        dto.flavor_id,
        dto.provider_id,
        dto.quantity,
        updated_by,
    )
    .await
}

pub async fn update_alert(
    repo: &Arc<dyn InventoryRepository>,
    id: Uuid,
    min_stock: i32,
) -> Result<InventoryItem, AppError> {
    repo.update_alert(id, min_stock).await
}
