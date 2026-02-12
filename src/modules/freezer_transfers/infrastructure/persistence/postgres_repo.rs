use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::freezer_transfers::domain::entities::*;
use crate::modules::freezer_transfers::domain::repositories::FreezerTransferRepository;
use crate::shared::errors::AppError;

pub struct PgFreezerTransferRepository {
    pool: PgPool,
}

impl PgFreezerTransferRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FreezerTransferRepository for PgFreezerTransferRepository {
    async fn find_all(&self, limit: i64) -> Result<Vec<FreezerTransfer>, AppError> {
        Ok(sqlx::query_as::<_, FreezerTransfer>(
            "SELECT * FROM freezer_transfers ORDER BY created_at DESC LIMIT $1",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?)
    }

    async fn find_by_id_with_items(&self, id: Uuid) -> Result<Option<TransferWithItems>, AppError> {
        let transfer =
            sqlx::query_as::<_, FreezerTransfer>("SELECT * FROM freezer_transfers WHERE id = $1")
                .bind(id)
                .fetch_optional(&self.pool)
                .await?;

        match transfer {
            Some(t) => {
                let items = sqlx::query_as::<_, FreezerTransferItem>(
                    "SELECT * FROM freezer_transfer_items WHERE transfer_id = $1",
                )
                .bind(id)
                .fetch_all(&self.pool)
                .await?;

                Ok(Some(TransferWithItems { transfer: t, items }))
            }
            None => Ok(None),
        }
    }

    async fn find_by_freezer(&self, freezer_id: Uuid) -> Result<Vec<FreezerTransfer>, AppError> {
        Ok(sqlx::query_as::<_, FreezerTransfer>(
            r#"
            SELECT * FROM freezer_transfers 
            WHERE from_freezer_id = $1 OR to_freezer_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(freezer_id)
        .fetch_all(&self.pool)
        .await?)
    }

    async fn create_transfer(
        &self,
        dto: &CreateTransferDto,
        created_by: Uuid,
    ) -> Result<FreezerTransfer, AppError> {
        let mut tx = self.pool.begin().await?;

        // 1. Crear transferencia
        let transfer = sqlx::query_as::<_, FreezerTransfer>(
            r#"
            INSERT INTO freezer_transfers (from_freezer_id, to_freezer_id, reason, created_by)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(dto.from_freezer_id)
        .bind(dto.to_freezer_id)
        .bind(&dto.reason)
        .bind(created_by)
        .fetch_one(&mut *tx)
        .await?;

        // 2. Procesar cada item
        for item in &dto.items {
            sqlx::query(
                r#"
                INSERT INTO freezer_transfer_items 
                (transfer_id, product_id, flavor_id, quantity)
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(transfer.id)
            .bind(item.product_id)
            .bind(item.flavor_id)
            .bind(item.quantity)
            .execute(&mut *tx)
            .await?;

            // Restar del congelador origen
            // Buscar el inventory_id exacto del congelador origen
            let source_inv = sqlx::query_scalar::<_, Uuid>(
                r#"
                SELECT id FROM inventory 
                WHERE freezer_id = $1 AND product_id = $2 AND flavor_id = $3 AND is_deformed = FALSE
                LIMIT 1
                "#,
            )
            .bind(dto.from_freezer_id)
            .bind(item.product_id)
            .bind(item.flavor_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "No se encontró inventario para producto {} sabor {} en congelador origen",
                    item.product_id, item.flavor_id
                ))
            })?;

            let rows = sqlx::query(
                "UPDATE inventory SET quantity = quantity - $1, last_updated = NOW(), updated_by = $2 WHERE id = $3 AND quantity >= $1",
            )
            .bind(item.quantity)
            .bind(created_by)
            .bind(source_inv)
            .execute(&mut *tx)
            .await?
            .rows_affected();

            if rows == 0 {
                return Err(AppError::InsufficientStock(source_inv));
            }

            // Sumar al congelador destino (UPSERT)
            // Necesitamos el provider_id del item origen
            let provider_id =
                sqlx::query_scalar::<_, Uuid>("SELECT provider_id FROM inventory WHERE id = $1")
                    .bind(source_inv)
                    .fetch_one(&mut *tx)
                    .await?;

            sqlx::query(
                r#"
                INSERT INTO inventory 
                (freezer_id, product_id, flavor_id, provider_id, quantity, is_deformed, min_stock_alert, updated_by)
                VALUES ($1, $2, $3, $4, $5, FALSE, 20, $6)
                ON CONFLICT (freezer_id, product_id, flavor_id, provider_id, is_deformed, assigned_worker_id)
                DO UPDATE SET 
                    quantity = inventory.quantity + EXCLUDED.quantity,
                    last_updated = NOW(),
                    updated_by = EXCLUDED.updated_by
                "#,
            )
            .bind(dto.to_freezer_id)
            .bind(item.product_id)
            .bind(item.flavor_id)
            .bind(provider_id)
            .bind(item.quantity)
            .bind(created_by)
            .execute(&mut *tx)
            .await?;
        }

        // 3. Auditoría
        sqlx::query(
            r#"
            INSERT INTO audit_log (action, table_name, record_id, changes_after, created_by)
            VALUES ('create', 'freezer_transfers', $1, $2, $3)
            "#,
        )
        .bind(transfer.id)
        .bind(serde_json::to_value(&transfer)?)
        .bind(created_by)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(transfer)
    }
}
