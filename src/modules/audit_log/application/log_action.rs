use std::sync::Arc;
use uuid::Uuid;

use crate::modules::audit_log::domain::entities::{AuditAction, AuditLogEntry, CreateAuditLogDto};
use crate::modules::audit_log::domain::repositories::AuditLogRepository;
use crate::shared::errors::AppError;

/// Caso de uso: Registrar una acción de auditoría.
pub async fn execute(
    repo: &Arc<dyn AuditLogRepository>,
    action: AuditAction,
    table_name: &str,
    record_id: Uuid,
    before: Option<serde_json::Value>,
    after: Option<serde_json::Value>,
    created_by: Uuid,
) -> Result<AuditLogEntry, AppError> {
    let dto = CreateAuditLogDto {
        action,
        table_name: table_name.to_string(),
        record_id,
        changes_before: before,
        changes_after: after,
        created_by,
    };
    repo.create(dto).await
}

/// Caso de uso: Registrar auditoría dentro de una transacción.
pub async fn execute_with_tx(
    repo: &Arc<dyn AuditLogRepository>,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    action: AuditAction,
    table_name: &str,
    record_id: Uuid,
    before: Option<serde_json::Value>,
    after: Option<serde_json::Value>,
    created_by: Uuid,
) -> Result<AuditLogEntry, AppError> {
    let dto = CreateAuditLogDto {
        action,
        table_name: table_name.to_string(),
        record_id,
        changes_before: before,
        changes_after: after,
        created_by,
    };
    repo.create_with_tx(tx, dto).await
}
