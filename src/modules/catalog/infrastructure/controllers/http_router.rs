use std::sync::Arc;

use axum::{
    extract::{Path, State},
    routing::{get, post, put},
    Json, Router,
};
use utoipa::OpenApi;
use uuid::Uuid;

use crate::modules::catalog::application::crud;
use crate::modules::catalog::domain::entities::*;
use crate::modules::catalog::domain::repositories::*;
use crate::shared::auth::{AppState, AuthUser, Role};
use crate::shared::errors::AppError;

#[derive(OpenApi)]
#[openapi(
    paths(
        list_products,
        get_product,
        create_product,
        update_product,
        list_flavors,
        list_product_flavors,
        create_flavor,
        update_flavor,
        list_providers,
        create_provider,
        update_provider,
        list_workers,
        get_worker,
        create_worker,
        update_worker,
        list_routes,
        create_route,
        list_freezers,
        get_freezer,
        create_freezer,
        update_freezer,
        toggle_freezer,
    ),
    components(schemas(
        crate::modules::catalog::domain::entities::Product,
        crate::modules::catalog::domain::entities::CreateProductDto,
        crate::modules::catalog::domain::entities::UpdateProductDto,
        crate::modules::catalog::domain::entities::Flavor,
        crate::modules::catalog::domain::entities::CreateFlavorDto,
        crate::modules::catalog::domain::entities::UpdateFlavorDto,
        crate::modules::catalog::domain::entities::Provider,
        crate::modules::catalog::domain::entities::CreateProviderDto,
        crate::modules::catalog::domain::entities::UpdateProviderDto,
        crate::modules::catalog::domain::entities::Worker,
        crate::modules::catalog::domain::entities::CreateWorkerDto,
        crate::modules::catalog::domain::entities::UpdateWorkerDto,
        crate::modules::catalog::domain::entities::Route,
        crate::modules::catalog::domain::entities::CreateRouteDto,
        crate::modules::catalog::domain::entities::Freezer,
        crate::modules::catalog::domain::entities::CreateFreezerDto,
        crate::modules::catalog::domain::entities::UpdateFreezerDto,
    ))
)]
pub struct CatalogApiDoc;

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
    fn from_ref(s: &CatalogState) -> AppState {
        s.app.clone()
    }
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
#[utoipa::path(
    get, path = "/products", tag = "Catálogo - Productos",
    responses((status = 200, description = "Lista de productos", body = Vec<Product>)),
    security(("bearer_auth" = []))
)]
async fn list_products(
    auth: AuthUser,
    State(state): State<CatalogState>,
) -> Result<Json<Vec<Product>>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(crud::list_products(&state.products).await?))
}

#[utoipa::path(
    get, path = "/products/{id}", tag = "Catálogo - Productos",
    params(("id" = Uuid, Path, description = "ID del producto")),
    responses(
        (status = 200, description = "Producto encontrado", body = Product),
        (status = 404, description = "No encontrado")
    ),
    security(("bearer_auth" = []))
)]
async fn get_product(
    auth: AuthUser,
    State(state): State<CatalogState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Product>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(crud::get_product(&state.products, id).await?))
}

#[utoipa::path(
    post, path = "/products", tag = "Catálogo - Productos",
    request_body = CreateProductDto,
    responses((status = 200, description = "Producto creado", body = Product)),
    security(("bearer_auth" = []))
)]
async fn create_product(
    auth: AuthUser,
    State(state): State<CatalogState>,
    Json(dto): Json<CreateProductDto>,
) -> Result<Json<Product>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(
        crud::create_product(&state.products, dto, auth.user_id()).await?,
    ))
}

#[utoipa::path(
    put, path = "/products/{id}", tag = "Catálogo - Productos",
    params(("id" = Uuid, Path, description = "ID del producto")),
    request_body = UpdateProductDto,
    responses((status = 200, description = "Producto actualizado", body = Product)),
    security(("bearer_auth" = []))
)]
async fn update_product(
    auth: AuthUser,
    State(state): State<CatalogState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateProductDto>,
) -> Result<Json<Product>, AppError> {
    auth.require_owner()?;
    Ok(Json(
        crud::update_product(&state.products, id, dto, auth.user_id()).await?,
    ))
}

