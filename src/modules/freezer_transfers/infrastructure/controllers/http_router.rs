use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::shared::auth::{AppState, AuthUser, Role};
use crate::shared::errors::AppError;
use crate::modules::freezer_transfers::application::manage_transfers;
use crate::modules::freezer_transfers::domain::entities::*;
use crate::modules::freezer_transfers::domain::repositories::FreezerTransferRepository;

#[derive(Clone)]
pub struct TransfersState {
    pub app: AppState,
    pub repo: Arc<dyn FreezerTransferRepository>,
}

impl axum::extract::FromRef<TransfersState> for AppState {
    fn from_ref(s: &TransfersState) -> AppState { s.app.clone() }
}

#[derive(Debug, Deserialize)]
pub struct LimitQuery {
    pub limit: Option<i64>,
}

pub fn router(app: AppState, repo: Arc<dyn FreezerTransferRepository>) -> Router {
    let state = TransfersState { app, repo };
    Router::new()
        .route("/", get(list_transfers).post(create_transfer))
        .route("/{id}", get(get_transfer))
        .route("/freezer/{freezer_id}", get(list_by_freezer))
        .with_state(state)
}

async fn list_transfers(
    State(state): State<TransfersState>,
    auth: AuthUser,
    Query(q): Query<LimitQuery>,
) -> Result<Json<Vec<FreezerTransfer>>, AppError> {
    auth.require_role(Role::Admin)?;
    let limit = q.limit.unwrap_or(50);
    let transfers = manage_transfers::list_transfers(state.repo.as_ref(), limit).await?;
    Ok(Json(transfers))
}

async fn get_transfer(
    State(state): State<TransfersState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<TransferWithItems>, AppError> {
    auth.require_role(Role::Admin)?;
    let transfer = manage_transfers::get_transfer(state.repo.as_ref(), id).await?;
    Ok(Json(transfer))
}

async fn list_by_freezer(
    State(state): State<TransfersState>,
    auth: AuthUser,
    Path(freezer_id): Path<Uuid>,
) -> Result<Json<Vec<FreezerTransfer>>, AppError> {
    auth.require_role(Role::Admin)?;
    let transfers =
        manage_transfers::list_by_freezer(state.repo.as_ref(), freezer_id).await?;
    Ok(Json(transfers))
}

async fn create_transfer(
    State(state): State<TransfersState>,
    auth: AuthUser,
    Json(dto): Json<CreateTransferDto>,
) -> Result<Json<FreezerTransfer>, AppError> {
    auth.require_role(Role::Admin)?;
    let transfer =
        manage_transfers::create_transfer(state.repo.as_ref(), &dto, auth.user_id()).await?;
    Ok(Json(transfer))
}
