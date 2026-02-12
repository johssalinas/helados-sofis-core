use async_trait::async_trait;
use uuid::Uuid;

use super::entities::{CreateLocalSaleDto, LocalSale, LocalSaleWithItems};
use crate::shared::errors::AppError;

#[async_trait]
pub trait LocalSaleRepository: Send + Sync {
    async fn find_all(&self, limit: i64) -> Result<Vec<LocalSale>, AppError>;
    async fn find_by_id_with_items(&self, id: Uuid)
        -> Result<Option<LocalSaleWithItems>, AppError>;
    async fn find_todays(&self) -> Result<Vec<LocalSale>, AppError>;
    /// Crea venta, resta inventario, registra en caja. Todo transaccional.
    async fn create_sale(
        &self,
        dto: &CreateLocalSaleDto,
        created_by: Uuid,
    ) -> Result<LocalSale, AppError>;
}
