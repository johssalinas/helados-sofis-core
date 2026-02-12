use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::modules::owner_sales::application::manage_owner_sales;
use crate::modules::owner_sales::domain::entities::*;
use crate::modules::owner_sales::domain::repositories::OwnerSaleRepository;
use crate::shared::auth::{AppState, AuthUser};
use crate::shared::errors::AppError;

#[derive(OpenApi)]
#[openapi(
    paths(list_sales, get_sale, create_sale, complete_sale),
    components(schemas(
        crate::modules::owner_sales::domain::entities::OwnerSale,
        crate::modules::owner_sales::domain::entities::OwnerSaleLoadedItem,
        crate::modules::owner_sales::domain::entities::OwnerSaleReturnedItem,
        crate::modules::owner_sales::domain::entities::CreateOwnerSaleDto,
        crate::modules::owner_sales::domain::entities::OwnerLoadedItemDto,
        crate::modules::owner_sales::domain::entities::CompleteOwnerSaleDto,
        crate::modules::owner_sales::domain::entities::OwnerReturnedItemDto,
        crate::modules::owner_sales::domain::entities::OwnerSaleWithItems,
    ))
)]
pub struct OwnerSalesApiDoc;

#[derive(Clone)]
pub struct OwnerSalesState {
    pub app: AppState,
    pub repo: Arc<dyn OwnerSaleRepository>,
}

impl axum::extract::FromRef<OwnerSalesState> for AppState {
    fn from_ref(s: &OwnerSalesState) -> AppState {
        s.app.clone()
    }
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct LimitQuery {
    pub limit: Option<i64>,
}

pub fn router(app: AppState, repo: Arc<dyn OwnerSaleRepository>) -> Router {
    let state = OwnerSalesState { app, repo };
    Router::new()
        .route("/", get(list_sales).post(create_sale))
        .route("/{id}", get(get_sale))
        .route("/{id}/complete", post(complete_sale))
        .with_state(state)
}

#[utoipa::path(
    get, path = "/", tag = "Ventas del Dueño",
    params(LimitQuery),
    responses((status = 200, description = "Lista de ventas del dueño", body = Vec<OwnerSale>)),
    security(("bearer_auth" = []))
)]
async fn list_sales(
    State(state): State<OwnerSalesState>,
    auth: AuthUser,
    Query(q): Query<LimitQuery>,
) -> Result<Json<Vec<OwnerSale>>, AppError> {
    auth.require_owner()?;
    let limit = q.limit.unwrap_or(50);
    let sales = manage_owner_sales::list_sales(state.repo.as_ref(), limit).await?;
    Ok(Json(sales))
}

#[utoipa::path(
    get, path = "/{id}", tag = "Ventas del Dueño",
    params(("id" = Uuid, Path, description = "ID de la venta")),
    responses(
        (status = 200, description = "Venta con items", body = OwnerSaleWithItems),
        (status = 404, description = "No encontrada")
    ),
    security(("bearer_auth" = []))
)]
async fn get_sale(
    State(state): State<OwnerSalesState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<OwnerSaleWithItems>, AppError> {
    auth.require_owner()?;
    let sale = manage_owner_sales::get_sale(state.repo.as_ref(), id).await?;
    Ok(Json(sale))
}

#[utoipa::path(
    post, path = "/", tag = "Ventas del Dueño",
    request_body = CreateOwnerSaleDto,
    responses((status = 200, description = "Venta creada", body = OwnerSale)),
    security(("bearer_auth" = []))
)]
async fn create_sale(
    State(state): State<OwnerSalesState>,
    auth: AuthUser,
    Json(dto): Json<CreateOwnerSaleDto>,
) -> Result<Json<OwnerSale>, AppError> {
    auth.require_owner()?;
    let sale = manage_owner_sales::create_sale(state.repo.as_ref(), &dto, auth.user_id()).await?;
    Ok(Json(sale))
}

#[utoipa::path(
    post, path = "/{id}/complete", tag = "Ventas del Dueño",
    params(("id" = Uuid, Path, description = "ID de la venta")),
    request_body = CompleteOwnerSaleDto,
    responses((status = 200, description = "Venta completada", body = OwnerSale)),
    security(("bearer_auth" = []))
)]
async fn complete_sale(
    State(state): State<OwnerSalesState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(dto): Json<CompleteOwnerSaleDto>,
) -> Result<Json<OwnerSale>, AppError> {
    auth.require_owner()?;
    let sale =
        manage_owner_sales::complete_sale(state.repo.as_ref(), id, &dto, auth.user_id()).await?;
    Ok(Json(sale))
}
