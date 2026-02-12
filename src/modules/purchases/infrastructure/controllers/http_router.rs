use std::sync::Arc;

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use utoipa::OpenApi;
use uuid::Uuid;

use crate::modules::purchases::application::manage_purchases;
use crate::modules::purchases::domain::entities::*;
use crate::modules::purchases::domain::repositories::PurchaseRepository;
use crate::shared::auth::{AppState, AuthUser, Role};
use crate::shared::errors::AppError;

#[derive(OpenApi)]
#[openapi(
    paths(list_handler, get_handler, create_handler),
    components(schemas(
        crate::modules::purchases::domain::entities::Purchase,
        crate::modules::purchases::domain::entities::PurchaseItem,
        crate::modules::purchases::domain::entities::CreatePurchaseDto,
        crate::modules::purchases::domain::entities::CreatePurchaseItemDto,
        crate::modules::purchases::domain::entities::PurchaseWithItems,
    ))
)]
pub struct PurchasesApiDoc;

#[derive(Clone)]
pub struct PurchasesState {
    pub app: AppState,
    pub repo: Arc<dyn PurchaseRepository>,
}

impl axum::extract::FromRef<PurchasesState> for AppState {
    fn from_ref(s: &PurchasesState) -> AppState {
        s.app.clone()
    }
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

#[utoipa::path(
    get, path = "/", tag = "Compras",
    responses((status = 200, description = "Lista de compras", body = Vec<Purchase>)),
    security(("bearer_auth" = []))
)]
async fn list_handler(
    auth: AuthUser,
    State(state): State<PurchasesState>,
) -> Result<Json<Vec<Purchase>>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(manage_purchases::list_purchases(&state.repo).await?))
}

#[utoipa::path(
    get, path = "/{id}", tag = "Compras",
    params(("id" = Uuid, Path, description = "ID de la compra")),
    responses(
        (status = 200, description = "Compra con items", body = PurchaseWithItems),
        (status = 404, description = "No encontrada")
    ),
    security(("bearer_auth" = []))
)]
async fn get_handler(
    auth: AuthUser,
    State(state): State<PurchasesState>,
    Path(id): Path<Uuid>,
) -> Result<Json<PurchaseWithItems>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(manage_purchases::get_purchase(&state.repo, id).await?))
}

#[utoipa::path(
    post, path = "/", tag = "Compras",
    request_body = CreatePurchaseDto,
    responses((status = 200, description = "Compra creada", body = PurchaseWithItems)),
    security(("bearer_auth" = []))
)]
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
