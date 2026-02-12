use std::sync::Arc;

use axum::{
    extract::{Path, State},
    routing::{get, put},
    Json, Router,
};
use utoipa::OpenApi;
use uuid::Uuid;

use crate::modules::inventory::application::manage_inventory;
use crate::modules::inventory::domain::entities::{AddStockDto, InventoryItem, UpdateAlertDto};
use crate::modules::inventory::domain::repositories::InventoryRepository;
use crate::shared::auth::{AppState, AuthUser, Role};
use crate::shared::errors::AppError;

#[derive(OpenApi)]
#[openapi(
    paths(
        list_all_handler,
        add_stock_handler,
        sellable_handler,
        low_stock_handler,
        by_freezer_handler,
        worker_deformed_handler,
        update_alert_handler,
    ),
    components(schemas(
        crate::modules::inventory::domain::entities::InventoryItem,
        crate::modules::inventory::domain::entities::AddStockDto,
        crate::modules::inventory::domain::entities::UpdateAlertDto,
    ))
)]
pub struct InventoryApiDoc;

#[derive(Clone)]
pub struct InventoryState {
    pub app: AppState,
    pub repo: Arc<dyn InventoryRepository>,
}

impl axum::extract::FromRef<InventoryState> for AppState {
    fn from_ref(s: &InventoryState) -> AppState {
        s.app.clone()
    }
}

pub fn router(app_state: AppState, repo: Arc<dyn InventoryRepository>) -> Router {
    let state = InventoryState {
        app: app_state,
        repo,
    };

    Router::new()
        .route("/", get(list_all_handler).post(add_stock_handler))
        .route("/sellable", get(sellable_handler))
        .route("/low-stock", get(low_stock_handler))
        .route("/freezer/{freezer_id}", get(by_freezer_handler))
        .route("/worker/{worker_id}/deformed", get(worker_deformed_handler))
        .route("/{id}/alert", put(update_alert_handler))
        .with_state(state)
}

#[utoipa::path(
    get, path = "/", tag = "Inventario",
    responses((status = 200, description = "Todo el inventario", body = Vec<InventoryItem>)),
    security(("bearer_auth" = []))
)]
async fn list_all_handler(
    auth: AuthUser,
    State(state): State<InventoryState>,
) -> Result<Json<Vec<InventoryItem>>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(manage_inventory::list_all(&state.repo).await?))
}

#[utoipa::path(
    get, path = "/freezer/{freezer_id}", tag = "Inventario",
    params(("freezer_id" = Uuid, Path, description = "ID del congelador")),
    responses((status = 200, description = "Inventario del congelador", body = Vec<InventoryItem>)),
    security(("bearer_auth" = []))
)]
async fn by_freezer_handler(
    auth: AuthUser,
    State(state): State<InventoryState>,
    Path(freezer_id): Path<Uuid>,
) -> Result<Json<Vec<InventoryItem>>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(
        manage_inventory::list_by_freezer(&state.repo, freezer_id).await?,
    ))
}

#[utoipa::path(
    get, path = "/sellable", tag = "Inventario",
    responses((status = 200, description = "Inventario vendible", body = Vec<InventoryItem>)),
    security(("bearer_auth" = []))
)]
async fn sellable_handler(
    auth: AuthUser,
    State(state): State<InventoryState>,
) -> Result<Json<Vec<InventoryItem>>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(manage_inventory::list_sellable(&state.repo).await?))
}

#[utoipa::path(
    get, path = "/low-stock", tag = "Inventario",
    responses((status = 200, description = "Items con stock bajo", body = Vec<InventoryItem>)),
    security(("bearer_auth" = []))
)]
async fn low_stock_handler(
    auth: AuthUser,
    State(state): State<InventoryState>,
) -> Result<Json<Vec<InventoryItem>>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(manage_inventory::list_low_stock(&state.repo).await?))
}

#[utoipa::path(
    get, path = "/worker/{worker_id}/deformed", tag = "Inventario",
    params(("worker_id" = Uuid, Path, description = "ID del trabajador")),
    responses((status = 200, description = "Deformados del trabajador", body = Vec<InventoryItem>)),
    security(("bearer_auth" = []))
)]
async fn worker_deformed_handler(
    auth: AuthUser,
    State(state): State<InventoryState>,
    Path(worker_id): Path<Uuid>,
) -> Result<Json<Vec<InventoryItem>>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(
        manage_inventory::list_worker_deformed(&state.repo, worker_id).await?,
    ))
}

#[utoipa::path(
    post, path = "/", tag = "Inventario",
    request_body = AddStockDto,
    responses((status = 200, description = "Stock añadido", body = InventoryItem)),
    security(("bearer_auth" = []))
)]
async fn add_stock_handler(
    auth: AuthUser,
    State(state): State<InventoryState>,
    Json(dto): Json<AddStockDto>,
) -> Result<Json<InventoryItem>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(
        manage_inventory::add_stock(&state.repo, dto, auth.user_id()).await?,
    ))
}

#[utoipa::path(
    put, path = "/{id}/alert", tag = "Inventario",
    params(("id" = Uuid, Path, description = "ID del item de inventario")),
    request_body = UpdateAlertDto,
    responses((status = 200, description = "Alerta actualizada", body = InventoryItem)),
    security(("bearer_auth" = []))
)]
async fn update_alert_handler(
    auth: AuthUser,
    State(state): State<InventoryState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateAlertDto>,
) -> Result<Json<InventoryItem>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(
        manage_inventory::update_alert(&state.repo, id, dto.min_stock_alert).await?,
    ))
}
