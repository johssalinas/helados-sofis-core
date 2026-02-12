use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::shared::errors::AppError;
use crate::modules::inventory::domain::entities::InventoryItem;
use crate::modules::inventory::domain::repositories::InventoryRepository;

pub struct PgInventoryRepository {
    pool: PgPool,
}

impl PgInventoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl InventoryRepository for PgInventoryRepository {
    async fn find_all(&self) -> Result<Vec<InventoryItem>, AppError> {
        Ok(sqlx::query_as::<_, InventoryItem>("SELECT * FROM inventory ORDER BY freezer_id, product_id")
            .fetch_all(&self.pool)
            .await?)
    }

    async fn find_by_freezer(&self, freezer_id: Uuid) -> Result<Vec<InventoryItem>, AppError> {
        Ok(sqlx::query_as::<_, InventoryItem>(
            "SELECT * FROM inventory WHERE freezer_id = $1 ORDER BY product_id",
        )
        .bind(freezer_id)
        .fetch_all(&self.pool)
        .await?)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<InventoryItem>, AppError> {
        Ok(sqlx::query_as::<_, InventoryItem>("SELECT * FROM inventory WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?)
    }

    async fn find_sellable(&self) -> Result<Vec<InventoryItem>, AppError> {
        Ok(sqlx::query_as::<_, InventoryItem>(
            "SELECT * FROM inventory WHERE is_deformed = FALSE ORDER BY freezer_id, product_id",
        )
        .fetch_all(&self.pool)
        .await?)
    }

    async fn find_low_stock(&self) -> Result<Vec<InventoryItem>, AppError> {
        Ok(sqlx::query_as::<_, InventoryItem>(
            "SELECT * FROM inventory WHERE is_deformed = FALSE AND quantity <= min_stock_alert",
        )
        .fetch_all(&self.pool)
        .await?)
    }

    async fn find_worker_deformed(&self, worker_id: Uuid) -> Result<Vec<InventoryItem>, AppError> {
        Ok(sqlx::query_as::<_, InventoryItem>(
            "SELECT * FROM inventory WHERE assigned_worker_id = $1 AND is_deformed = TRUE",
        )
        .bind(worker_id)
        .fetch_all(&self.pool)
        .await?)
    }

    async fn add_stock(
        &self,
        freezer_id: Uuid,
        product_id: Uuid,
        flavor_id: Uuid,
        provider_id: Uuid,
        quantity: i32,
        updated_by: Uuid,
    ) -> Result<InventoryItem, AppError> {
        Ok(sqlx::query_as::<_, InventoryItem>(
            r#"
            INSERT INTO inventory 
            (freezer_id, product_id, flavor_id, provider_id, quantity, is_deformed, min_stock_alert, updated_by)
            VALUES ($1, $2, $3, $4, $5, FALSE, 20, $6)
            ON CONFLICT (freezer_id, product_id, flavor_id, provider_id, is_deformed, assigned_worker_id)
            DO UPDATE SET 
                quantity = inventory.quantity + EXCLUDED.quantity,
                last_updated = NOW(),
                updated_by = EXCLUDED.updated_by
            RETURNING *
            "#,
        )
        .bind(freezer_id)
        .bind(product_id)
        .bind(flavor_id)
        .bind(provider_id)
        .bind(quantity)
        .bind(updated_by)
        .fetch_one(&self.pool)
        .await?)
    }

    async fn subtract_stock_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        inventory_id: Uuid,
        quantity: i32,
        updated_by: Uuid,
    ) -> Result<(), AppError> {
        let rows = sqlx::query(
            r#"
            UPDATE inventory 
            SET quantity = quantity - $1, last_updated = NOW(), updated_by = $2
            WHERE id = $3 AND quantity >= $1
            "#,
        )
        .bind(quantity)
        .bind(updated_by)
        .bind(inventory_id)
        .execute(&mut **tx)
        .await?
        .rows_affected();

        if rows == 0 {
            return Err(AppError::InsufficientStock(inventory_id));
        }

        // Eliminar registro si quedó en 0 y es deformado
        sqlx::query("DELETE FROM inventory WHERE id = $1 AND quantity = 0 AND is_deformed = TRUE")
            .bind(inventory_id)
            .execute(&mut **tx)
            .await?;

        Ok(())
    }

    async fn add_deformed_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        freezer_id: Uuid,
        product_id: Uuid,
        flavor_id: Uuid,
        provider_id: Uuid,
        quantity: i32,
        worker_id: Uuid,
        updated_by: Uuid,
    ) -> Result<InventoryItem, AppError> {
        Ok(sqlx::query_as::<_, InventoryItem>(
            r#"
            INSERT INTO inventory 
            (freezer_id, product_id, flavor_id, provider_id, quantity, 
             is_deformed, assigned_worker_id, min_stock_alert, updated_by)
            VALUES ($1, $2, $3, $4, $5, TRUE, $6, 0, $7)
            RETURNING *
            "#,
        )
        .bind(freezer_id)
        .bind(product_id)
        .bind(flavor_id)
        .bind(provider_id)
        .bind(quantity)
        .bind(worker_id)
        .bind(updated_by)
        .fetch_one(&mut **tx)
        .await?)
    }

    async fn return_stock_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        freezer_id: Uuid,
        product_id: Uuid,
        flavor_id: Uuid,
        provider_id: Uuid,
        quantity: i32,
        updated_by: Uuid,
    ) -> Result<(), AppError> {
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
        .bind(freezer_id)
        .bind(product_id)
        .bind(flavor_id)
        .bind(provider_id)
        .bind(quantity)
        .bind(updated_by)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    async fn update_alert(&self, id: Uuid, min_stock: i32) -> Result<InventoryItem, AppError> {
        Ok(sqlx::query_as::<_, InventoryItem>(
            "UPDATE inventory SET min_stock_alert = $1 WHERE id = $2 RETURNING *",
        )
        .bind(min_stock)
        .bind(id)
        .fetch_one(&self.pool)
        .await?)
    }
}
