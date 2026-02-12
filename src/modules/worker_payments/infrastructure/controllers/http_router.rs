use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::modules::worker_payments::application::pay_worker;
use crate::modules::worker_payments::domain::entities::*;
use crate::modules::worker_payments::domain::repositories::WorkerPaymentRepository;
use crate::shared::auth::{AppState, AuthUser};
use crate::shared::errors::AppError;

#[derive(OpenApi)]
#[openapi(
    paths(list_by_worker, get_by_trip, create_payment),
    components(schemas(
        crate::modules::worker_payments::domain::entities::WorkerPayment,
        crate::modules::worker_payments::domain::entities::CreatePaymentDto,
    ))
)]
pub struct PaymentsApiDoc;

#[derive(Clone)]
pub struct PaymentsState {
    pub app: AppState,
    pub repo: Arc<dyn WorkerPaymentRepository>,
}

impl axum::extract::FromRef<PaymentsState> for AppState {
    fn from_ref(s: &PaymentsState) -> AppState {
        s.app.clone()
    }
}

pub fn router(app: AppState, repo: Arc<dyn WorkerPaymentRepository>) -> Router {
    let state = PaymentsState { app, repo };
    Router::new()
        .route("/worker/{worker_id}", get(list_by_worker))
        .route("/trip/{trip_id}", get(get_by_trip))
        .route("/", post(create_payment))
        .with_state(state)
}

#[utoipa::path(
    get, path = "/worker/{worker_id}", tag = "Pagos a Trabajadores",
    params(("worker_id" = Uuid, Path, description = "ID del trabajador")),
    responses((status = 200, description = "Pagos del trabajador", body = Vec<WorkerPayment>)),
    security(("bearer_auth" = []))
)]
async fn list_by_worker(
    State(state): State<PaymentsState>,
    auth: AuthUser,
    Path(worker_id): Path<Uuid>,
) -> Result<Json<Vec<WorkerPayment>>, AppError> {
    auth.require_role(crate::shared::auth::Role::Admin)?;
    let payments = pay_worker::list_by_worker(state.repo.as_ref(), worker_id).await?;
    Ok(Json(payments))
}

#[utoipa::path(
    get, path = "/trip/{trip_id}", tag = "Pagos a Trabajadores",
    params(("trip_id" = Uuid, Path, description = "ID del viaje")),
    responses(
        (status = 200, description = "Pago del viaje", body = WorkerPayment),
        (status = 404, description = "No encontrado")
    ),
    security(("bearer_auth" = []))
)]
async fn get_by_trip(
    State(state): State<PaymentsState>,
    auth: AuthUser,
    Path(trip_id): Path<Uuid>,
) -> Result<Json<WorkerPayment>, AppError> {
    auth.require_role(crate::shared::auth::Role::Admin)?;
    let payment = pay_worker::get_by_trip(state.repo.as_ref(), trip_id).await?;
    Ok(Json(payment))
}

#[utoipa::path(
    post, path = "/", tag = "Pagos a Trabajadores",
    request_body = CreatePaymentDto,
    responses((status = 200, description = "Pago creado", body = WorkerPayment)),
    security(("bearer_auth" = []))
)]
async fn create_payment(
    State(state): State<PaymentsState>,
    auth: AuthUser,
    Json(dto): Json<CreatePaymentDto>,
) -> Result<Json<WorkerPayment>, AppError> {
    auth.require_role(crate::shared::auth::Role::Admin)?;
    let payment =
        pay_worker::create_payment(state.repo.as_ref(), dto.trip_id, auth.user_id()).await?;
    Ok(Json(payment))
}
