use std::sync::Arc;

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use uuid::Uuid;

use crate::shared::auth::{AppState, AuthUser, Role};
use crate::shared::errors::AppError;
use crate::modules::purchases::application::manage_purchases;
use crate::modules::purchases::domain::entities::*;
use crate::modules::purchases::domain::repositories::PurchaseRepository;

#[derive(Clone)]
pub struct PurchasesState {
    pub app: AppState,
    pub repo: Arc<dyn PurchaseRepository>,
}

impl axum::extract::FromRef<PurchasesState> for AppState {
    fn from_ref(s: &PurchasesState) -> AppState { s.app.clone() }
}

pub fn router(app_state: AppState, repo: Arc<dyn PurchaseRepository>) -> Router {
    let state = PurchasesState {
        app: app_state,
        repo,
    };

    Router::new()
        .route("/", get(list_handler).post(create_handler))
        .route("/{id}", get(get_handler))
        .with_state(state)
}

async fn list_handler(
    auth: AuthUser,
    State(state): State<PurchasesState>,
) -> Result<Json<Vec<Purchase>>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(manage_purchases::list_purchases(&state.repo).await?))
}

async fn get_handler(
    auth: AuthUser,
    State(state): State<PurchasesState>,
    Path(id): Path<Uuid>,
) -> Result<Json<PurchaseWithItems>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(manage_purchases::get_purchase(&state.repo, id).await?))
}

async fn create_handler(
    auth: AuthUser,
    State(state): State<PurchasesState>,
    Json(dto): Json<CreatePurchaseDto>,
) -> Result<Json<PurchaseWithItems>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(
        manage_purchases::create_purchase(&state.repo, dto, auth.user_id()).await?,
    ))
}
