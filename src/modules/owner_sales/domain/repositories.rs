use async_trait::async_trait;
use uuid::Uuid;

use crate::shared::errors::AppError;
use super::entities::*;

#[async_trait]
pub trait OwnerSaleRepository: Send + Sync {
    async fn find_all(&self, limit: i64) -> Result<Vec<OwnerSale>, AppError>;
    async fn find_by_id_with_items(&self, id: Uuid) -> Result<Option<OwnerSaleWithItems>, AppError>;
    async fn create_sale(&self, dto: &CreateOwnerSaleDto, owner_id: Uuid) -> Result<OwnerSale, AppError>;
    /// Completa venta, procesa devoluciones, registra 2 eventos en caja (ingreso + retiro).
    async fn complete_sale(&self, sale_id: Uuid, dto: &CompleteOwnerSaleDto, owner_id: Uuid) -> Result<OwnerSale, AppError>;
}
