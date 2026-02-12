use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::shared::errors::AppError;
use crate::modules::local_sales::domain::entities::*;
use crate::modules::local_sales::domain::repositories::LocalSaleRepository;

pub struct PgLocalSaleRepository {
    pool: PgPool,
}

impl PgLocalSaleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LocalSaleRepository for PgLocalSaleRepository {
    async fn find_all(&self, limit: i64) -> Result<Vec<LocalSale>, AppError> {
        Ok(sqlx::query_as::<_, LocalSale>(
            "SELECT * FROM local_sales ORDER BY created_at DESC LIMIT $1",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?)
    }

    async fn find_by_id_with_items(
        &self,
        id: Uuid,
    ) -> Result<Option<LocalSaleWithItems>, AppError> {
        let sale = sqlx::query_as::<_, LocalSale>("SELECT * FROM local_sales WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        match sale {
            Some(s) => {
                let items = sqlx::query_as::<_, LocalSaleItem>(
                    "SELECT * FROM local_sale_items WHERE sale_id = $1",
                )
                .bind(id)
                .fetch_all(&self.pool)
                .await?;

                Ok(Some(LocalSaleWithItems { sale: s, items }))
            }
            None => Ok(None),
        }
    }

    async fn find_todays(&self) -> Result<Vec<LocalSale>, AppError> {
        Ok(sqlx::query_as::<_, LocalSale>(
            r#"
            SELECT * FROM local_sales 
            WHERE created_at >= CURRENT_DATE 
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?)
    }

    async fn create_sale(
        &self,
        dto: &CreateLocalSaleDto,
        created_by: Uuid,
    ) -> Result<LocalSale, AppError> {
        let mut tx = self.pool.begin().await?;

        // Calcular total
        let total: Decimal = dto
            .items
            .iter()
            .map(|i| i.unit_price * Decimal::from(i.quantity))
            .sum();

        // 1. Crear venta
        let sale = sqlx::query_as::<_, LocalSale>(
            r#"
            INSERT INTO local_sales (total, sale_type, notes, created_by)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(total)
        .bind(&dto.sale_type)
        .bind(&dto.notes)
        .bind(created_by)
        .fetch_one(&mut *tx)
        .await?;

        // 2. Insertar items y restar inventario
        for item in &dto.items {
            sqlx::query(
                r#"
                INSERT INTO local_sale_items 
                (sale_id, inventory_id, product_id, flavor_id, freezer_id, quantity, unit_price)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(sale.id)
            .bind(item.inventory_id)
            .bind(item.product_id)
            .bind(item.flavor_id)
            .bind(item.freezer_id)
            .bind(item.quantity)
            .bind(item.unit_price)
            .execute(&mut *tx)
            .await?;

            // Restar inventario
            let rows = sqlx::query(
                "UPDATE inventory SET quantity = quantity - $1, last_updated = NOW(), updated_by = $2 WHERE id = $3 AND quantity >= $1",
            )
            .bind(item.quantity)
            .bind(created_by)
            .bind(item.inventory_id)
            .execute(&mut *tx)
            .await?
            .rows_affected();

            if rows == 0 {
                return Err(AppError::InsufficientStock(item.inventory_id));
            }
        }

        // 3. Registrar en caja (solo si es venta "local" o "custom", no para regalos)
        if dto.sale_type != "gift" {
            let current_balance = sqlx::query_scalar::<_, Option<Decimal>>(
                "SELECT balance FROM cash_register ORDER BY created_at DESC LIMIT 1 FOR UPDATE",
            )
            .fetch_optional(&mut *tx)
            .await?
            .flatten()
            .unwrap_or(Decimal::ZERO);

            let new_balance = current_balance + total;

            sqlx::query(
                r#"
                INSERT INTO cash_register 
                (type, amount, balance, related_doc_type, related_doc_id, created_by)
                VALUES ('local_sale', $1, $2, 'local_sales', $3, $4)
                "#,
            )
            .bind(total)
            .bind(new_balance)
            .bind(sale.id)
            .bind(created_by)
            .execute(&mut *tx)
            .await?;
        }

        // 4. Auditoría
        sqlx::query(
            r#"
            INSERT INTO audit_log (action, table_name, record_id, changes_after, created_by)
            VALUES ('create', 'local_sales', $1, $2, $3)
            "#,
        )
        .bind(sale.id)
        .bind(serde_json::to_value(&sale)?)
        .bind(created_by)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(sale)
    }
}
