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
use crate::modules::local_sales::application::manage_local_sales;
use crate::modules::local_sales::domain::entities::*;
use crate::modules::local_sales::domain::repositories::LocalSaleRepository;

#[derive(Clone)]
pub struct LocalSalesState {
    pub app: AppState,
    pub repo: Arc<dyn LocalSaleRepository>,
}

impl axum::extract::FromRef<LocalSalesState> for AppState {
    fn from_ref(s: &LocalSalesState) -> AppState { s.app.clone() }
}

#[derive(Debug, Deserialize)]
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

async fn get_sale(
    State(state): State<LocalSalesState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<LocalSaleWithItems>, AppError> {
    auth.require_role(Role::Admin)?;
    let sale = manage_local_sales::get_sale(state.repo.as_ref(), id).await?;
    Ok(Json(sale))
}

async fn todays_sales(
    State(state): State<LocalSalesState>,
    auth: AuthUser,
) -> Result<Json<Vec<LocalSale>>, AppError> {
    auth.require_role(Role::Admin)?;
    let sales = manage_local_sales::todays_sales(state.repo.as_ref()).await?;
    Ok(Json(sales))
}

async fn create_sale(
    State(state): State<LocalSalesState>,
    auth: AuthUser,
    Json(dto): Json<CreateLocalSaleDto>,
) -> Result<Json<LocalSale>, AppError> {
    auth.require_role(Role::Admin)?;
    let sale =
        manage_local_sales::create_sale(state.repo.as_ref(), &dto, auth.user_id()).await?;
    Ok(Json(sale))
}