// ─── Flavors ────────────────────────────────────────────
#[utoipa::path(
    get, path = "/flavors", tag = "Catálogo - Sabores",
    responses((status = 200, description = "Lista de sabores", body = Vec<Flavor>)),
    security(("bearer_auth" = []))
)]
async fn list_flavors(
    auth: AuthUser,
    State(state): State<CatalogState>,
) -> Result<Json<Vec<Flavor>>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(crud::list_flavors(&state.flavors).await?))
}

#[utoipa::path(
    get, path = "/products/{product_id}/flavors", tag = "Catálogo - Sabores",
    params(("product_id" = Uuid, Path, description = "ID del producto")),
    responses((status = 200, description = "Sabores del producto", body = Vec<Flavor>)),
    security(("bearer_auth" = []))
)]
async fn list_product_flavors(
    auth: AuthUser,
    State(state): State<CatalogState>,
    Path(product_id): Path<Uuid>,
) -> Result<Json<Vec<Flavor>>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(
        crud::list_flavors_by_product(&state.flavors, product_id).await?,
    ))
}

#[utoipa::path(
    post, path = "/flavors", tag = "Catálogo - Sabores",
    request_body = CreateFlavorDto,
    responses((status = 200, description = "Sabor creado", body = Flavor)),
    security(("bearer_auth" = []))
)]
async fn create_flavor(
    auth: AuthUser,
    State(state): State<CatalogState>,
    Json(dto): Json<CreateFlavorDto>,
) -> Result<Json<Flavor>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(
        crud::create_flavor(&state.flavors, dto, auth.user_id()).await?,
    ))
}

#[utoipa::path(
    put, path = "/flavors/{id}", tag = "Catálogo - Sabores",
    params(("id" = Uuid, Path, description = "ID del sabor")),
    request_body = UpdateFlavorDto,
    responses((status = 200, description = "Sabor actualizado", body = Flavor)),
    security(("bearer_auth" = []))
)]
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
#[utoipa::path(
    get, path = "/providers", tag = "Catálogo - Proveedores",
    responses((status = 200, description = "Lista de proveedores", body = Vec<Provider>)),
    security(("bearer_auth" = []))
)]
async fn list_providers(
    auth: AuthUser,
    State(state): State<CatalogState>,
) -> Result<Json<Vec<Provider>>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(crud::list_providers(&state.providers).await?))
}

#[utoipa::path(
    post, path = "/providers", tag = "Catálogo - Proveedores",
    request_body = CreateProviderDto,
    responses((status = 200, description = "Proveedor creado", body = Provider)),
    security(("bearer_auth" = []))
)]
async fn create_provider(
    auth: AuthUser,
    State(state): State<CatalogState>,
    Json(dto): Json<CreateProviderDto>,
) -> Result<Json<Provider>, AppError> {
    auth.require_owner()?;
    Ok(Json(
        crud::create_provider(&state.providers, dto, auth.user_id()).await?,
    ))
}

#[utoipa::path(
    put, path = "/providers/{id}", tag = "Catálogo - Proveedores",
    params(("id" = Uuid, Path, description = "ID del proveedor")),
    request_body = UpdateProviderDto,
    responses((status = 200, description = "Proveedor actualizado", body = Provider)),
    security(("bearer_auth" = []))
)]
async fn update_provider(
    auth: AuthUser,
    State(state): State<CatalogState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateProviderDto>,
) -> Result<Json<Provider>, AppError> {
    auth.require_owner()?;
    Ok(Json(
        crud::update_provider(&state.providers, id, dto).await?,
    ))
}

// ─── Workers ────────────────────────────────────────────
#[utoipa::path(
    get, path = "/workers", tag = "Catálogo - Trabajadores",
    responses((status = 200, description = "Lista de trabajadores", body = Vec<Worker>)),
    security(("bearer_auth" = []))
)]
async fn list_workers(
    auth: AuthUser,
    State(state): State<CatalogState>,
) -> Result<Json<Vec<Worker>>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(crud::list_workers(&state.workers).await?))
}

#[utoipa::path(
    get, path = "/workers/{id}", tag = "Catálogo - Trabajadores",
    params(("id" = Uuid, Path, description = "ID del trabajador")),
    responses((status = 200, description = "Trabajador encontrado", body = Worker)),
    security(("bearer_auth" = []))
)]
async fn get_worker(
    auth: AuthUser,
    State(state): State<CatalogState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Worker>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(crud::get_worker(&state.workers, id).await?))
}

