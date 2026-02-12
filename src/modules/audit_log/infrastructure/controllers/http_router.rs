use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::modules::audit_log::domain::entities::AuditLogEntry;
use crate::modules::audit_log::domain::repositories::AuditLogRepository;
use crate::shared::auth::{AppState, AuthUser};
use crate::shared::errors::AppError;

#[derive(OpenApi)]
#[openapi(
    paths(list_handler, by_user_handler),
    components(schemas(crate::modules::audit_log::domain::entities::AuditLogEntry,))
)]
pub struct AuditApiDoc;

#[derive(Clone)]
pub struct AuditState {
    pub app: AppState,
    pub repo: Arc<dyn AuditLogRepository>,
}

impl axum::extract::FromRef<AuditState> for AppState {
    fn from_ref(s: &AuditState) -> AppState {
        s.app.clone()
    }
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct AuditQuery {
    pub table_name: Option<String>,
    pub record_id: Option<Uuid>,
    pub limit: Option<i64>,
}

pub fn router(app_state: AppState, repo: Arc<dyn AuditLogRepository>) -> Router {
    let state = AuditState {
        app: app_state,
        repo,
    };

    Router::new()
        .route("/", get(list_handler))
        .route("/user/{user_id}", get(by_user_handler))
        .with_state(state)
}

/// GET /audit — Buscar registros de auditoría por tabla y record_id.
#[utoipa::path(
    get,
    path = "/",
    tag = "Auditoría",
    params(AuditQuery),
    responses(
        (status = 200, description = "Registros de auditoría", body = Vec<AuditLogEntry>),
        (status = 400, description = "Faltan parámetros requeridos")
    ),
    security(("bearer_auth" = []))
)]
async fn list_handler(
    auth: AuthUser,
    State(state): State<AuditState>,
    Query(query): Query<AuditQuery>,
) -> Result<Json<Vec<AuditLogEntry>>, AppError> {
    auth.require_owner()?;

    match (query.table_name, query.record_id) {
        (Some(table), Some(record_id)) => {
            let entries = state.repo.find_by_record(&table, record_id).await?;
            Ok(Json(entries))
        }
        _ => Err(AppError::BadRequest(
            "Se requieren los parámetros table_name y record_id".into(),
        )),
    }
}

/// GET /audit/user/:user_id — Auditoría de un usuario específico.
#[utoipa::path(
    get,
    path = "/user/{user_id}",
    tag = "Auditoría",
    params(
        ("user_id" = Uuid, Path, description = "ID del usuario"),
        AuditQuery,
    ),
    responses(
        (status = 200, description = "Registros de auditoría del usuario", body = Vec<AuditLogEntry>)
    ),
    security(("bearer_auth" = []))
)]
async fn by_user_handler(
    auth: AuthUser,
    State(state): State<AuditState>,
    Path(user_id): Path<Uuid>,
    Query(query): Query<AuditQuery>,
) -> Result<Json<Vec<AuditLogEntry>>, AppError> {
    auth.require_owner()?;
    let limit = query.limit.unwrap_or(50);
    let entries = state.repo.find_by_user(user_id, limit).await?;
    Ok(Json(entries))
}
