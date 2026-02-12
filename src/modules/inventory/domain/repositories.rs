use async_trait::async_trait;
use uuid::Uuid;

use super::entities::InventoryItem;
use crate::shared::errors::AppError;

/// Puerto de salida: persistencia de inventario.
#[async_trait]
pub trait InventoryRepository: Send + Sync {
    /// Listar todo el inventario.
    async fn find_all(&self) -> Result<Vec<InventoryItem>, AppError>;

    /// Obtener inventario de un congelador.
    async fn find_by_freezer(&self, freezer_id: Uuid) -> Result<Vec<InventoryItem>, AppError>;

    /// Obtener un item por ID.
    async fn find_by_id(&self, id: Uuid) -> Result<Option<InventoryItem>, AppError>;

    /// Stock vendible (no deformado).
    async fn find_sellable(&self) -> Result<Vec<InventoryItem>, AppError>;

    /// Items con stock bajo.
    async fn find_low_stock(&self) -> Result<Vec<InventoryItem>, AppError>;

    /// Deformados asignados a un trabajador.
    async fn find_worker_deformed(&self, worker_id: Uuid) -> Result<Vec<InventoryItem>, AppError>;

    /// Agregar stock (UPSERT: crea o incrementa).
    async fn add_stock(
        &self,
        freezer_id: Uuid,
        product_id: Uuid,
        flavor_id: Uuid,
        provider_id: Uuid,
        quantity: i32,
        updated_by: Uuid,
    ) -> Result<InventoryItem, AppError>;

    /// Restar stock con verificación. Devuelve error si es insuficiente.
    async fn subtract_stock_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        inventory_id: Uuid,
        quantity: i32,
        updated_by: Uuid,
    ) -> Result<(), AppError>;

    /// Agregar stock deformado asignado a trabajador.
    async fn add_deformed_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        freezer_id: Uuid,
        product_id: Uuid,
        flavor_id: Uuid,
        provider_id: Uuid,
        quantity: i32,
        worker_id: Uuid,
        updated_by: Uuid,
    ) -> Result<InventoryItem, AppError>;

    /// Devolver stock normal (UPSERT en transacción).
    async fn return_stock_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        freezer_id: Uuid,
        product_id: Uuid,
        flavor_id: Uuid,
        provider_id: Uuid,
        quantity: i32,
        updated_by: Uuid,
    ) -> Result<(), AppError>;

    /// Actualizar alerta de stock mínimo.
    async fn update_alert(&self, id: Uuid, min_stock: i32) -> Result<InventoryItem, AppError>;
}
