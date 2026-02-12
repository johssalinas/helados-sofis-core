use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::modules::local_sales::application::manage_local_sales;
use crate::modules::local_sales::domain::entities::*;
use crate::modules::local_sales::domain::repositories::LocalSaleRepository;
use crate::shared::auth::{AppState, AuthUser, Role};
use crate::shared::errors::AppError;

#[derive(OpenApi)]
#[openapi(
    paths(list_sales, create_sale, get_sale, todays_sales),
    components(schemas(
        crate::modules::local_sales::domain::entities::LocalSale,
        crate::modules::local_sales::domain::entities::LocalSaleItem,
        crate::modules::local_sales::domain::entities::CreateLocalSaleDto,
        crate::modules::local_sales::domain::entities::LocalSaleItemDto,
        crate::modules::local_sales::domain::entities::LocalSaleWithItems,
    ))
)]
pub struct LocalSalesApiDoc;

#[derive(Clone)]
pub struct LocalSalesState {
    pub app: AppState,
    pub repo: Arc<dyn LocalSaleRepository>,
}

impl axum::extract::FromRef<LocalSalesState> for AppState {
    fn from_ref(s: &LocalSalesState) -> AppState {
        s.app.clone()
    }
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct LimitQuery {
    pub limit: Option<i64>,
}

pub fn router(app: AppState, repo: Arc<dyn LocalSaleRepository>) -> Router {
    let state = LocalSalesState { app, repo };
    Router::new()
        .route("/", get(list_sales).post(create_sale))
        .route("/today", get(todays_sales))
        .route("/{id}", get(get_sale))
        .with_state(state)
}

#[utoipa::path(
    get, path = "/", tag = "Ventas Locales",
    params(LimitQuery),
    responses((status = 200, description = "Lista de ventas locales", body = Vec<LocalSale>)),
    security(("bearer_auth" = []))
)]
async fn list_sales(
    State(state): State<LocalSalesState>,
    auth: AuthUser,
    Query(q): Query<LimitQuery>,
) -> Result<Json<Vec<LocalSale>>, AppError> {
    auth.require_role(Role::Admin)?;
    let limit = q.limit.unwrap_or(50);
    let sales = manage_local_sales::list_sales(state.repo.as_ref(), limit).await?;
    Ok(Json(sales))
}

#[utoipa::path(
    get, path = "/{id}", tag = "Ventas Locales",
    params(("id" = Uuid, Path, description = "ID de la venta")),
    responses(
        (status = 200, description = "Venta con items", body = LocalSaleWithItems),
        (status = 404, description = "No encontrada")
    ),
    security(("bearer_auth" = []))
)]
async fn get_sale(
    State(state): State<LocalSalesState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<LocalSaleWithItems>, AppError> {
    auth.require_role(Role::Admin)?;
    let sale = manage_local_sales::get_sale(state.repo.as_ref(), id).await?;
    Ok(Json(sale))
}

#[utoipa::path(
    get, path = "/today", tag = "Ventas Locales",
    responses((status = 200, description = "Ventas de hoy", body = Vec<LocalSale>)),
    security(("bearer_auth" = []))
)]
async fn todays_sales(
    State(state): State<LocalSalesState>,
    auth: AuthUser,
) -> Result<Json<Vec<LocalSale>>, AppError> {
    auth.require_role(Role::Admin)?;
    let sales = manage_local_sales::todays_sales(state.repo.as_ref()).await?;
    Ok(Json(sales))
}

#[utoipa::path(
    post, path = "/", tag = "Ventas Locales",
    request_body = CreateLocalSaleDto,
    responses((status = 200, description = "Venta creada", body = LocalSale)),
    security(("bearer_auth" = []))
)]
async fn create_sale(
    State(state): State<LocalSalesState>,
    auth: AuthUser,
    Json(dto): Json<CreateLocalSaleDto>,
) -> Result<Json<LocalSale>, AppError> {
    auth.require_role(Role::Admin)?;
    let sale = manage_local_sales::create_sale(state.repo.as_ref(), &dto, auth.user_id()).await?;
    Ok(Json(sale))
}