#[utoipa::path(
    post, path = "/workers", tag = "Catálogo - Trabajadores",
    request_body = CreateWorkerDto,
    responses((status = 200, description = "Trabajador creado", body = Worker)),
    security(("bearer_auth" = []))
)]
async fn create_worker(
    auth: AuthUser,
    State(state): State<CatalogState>,
    Json(dto): Json<CreateWorkerDto>,
) -> Result<Json<Worker>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(
        crud::create_worker(&state.workers, dto, auth.user_id()).await?,
    ))
}

#[utoipa::path(
    put, path = "/workers/{id}", tag = "Catálogo - Trabajadores",
    params(("id" = Uuid, Path, description = "ID del trabajador")),
    request_body = UpdateWorkerDto,
    responses((status = 200, description = "Trabajador actualizado", body = Worker)),
    security(("bearer_auth" = []))
)]
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
#[utoipa::path(
    get, path = "/routes", tag = "Catálogo - Rutas",
    responses((status = 200, description = "Lista de rutas", body = Vec<Route>)),
    security(("bearer_auth" = []))
)]
async fn list_routes(
    auth: AuthUser,
    State(state): State<CatalogState>,
) -> Result<Json<Vec<Route>>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(crud::list_routes(&state.routes).await?))
}

#[utoipa::path(
    post, path = "/routes", tag = "Catálogo - Rutas",
    request_body = CreateRouteDto,
    responses((status = 200, description = "Ruta creada", body = Route)),
    security(("bearer_auth" = []))
)]
async fn create_route(
    auth: AuthUser,
    State(state): State<CatalogState>,
    Json(dto): Json<CreateRouteDto>,
) -> Result<Json<Route>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(
        crud::create_route(&state.routes, dto, auth.user_id()).await?,
    ))
}

// ─── Freezers ───────────────────────────────────────────
#[utoipa::path(
    get, path = "/freezers", tag = "Catálogo - Congeladores",
    responses((status = 200, description = "Lista de congeladores", body = Vec<Freezer>)),
    security(("bearer_auth" = []))
)]
async fn list_freezers(
    auth: AuthUser,
    State(state): State<CatalogState>,
) -> Result<Json<Vec<Freezer>>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(crud::list_freezers(&state.freezers).await?))
}

#[utoipa::path(
    get, path = "/freezers/{id}", tag = "Catálogo - Congeladores",
    params(("id" = Uuid, Path, description = "ID del congelador")),
    responses((status = 200, description = "Congelador encontrado", body = Freezer)),
    security(("bearer_auth" = []))
)]
async fn get_freezer(
    auth: AuthUser,
    State(state): State<CatalogState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Freezer>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(crud::get_freezer(&state.freezers, id).await?))
}

#[utoipa::path(
    post, path = "/freezers", tag = "Catálogo - Congeladores",
    request_body = CreateFreezerDto,
    responses((status = 200, description = "Congelador creado", body = Freezer)),
    security(("bearer_auth" = []))
)]
async fn create_freezer(
    auth: AuthUser,
    State(state): State<CatalogState>,
    Json(dto): Json<CreateFreezerDto>,
) -> Result<Json<Freezer>, AppError> {
    auth.require_owner()?;
    Ok(Json(
        crud::create_freezer(&state.freezers, dto, auth.user_id()).await?,
    ))
}

#[utoipa::path(
    put, path = "/freezers/{id}", tag = "Catálogo - Congeladores",
    params(("id" = Uuid, Path, description = "ID del congelador")),
    request_body = UpdateFreezerDto,
    responses((status = 200, description = "Congelador actualizado", body = Freezer)),
    security(("bearer_auth" = []))
)]
async fn update_freezer(
    auth: AuthUser,
    State(state): State<CatalogState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateFreezerDto>,
) -> Result<Json<Freezer>, AppError> {
    auth.require_owner()?;
    Ok(Json(crud::update_freezer(&state.freezers, id, dto).await?))
}

#[utoipa::path(
    post, path = "/freezers/{id}/toggle", tag = "Catálogo - Congeladores",
    params(("id" = Uuid, Path, description = "ID del congelador")),
    responses((status = 200, description = "Estado del congelador cambiado", body = Freezer)),
    security(("bearer_auth" = []))
)]
async fn toggle_freezer(
    auth: AuthUser,
    State(state): State<CatalogState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Freezer>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(crud::toggle_freezer(&state.freezers, id).await?))
}
