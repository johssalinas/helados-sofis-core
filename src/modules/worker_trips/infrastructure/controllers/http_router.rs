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
use crate::modules::worker_trips::application::manage_trips;
use crate::modules::worker_trips::domain::entities::*;
use crate::modules::worker_trips::domain::repositories::WorkerTripRepository;

#[derive(Clone)]
pub struct TripsState {
    pub app: AppState,
    pub repo: Arc<dyn WorkerTripRepository>,
}

impl axum::extract::FromRef<TripsState> for AppState {
    fn from_ref(s: &TripsState) -> AppState { s.app.clone() }
}

#[derive(Debug, Deserialize)]
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

async fn list_active(
    State(state): State<TripsState>,
    auth: AuthUser,
) -> Result<Json<Vec<WorkerTrip>>, AppError> {
    auth.require_role(crate::shared::auth::Role::Admin)?;
    let trips = manage_trips::list_active(state.repo.as_ref()).await?;
    Ok(Json(trips))
}

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

async fn get_trip(
    State(state): State<TripsState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<TripWithItems>, AppError> {
    auth.require_role(crate::shared::auth::Role::Admin)?;
    let trip = manage_trips::get_trip(state.repo.as_ref(), id).await?;
    Ok(Json(trip))
}

async fn create_trip(
    State(state): State<TripsState>,
    auth: AuthUser,
    Json(dto): Json<CreateTripDto>,
) -> Result<Json<WorkerTrip>, AppError> {
    auth.require_role(crate::shared::auth::Role::Admin)?;
    let trip = manage_trips::create_trip(state.repo.as_ref(), &dto, auth.user_id()).await?;
    Ok(Json(trip))
}

async fn complete_trip(
    State(state): State<TripsState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(dto): Json<CompleteTripDto>,
) -> Result<Json<WorkerTrip>, AppError> {
    auth.require_role(crate::shared::auth::Role::Admin)?;
    let trip =
        manage_trips::complete_trip(state.repo.as_ref(), id, &dto, auth.user_id()).await?;
    Ok(Json(trip))
}

async fn todays_returned(
    State(state): State<TripsState>,
    auth: AuthUser,
) -> Result<Json<Vec<WorkerTrip>>, AppError> {
    auth.require_role(crate::shared::auth::Role::Admin)?;
    let trips = manage_trips::todays_returned(state.repo.as_ref()).await?;
    Ok(Json(trips))
}
