use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::audit_log::domain::entities::{AuditLogEntry, CreateAuditLogDto};
use crate::modules::audit_log::domain::repositories::AuditLogRepository;
use crate::shared::errors::AppError;

/// Implementación PostgreSQL del repositorio de auditoría.
pub struct PgAuditLogRepository {
    pool: PgPool,
}

impl PgAuditLogRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuditLogRepository for PgAuditLogRepository {
    async fn create(&self, dto: CreateAuditLogDto) -> Result<AuditLogEntry, AppError> {
        let entry = sqlx::query_as::<_, AuditLogEntry>(
            r#"
            INSERT INTO audit_log (action, table_name, record_id, changes_before, changes_after, created_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(dto.action.as_str())
        .bind(&dto.table_name)
        .bind(dto.record_id)
        .bind(&dto.changes_before)
        .bind(&dto.changes_after)
        .bind(dto.created_by)
        .fetch_one(&self.pool)
        .await?;
        Ok(entry)
    }

    async fn create_with_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        dto: CreateAuditLogDto,
    ) -> Result<AuditLogEntry, AppError> {
        let entry = sqlx::query_as::<_, AuditLogEntry>(
            r#"
            INSERT INTO audit_log (action, table_name, record_id, changes_before, changes_after, created_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(dto.action.as_str())
        .bind(&dto.table_name)
        .bind(dto.record_id)
        .bind(&dto.changes_before)
        .bind(&dto.changes_after)
        .bind(dto.created_by)
        .fetch_one(&mut **tx)
        .await?;
        Ok(entry)
    }

    async fn find_by_record(
        &self,
        table_name: &str,
        record_id: Uuid,
    ) -> Result<Vec<AuditLogEntry>, AppError> {
        let entries = sqlx::query_as::<_, AuditLogEntry>(
            r#"
            SELECT * FROM audit_log
            WHERE table_name = $1 AND record_id = $2
            ORDER BY created_at DESC
            "#,
        )
        .bind(table_name)
        .bind(record_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(entries)
    }

    async fn find_by_user(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> Result<Vec<AuditLogEntry>, AppError> {
        let entries = sqlx::query_as::<_, AuditLogEntry>(
            r#"
            SELECT * FROM audit_log
            WHERE created_by = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(entries)
    }
}
