use async_trait::async_trait;
use uuid::Uuid;

use super::entities::*;
use crate::shared::errors::AppError;

// ─── ProductRepository ──────────────────────────────────

#[async_trait]
pub trait ProductRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<Product>, AppError>;
    async fn find_active(&self) -> Result<Vec<Product>, AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Product>, AppError>;
    async fn create(&self, dto: &CreateProductDto, created_by: Uuid) -> Result<Product, AppError>;
    async fn update(
        &self,
        id: Uuid,
        dto: &UpdateProductDto,
        modified_by: Uuid,
    ) -> Result<Product, AppError>;
}

// ─── FlavorRepository ───────────────────────────────────

#[async_trait]
pub trait FlavorRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<Flavor>, AppError>;
    async fn find_by_product(&self, product_id: Uuid) -> Result<Vec<Flavor>, AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Flavor>, AppError>;
    async fn create(&self, dto: &CreateFlavorDto, created_by: Uuid) -> Result<Flavor, AppError>;
    async fn update(&self, id: Uuid, dto: &UpdateFlavorDto) -> Result<Flavor, AppError>;
}

// ─── ProviderRepository ─────────────────────────────────

#[async_trait]
pub trait ProviderRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<Provider>, AppError>;
    async fn find_active(&self) -> Result<Vec<Provider>, AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Provider>, AppError>;
    async fn create(&self, dto: &CreateProviderDto, created_by: Uuid)
        -> Result<Provider, AppError>;
    async fn update(&self, id: Uuid, dto: &UpdateProviderDto) -> Result<Provider, AppError>;
}

// ─── WorkerRepository ───────────────────────────────────

#[async_trait]
pub trait WorkerRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<Worker>, AppError>;
    async fn find_active(&self) -> Result<Vec<Worker>, AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Worker>, AppError>;
    async fn create(&self, dto: &CreateWorkerDto, created_by: Uuid) -> Result<Worker, AppError>;
    async fn update(&self, id: Uuid, dto: &UpdateWorkerDto) -> Result<Worker, AppError>;
}

// ─── RouteRepository ────────────────────────────────────

#[async_trait]
pub trait RouteRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<Route>, AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Route>, AppError>;
    async fn create(&self, dto: &CreateRouteDto, created_by: Uuid) -> Result<Route, AppError>;
}

// ─── FreezerRepository ──────────────────────────────────

#[async_trait]
pub trait FreezerRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<Freezer>, AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Freezer>, AppError>;
    async fn create(&self, dto: &CreateFreezerDto, created_by: Uuid) -> Result<Freezer, AppError>;
    async fn update(&self, id: Uuid, dto: &UpdateFreezerDto) -> Result<Freezer, AppError>;
    async fn toggle_power(&self, id: Uuid) -> Result<Freezer, AppError>;
}
