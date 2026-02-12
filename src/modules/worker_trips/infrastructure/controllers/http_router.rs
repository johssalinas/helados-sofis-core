use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::modules::worker_trips::application::manage_trips;
use crate::modules::worker_trips::domain::entities::*;
use crate::modules::worker_trips::domain::repositories::WorkerTripRepository;
use crate::shared::auth::{AppState, AuthUser};
use crate::shared::errors::AppError;

#[derive(OpenApi)]
#[openapi(
    paths(
        list_active,
        list_by_worker,
        todays_returned,
        create_trip,
        get_trip,
        complete_trip
    ),
    components(schemas(
        crate::modules::worker_trips::domain::entities::WorkerTrip,
        crate::modules::worker_trips::domain::entities::LoadedItem,
        crate::modules::worker_trips::domain::entities::ReturnedItem,
        crate::modules::worker_trips::domain::entities::CreateTripDto,
        crate::modules::worker_trips::domain::entities::LoadedItemDto,
        crate::modules::worker_trips::domain::entities::CompleteTripDto,
        crate::modules::worker_trips::domain::entities::ReturnedItemDto,
        crate::modules::worker_trips::domain::entities::TripWithItems,
    ))
)]
pub struct TripsApiDoc;

#[derive(Clone)]
pub struct TripsState {
    pub app: AppState,
    pub repo: Arc<dyn WorkerTripRepository>,
}

impl axum::extract::FromRef<TripsState> for AppState {
    fn from_ref(s: &TripsState) -> AppState {
        s.app.clone()
    }
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct WorkerQuery {
    pub limit: Option<i64>,
}

pub fn router(app: AppState, repo: Arc<dyn WorkerTripRepository>) -> Router {
    let state = TripsState { app, repo };
    Router::new()
        .route("/active", get(list_active))
        .route("/worker/{worker_id}", get(list_by_worker))
        .route("/today", get(todays_returned))
        .route("/", post(create_trip))
        .route("/{id}", get(get_trip))
        .route("/{id}/complete", post(complete_trip))
        .with_state(state)
}

#[utoipa::path(
    get, path = "/active", tag = "Viajes de Trabajadores",
    responses((status = 200, description = "Viajes activos", body = Vec<WorkerTrip>)),
    security(("bearer_auth" = []))
)]
async fn list_active(
    State(state): State<TripsState>,
    auth: AuthUser,
) -> Result<Json<Vec<WorkerTrip>>, AppError> {
    auth.require_role(crate::shared::auth::Role::Admin)?;
    let trips = manage_trips::list_active(state.repo.as_ref()).await?;
    Ok(Json(trips))
}

#[utoipa::path(
    get, path = "/worker/{worker_id}", tag = "Viajes de Trabajadores",
    params(
        ("worker_id" = Uuid, Path, description = "ID del trabajador"),
        WorkerQuery,
    ),
    responses((status = 200, description = "Viajes del trabajador", body = Vec<WorkerTrip>)),
    security(("bearer_auth" = []))
)]
async fn list_by_worker(
    State(state): State<TripsState>,
    auth: AuthUser,
    Path(worker_id): Path<Uuid>,
    Query(q): Query<WorkerQuery>,
) -> Result<Json<Vec<WorkerTrip>>, AppError> {
    auth.require_role(crate::shared::auth::Role::Admin)?;
    let limit = q.limit.unwrap_or(50);
    let trips = manage_trips::list_by_worker(state.repo.as_ref(), worker_id, limit).await?;
    Ok(Json(trips))
}

#[utoipa::path(
    get, path = "/{id}", tag = "Viajes de Trabajadores",
    params(("id" = Uuid, Path, description = "ID del viaje")),
    responses(
        (status = 200, description = "Viaje con items", body = TripWithItems),
        (status = 404, description = "No encontrado")
    ),
    security(("bearer_auth" = []))
)]
async fn get_trip(
    State(state): State<TripsState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<TripWithItems>, AppError> {
    auth.require_role(crate::shared::auth::Role::Admin)?;
    let trip = manage_trips::get_trip(state.repo.as_ref(), id).await?;
    Ok(Json(trip))
}

#[utoipa::path(
    post, path = "/", tag = "Viajes de Trabajadores",
    request_body = CreateTripDto,
    responses((status = 200, description = "Viaje creado", body = WorkerTrip)),
    security(("bearer_auth" = []))
)]
async fn create_trip(
    State(state): State<TripsState>,
    auth: AuthUser,
    Json(dto): Json<CreateTripDto>,
) -> Result<Json<WorkerTrip>, AppError> {
    auth.require_role(crate::shared::auth::Role::Admin)?;
    let trip = manage_trips::create_trip(state.repo.as_ref(), &dto, auth.user_id()).await?;
    Ok(Json(trip))
}

#[utoipa::path(
    post, path = "/{id}/complete", tag = "Viajes de Trabajadores",
    params(("id" = Uuid, Path, description = "ID del viaje")),
    request_body = CompleteTripDto,
    responses((status = 200, description = "Viaje completado", body = WorkerTrip)),
    security(("bearer_auth" = []))
)]
async fn complete_trip(
    State(state): State<TripsState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(dto): Json<CompleteTripDto>,
) -> Result<Json<WorkerTrip>, AppError> {
    auth.require_role(crate::shared::auth::Role::Admin)?;
    let trip = manage_trips::complete_trip(state.repo.as_ref(), id, &dto, auth.user_id()).await?;
    Ok(Json(trip))
}

#[utoipa::path(
    get, path = "/today", tag = "Viajes de Trabajadores",
    responses((status = 200, description = "Viajes retornados hoy", body = Vec<WorkerTrip>)),
    security(("bearer_auth" = []))
)]
async fn todays_returned(
    State(state): State<TripsState>,
    auth: AuthUser,
) -> Result<Json<Vec<WorkerTrip>>, AppError> {
    auth.require_role(crate::shared::auth::Role::Admin)?;
    let trips = manage_trips::todays_returned(state.repo.as_ref()).await?;
    Ok(Json(trips))
}
