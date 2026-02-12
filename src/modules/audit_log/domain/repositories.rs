use async_trait::async_trait;
use uuid::Uuid;

use super::entities::{AuditLogEntry, CreateAuditLogDto};
use crate::shared::errors::AppError;

/// Puerto de salida: contrato de persistencia para audit_log.
#[async_trait]
pub trait AuditLogRepository: Send + Sync {
    /// Registrar un evento de auditoría.
    async fn create(&self, dto: CreateAuditLogDto) -> Result<AuditLogEntry, AppError>;

    /// Registrar auditoría usando una transacción existente.
    async fn create_with_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        dto: CreateAuditLogDto,
    ) -> Result<AuditLogEntry, AppError>;

    /// Obtener registros de auditoría para una tabla/registro.
    async fn find_by_record(
        &self,
        table_name: &str,
        record_id: Uuid,
    ) -> Result<Vec<AuditLogEntry>, AppError>;

    /// Obtener registros de un usuario específico.
    async fn find_by_user(&self, user_id: Uuid, limit: i64)
        -> Result<Vec<AuditLogEntry>, AppError>;
}
