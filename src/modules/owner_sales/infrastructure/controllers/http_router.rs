use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::shared::auth::{AppState, AuthUser};
use crate::shared::errors::AppError;
use crate::modules::owner_sales::application::manage_owner_sales;
use crate::modules::owner_sales::domain::entities::*;
use crate::modules::owner_sales::domain::repositories::OwnerSaleRepository;

#[derive(Clone)]
pub struct OwnerSalesState {
    pub app: AppState,
    pub repo: Arc<dyn OwnerSaleRepository>,
}

impl axum::extract::FromRef<OwnerSalesState> for AppState {
    fn from_ref(s: &OwnerSalesState) -> AppState { s.app.clone() }
}

#[derive(Debug, Deserialize)]
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

async fn get_sale(
    State(state): State<OwnerSalesState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<OwnerSaleWithItems>, AppError> {
    auth.require_owner()?;
    let sale = manage_owner_sales::get_sale(state.repo.as_ref(), id).await?;
    Ok(Json(sale))
}

async fn create_sale(
    State(state): State<OwnerSalesState>,
    auth: AuthUser,
    Json(dto): Json<CreateOwnerSaleDto>,
) -> Result<Json<OwnerSale>, AppError> {
    auth.require_owner()?;
    let sale = manage_owner_sales::create_sale(state.repo.as_ref(), &dto, auth.user_id()).await?;
    Ok(Json(sale))
}

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
