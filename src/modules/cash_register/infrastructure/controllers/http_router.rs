use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;

use crate::shared::auth::{AppState, AuthUser, Role};
use crate::shared::errors::AppError;
use crate::modules::cash_register::application::manage_cash;
use crate::modules::cash_register::domain::entities::*;
use crate::modules::cash_register::domain::repositories::CashRegisterRepository;

#[derive(Clone)]
pub struct CashState {
    pub app: AppState,
    pub repo: Arc<dyn CashRegisterRepository>,
}

impl axum::extract::FromRef<CashState> for AppState {
    fn from_ref(s: &CashState) -> AppState { s.app.clone() }
}

pub fn router(app: AppState, repo: Arc<dyn CashRegisterRepository>) -> Router {
    let state = CashState { app, repo };
    Router::new()
        .route("/balance", get(get_balance))
        .route("/today", get(todays_transactions))
        .route("/range", get(transactions_by_range))
        .route("/expense", post(add_expense))
        .route("/withdrawal", post(add_withdrawal))
        .with_state(state)
}

async fn get_balance(
    State(state): State<CashState>,
    auth: AuthUser,
) -> Result<Json<BalanceInfo>, AppError> {
    auth.require_role(Role::Admin)?;
    let info = manage_cash::get_balance(state.repo.as_ref()).await?;
    Ok(Json(info))
}

async fn todays_transactions(
    State(state): State<CashState>,
    auth: AuthUser,
) -> Result<Json<Vec<CashTransaction>>, AppError> {
    auth.require_role(Role::Admin)?;
    let txs = manage_cash::todays_transactions(state.repo.as_ref()).await?;
    Ok(Json(txs))
}

async fn transactions_by_range(
    State(state): State<CashState>,
    auth: AuthUser,
    Query(q): Query<DateRangeQuery>,
) -> Result<Json<Vec<CashTransaction>>, AppError> {
    auth.require_role(Role::Admin)?;
    let from = q.from.unwrap_or_else(|| {
        chrono::Utc::now() - chrono::Duration::days(30)
    });
    let to = q.to.unwrap_or_else(chrono::Utc::now);
    let txs = manage_cash::transactions_by_range(state.repo.as_ref(), from, to).await?;
    Ok(Json(txs))
}

async fn add_expense(
    State(state): State<CashState>,
    auth: AuthUser,
    Json(dto): Json<CreateExpenseDto>,
) -> Result<Json<CashTransaction>, AppError> {
    auth.require_owner()?;
    let tx = manage_cash::add_expense(state.repo.as_ref(), &dto, auth.user_id()).await?;
    Ok(Json(tx))
}

async fn add_withdrawal(
    State(state): State<CashState>,
    auth: AuthUser,
    Json(dto): Json<CreateWithdrawalDto>,
) -> Result<Json<CashTransaction>, AppError> {
    auth.require_owner()?;
    let tx = manage_cash::add_withdrawal(state.repo.as_ref(), &dto, auth.user_id()).await?;
    Ok(Json(tx))
}
