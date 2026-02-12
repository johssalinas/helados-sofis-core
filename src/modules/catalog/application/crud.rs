use std::sync::Arc;
use uuid::Uuid;

use crate::modules::catalog::domain::entities::*;
use crate::modules::catalog::domain::repositories::*;
use crate::shared::errors::AppError;

// ─── Products ───────────────────────────────────────────

pub async fn list_products(repo: &Arc<dyn ProductRepository>) -> Result<Vec<Product>, AppError> {
    repo.find_active().await
}

pub async fn get_product(repo: &Arc<dyn ProductRepository>, id: Uuid) -> Result<Product, AppError> {
    repo.find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Producto {id} no encontrado")))
}

pub async fn create_product(
    repo: &Arc<dyn ProductRepository>,
    dto: CreateProductDto,
    created_by: Uuid,
) -> Result<Product, AppError> {
    repo.create(&dto, created_by).await
}

pub async fn update_product(
    repo: &Arc<dyn ProductRepository>,
    id: Uuid,
    dto: UpdateProductDto,
    modified_by: Uuid,
) -> Result<Product, AppError> {
    let _ = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Producto {id} no encontrado")))?;
    repo.update(id, &dto, modified_by).await
}

// ─── Flavors ────────────────────────────────────────────

pub async fn list_flavors(repo: &Arc<dyn FlavorRepository>) -> Result<Vec<Flavor>, AppError> {
    repo.find_all().await
}

pub async fn list_flavors_by_product(
    repo: &Arc<dyn FlavorRepository>,
    product_id: Uuid,
) -> Result<Vec<Flavor>, AppError> {
    repo.find_by_product(product_id).await
}

pub async fn create_flavor(
    repo: &Arc<dyn FlavorRepository>,
    dto: CreateFlavorDto,
    created_by: Uuid,
) -> Result<Flavor, AppError> {
    repo.create(&dto, created_by).await
}

pub async fn update_flavor(
    repo: &Arc<dyn FlavorRepository>,
    id: Uuid,
    dto: UpdateFlavorDto,
) -> Result<Flavor, AppError> {
    let _ = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Sabor {id} no encontrado")))?;
    repo.update(id, &dto).await
}

// ─── Providers ──────────────────────────────────────────

pub async fn list_providers(repo: &Arc<dyn ProviderRepository>) -> Result<Vec<Provider>, AppError> {
    repo.find_active().await
}

pub async fn create_provider(
    repo: &Arc<dyn ProviderRepository>,
    dto: CreateProviderDto,
    created_by: Uuid,
) -> Result<Provider, AppError> {
    repo.create(&dto, created_by).await
}

pub async fn update_provider(
    repo: &Arc<dyn ProviderRepository>,
    id: Uuid,
    dto: UpdateProviderDto,
) -> Result<Provider, AppError> {
    let _ = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Proveedor {id} no encontrado")))?;
    repo.update(id, &dto).await
}

// ─── Workers ────────────────────────────────────────────

pub async fn list_workers(repo: &Arc<dyn WorkerRepository>) -> Result<Vec<Worker>, AppError> {
    repo.find_active().await
}

pub async fn get_worker(repo: &Arc<dyn WorkerRepository>, id: Uuid) -> Result<Worker, AppError> {
    repo.find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Trabajador {id} no encontrado")))
}

pub async fn create_worker(
    repo: &Arc<dyn WorkerRepository>,
    dto: CreateWorkerDto,
    created_by: Uuid,
) -> Result<Worker, AppError> {
    repo.create(&dto, created_by).await
}

pub async fn update_worker(
    repo: &Arc<dyn WorkerRepository>,
    id: Uuid,
    dto: UpdateWorkerDto,
) -> Result<Worker, AppError> {
    let _ = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Trabajador {id} no encontrado")))?;
    repo.update(id, &dto).await
}

// ─── Routes ─────────────────────────────────────────────

pub async fn list_routes(repo: &Arc<dyn RouteRepository>) -> Result<Vec<Route>, AppError> {
    repo.find_all().await
}

pub async fn create_route(
    repo: &Arc<dyn RouteRepository>,
    dto: CreateRouteDto,
    created_by: Uuid,
) -> Result<Route, AppError> {
    repo.create(&dto, created_by).await
}

// ─── Freezers ───────────────────────────────────────────

pub async fn list_freezers(repo: &Arc<dyn FreezerRepository>) -> Result<Vec<Freezer>, AppError> {
    repo.find_all().await
}

pub async fn get_freezer(repo: &Arc<dyn FreezerRepository>, id: Uuid) -> Result<Freezer, AppError> {
    repo.find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Congelador {id} no encontrado")))
}

pub async fn create_freezer(
    repo: &Arc<dyn FreezerRepository>,
    dto: CreateFreezerDto,
    created_by: Uuid,
) -> Result<Freezer, AppError> {
    repo.create(&dto, created_by).await
}

pub async fn update_freezer(
    repo: &Arc<dyn FreezerRepository>,
    id: Uuid,
    dto: UpdateFreezerDto,
) -> Result<Freezer, AppError> {
    let _ = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Congelador {id} no encontrado")))?;
    repo.update(id, &dto).await
}

pub async fn toggle_freezer(
    repo: &Arc<dyn FreezerRepository>,
    id: Uuid,
) -> Result<Freezer, AppError> {
    let _ = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Congelador {id} no encontrado")))?;
    repo.toggle_power(id).await
}
