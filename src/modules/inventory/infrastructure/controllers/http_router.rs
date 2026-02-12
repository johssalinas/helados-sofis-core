use std::sync::Arc;

use axum::{
    extract::{Path, State},
    routing::{get, put},
    Json, Router,
};
use uuid::Uuid;

use crate::shared::auth::{AppState, AuthUser, Role};
use crate::shared::errors::AppError;
use crate::modules::inventory::application::manage_inventory;
use crate::modules::inventory::domain::entities::{AddStockDto, InventoryItem, UpdateAlertDto};
use crate::modules::inventory::domain::repositories::InventoryRepository;

#[derive(Clone)]
pub struct InventoryState {
    pub app: AppState,
    pub repo: Arc<dyn InventoryRepository>,
}

impl axum::extract::FromRef<InventoryState> for AppState {
    fn from_ref(s: &InventoryState) -> AppState { s.app.clone() }
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

async fn list_all_handler(
    auth: AuthUser,
    State(state): State<InventoryState>,
) -> Result<Json<Vec<InventoryItem>>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(manage_inventory::list_all(&state.repo).await?))
}

async fn by_freezer_handler(
    auth: AuthUser,
    State(state): State<InventoryState>,
    Path(freezer_id): Path<Uuid>,
) -> Result<Json<Vec<InventoryItem>>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(manage_inventory::list_by_freezer(&state.repo, freezer_id).await?))
}

async fn sellable_handler(
    auth: AuthUser,
    State(state): State<InventoryState>,
) -> Result<Json<Vec<InventoryItem>>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(manage_inventory::list_sellable(&state.repo).await?))
}

async fn low_stock_handler(
    auth: AuthUser,
    State(state): State<InventoryState>,
) -> Result<Json<Vec<InventoryItem>>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(manage_inventory::list_low_stock(&state.repo).await?))
}

async fn worker_deformed_handler(
    auth: AuthUser,
    State(state): State<InventoryState>,
    Path(worker_id): Path<Uuid>,
) -> Result<Json<Vec<InventoryItem>>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(manage_inventory::list_worker_deformed(&state.repo, worker_id).await?))
}

async fn add_stock_handler(
    auth: AuthUser,
    State(state): State<InventoryState>,
    Json(dto): Json<AddStockDto>,
) -> Result<Json<InventoryItem>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(manage_inventory::add_stock(&state.repo, dto, auth.user_id()).await?))
}

async fn update_alert_handler(
    auth: AuthUser,
    State(state): State<InventoryState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateAlertDto>,
) -> Result<Json<InventoryItem>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(manage_inventory::update_alert(&state.repo, id, dto.min_stock_alert).await?))
}
