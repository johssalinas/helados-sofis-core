use async_trait::async_trait;
use uuid::Uuid;

use crate::shared::errors::AppError;
use super::entities::*;

#[async_trait]
pub trait FreezerTransferRepository: Send + Sync {
    async fn find_all(&self, limit: i64) -> Result<Vec<FreezerTransfer>, AppError>;
    async fn find_by_id_with_items(&self, id: Uuid) -> Result<Option<TransferWithItems>, AppError>;
    async fn find_by_freezer(&self, freezer_id: Uuid) -> Result<Vec<FreezerTransfer>, AppError>;
    /// Crea transferencia, resta inventario origen, suma inventario destino. Transaccional.
    async fn create_transfer(&self, dto: &CreateTransferDto, created_by: Uuid) -> Result<FreezerTransfer, AppError>;
}
