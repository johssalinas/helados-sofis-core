use std::sync::Arc;

use axum::{
    extract::{Path, State},
    routing::{get, post, put},
    Json, Router,
};
use uuid::Uuid;

use crate::shared::auth::{AppState, AuthUser, Role};
use crate::shared::errors::AppError;
use crate::modules::catalog::application::crud;
use crate::modules::catalog::domain::entities::*;
use crate::modules::catalog::domain::repositories::*;

/// Estado del módulo catálogo compartido con handlers.
#[derive(Clone)]
pub struct CatalogState {
    pub app: AppState,
    pub products: Arc<dyn ProductRepository>,
    pub flavors: Arc<dyn FlavorRepository>,
    pub providers: Arc<dyn ProviderRepository>,
    pub workers: Arc<dyn WorkerRepository>,
    pub routes: Arc<dyn RouteRepository>,
    pub freezers: Arc<dyn FreezerRepository>,
}

impl axum::extract::FromRef<CatalogState> for AppState {
    fn from_ref(s: &CatalogState) -> AppState { s.app.clone() }
}

/// Crea el router completo del módulo catálogo.
pub fn router(state: CatalogState) -> Router {
    Router::new()
        // Products
        .route("/products", get(list_products).post(create_product))
        .route("/products/{id}", get(get_product).put(update_product))
        // Flavors
        .route("/flavors", get(list_flavors).post(create_flavor))
        .route("/flavors/{id}", put(update_flavor))
        .route("/products/{product_id}/flavors", get(list_product_flavors))
        // Providers
        .route("/providers", get(list_providers).post(create_provider))
        .route("/providers/{id}", put(update_provider))
        // Workers
        .route("/workers", get(list_workers).post(create_worker))
        .route("/workers/{id}", get(get_worker).put(update_worker))
        // Routes
        .route("/routes", get(list_routes).post(create_route))
        // Freezers
        .route("/freezers", get(list_freezers).post(create_freezer))
        .route("/freezers/{id}", get(get_freezer).put(update_freezer))
        .route("/freezers/{id}/toggle", post(toggle_freezer))
        .with_state(state)
}

// ─── Products ───────────────────────────────────────────

async fn list_products(
    auth: AuthUser,
    State(state): State<CatalogState>,
) -> Result<Json<Vec<Product>>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(crud::list_products(&state.products).await?))
}

async fn get_product(
    auth: AuthUser,
    State(state): State<CatalogState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Product>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(crud::get_product(&state.products, id).await?))
}

async fn create_product(
    auth: AuthUser,
    State(state): State<CatalogState>,
    Json(dto): Json<CreateProductDto>,
) -> Result<Json<Product>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(crud::create_product(&state.products, dto, auth.user_id()).await?))
}

async fn update_product(
    auth: AuthUser,
    State(state): State<CatalogState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateProductDto>,
) -> Result<Json<Product>, AppError> {
    auth.require_owner()?;
    Ok(Json(crud::update_product(&state.products, id, dto, auth.user_id()).await?))
}

// ─── Flavors ────────────────────────────────────────────

async fn list_flavors(
    auth: AuthUser,
    State(state): State<CatalogState>,
) -> Result<Json<Vec<Flavor>>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(crud::list_flavors(&state.flavors).await?))
}

async fn list_product_flavors(
    auth: AuthUser,
    State(state): State<CatalogState>,
    Path(product_id): Path<Uuid>,
) -> Result<Json<Vec<Flavor>>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(crud::list_flavors_by_product(&state.flavors, product_id).await?))
}

async fn create_flavor(
    auth: AuthUser,
    State(state): State<CatalogState>,
    Json(dto): Json<CreateFlavorDto>,
) -> Result<Json<Flavor>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(crud::create_flavor(&state.flavors, dto, auth.user_id()).await?))
}

async fn update_flavor(
    auth: AuthUser,
    State(state): State<CatalogState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateFlavorDto>,
) -> Result<Json<Flavor>, AppError> {
    auth.require_owner()?;
    Ok(Json(crud::update_flavor(&state.flavors, id, dto).await?))
}

// ─── Providers ──────────────────────────────────────────

async fn list_providers(
    auth: AuthUser,
    State(state): State<CatalogState>,
) -> Result<Json<Vec<Provider>>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(crud::list_providers(&state.providers).await?))
}

async fn create_provider(
    auth: AuthUser,
    State(state): State<CatalogState>,
    Json(dto): Json<CreateProviderDto>,
) -> Result<Json<Provider>, AppError> {
    auth.require_owner()?;
    Ok(Json(crud::create_provider(&state.providers, dto, auth.user_id()).await?))
}

async fn update_provider(
    auth: AuthUser,
    State(state): State<CatalogState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateProviderDto>,
) -> Result<Json<Provider>, AppError> {
    auth.require_owner()?;
    Ok(Json(crud::update_provider(&state.providers, id, dto).await?))
}

// ─── Workers ────────────────────────────────────────────

async fn list_workers(
    auth: AuthUser,
    State(state): State<CatalogState>,
) -> Result<Json<Vec<Worker>>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(crud::list_workers(&state.workers).await?))
}

async fn get_worker(
    auth: AuthUser,
    State(state): State<CatalogState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Worker>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(crud::get_worker(&state.workers, id).await?))
}

async fn create_worker(
    auth: AuthUser,
    State(state): State<CatalogState>,
    Json(dto): Json<CreateWorkerDto>,
) -> Result<Json<Worker>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(crud::create_worker(&state.workers, dto, auth.user_id()).await?))
}

async fn update_worker(
    auth: AuthUser,
    State(state): State<CatalogState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateWorkerDto>,
) -> Result<Json<Worker>, AppError> {
    auth.require_owner()?;
    Ok(Json(crud::update_worker(&state.workers, id, dto).await?))
}

// ─── Routes ─────────────────────────────────────────────

async fn list_routes(
    auth: AuthUser,
    State(state): State<CatalogState>,
) -> Result<Json<Vec<Route>>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(crud::list_routes(&state.routes).await?))
}

async fn create_route(
    auth: AuthUser,
    State(state): State<CatalogState>,
    Json(dto): Json<CreateRouteDto>,
) -> Result<Json<Route>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(crud::create_route(&state.routes, dto, auth.user_id()).await?))
}

// ─── Freezers ───────────────────────────────────────────

async fn list_freezers(
    auth: AuthUser,
    State(state): State<CatalogState>,
) -> Result<Json<Vec<Freezer>>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(crud::list_freezers(&state.freezers).await?))
}

async fn get_freezer(
    auth: AuthUser,
    State(state): State<CatalogState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Freezer>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(crud::get_freezer(&state.freezers, id).await?))
}

async fn create_freezer(
    auth: AuthUser,
    State(state): State<CatalogState>,
    Json(dto): Json<CreateFreezerDto>,
) -> Result<Json<Freezer>, AppError> {
    auth.require_owner()?;
    Ok(Json(crud::create_freezer(&state.freezers, dto, auth.user_id()).await?))
}

async fn update_freezer(
    auth: AuthUser,
    State(state): State<CatalogState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateFreezerDto>,
) -> Result<Json<Freezer>, AppError> {
    auth.require_owner()?;
    Ok(Json(crud::update_freezer(&state.freezers, id, dto).await?))
}

async fn toggle_freezer(
    auth: AuthUser,
    State(state): State<CatalogState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Freezer>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(crud::toggle_freezer(&state.freezers, id).await?))
}
