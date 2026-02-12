use async_trait::async_trait;
use uuid::Uuid;

use crate::shared::errors::AppError;
use super::entities::{CreatePurchaseDto, Purchase, PurchaseWithItems};

#[async_trait]
pub trait PurchaseRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<Purchase>, AppError>;
    async fn find_by_id_with_items(&self, id: Uuid) -> Result<Option<PurchaseWithItems>, AppError>;
    async fn create(
        &self,
        dto: &CreatePurchaseDto,
        created_by: Uuid,
    ) -> Result<PurchaseWithItems, AppError>;
}
