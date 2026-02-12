use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Acción registrada en el log de auditoría.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "VARCHAR")]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum AuditAction {
    Create,
    Update,
    Delete,
}

impl AuditAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditAction::Create => "create",
            AuditAction::Update => "update",
            AuditAction::Delete => "delete",
        }
    }
}

/// Registro de auditoría.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub action: String,
    pub table_name: String,
    pub record_id: Uuid,
    pub changes_before: Option<serde_json::Value>,
    pub changes_after: Option<serde_json::Value>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

/// DTO para crear un nuevo registro de auditoría.
pub struct CreateAuditLogDto {
    pub action: AuditAction,
    pub table_name: String,
    pub record_id: Uuid,
    pub changes_before: Option<serde_json::Value>,
    pub changes_after: Option<serde_json::Value>,
    pub created_by: Uuid,
}
