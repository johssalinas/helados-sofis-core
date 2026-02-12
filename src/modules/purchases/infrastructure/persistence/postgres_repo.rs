use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::purchases::domain::entities::*;
use crate::modules::purchases::domain::repositories::PurchaseRepository;
use crate::shared::errors::AppError;

pub struct PgPurchaseRepository {
    pool: PgPool,
}

impl PgPurchaseRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PurchaseRepository for PgPurchaseRepository {
    async fn find_all(&self) -> Result<Vec<Purchase>, AppError> {
        Ok(
            sqlx::query_as::<_, Purchase>("SELECT * FROM purchases ORDER BY created_at DESC")
                .fetch_all(&self.pool)
                .await?,
        )
    }

    async fn find_by_id_with_items(&self, id: Uuid) -> Result<Option<PurchaseWithItems>, AppError> {
        let purchase = sqlx::query_as::<_, Purchase>("SELECT * FROM purchases WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        match purchase {
            Some(p) => {
                let items = sqlx::query_as::<_, PurchaseItem>(
                    "SELECT * FROM purchase_items WHERE purchase_id = $1",
                )
                .bind(id)
                .fetch_all(&self.pool)
                .await?;

                Ok(Some(PurchaseWithItems { purchase: p, items }))
            }
            None => Ok(None),
        }
    }

    async fn create(
        &self,
        dto: &CreatePurchaseDto,
        created_by: Uuid,
    ) -> Result<PurchaseWithItems, AppError> {
        let mut tx = self.pool.begin().await?;

        // Calcular total
        let total: Decimal = dto
            .items
            .iter()
            .map(|i| i.unit_price * Decimal::from(i.quantity))
            .sum();

        let paid_at = if dto.payment_status == "paid" {
            Some(chrono::Utc::now())
        } else {
            None
        };

        // Crear la compra
        let purchase = sqlx::query_as::<_, Purchase>(
            r#"
            INSERT INTO purchases (provider_id, total, payment_status, paid_at, created_by)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(dto.provider_id)
        .bind(total)
        .bind(&dto.payment_status)
        .bind(paid_at)
        .bind(created_by)
        .fetch_one(&mut *tx)
        .await?;

        // Insertar items y actualizar inventario
        let mut items = Vec::new();
        for item_dto in &dto.items {
            let item = sqlx::query_as::<_, PurchaseItem>(
                r#"
                INSERT INTO purchase_items 
                (purchase_id, product_id, flavor_id, quantity, unit_price, freezer_id)
                VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING *
                "#,
            )
            .bind(purchase.id)
            .bind(item_dto.product_id)
            .bind(item_dto.flavor_id)
            .bind(item_dto.quantity)
            .bind(item_dto.unit_price)
            .bind(item_dto.freezer_id)
            .fetch_one(&mut *tx)
            .await?;

            // UPSERT inventario — agregar stock comprado
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
            .bind(item_dto.freezer_id)
            .bind(item_dto.product_id)
            .bind(item_dto.flavor_id)
            .bind(dto.provider_id)
            .bind(item_dto.quantity)
            .bind(created_by)
            .execute(&mut *tx)
            .await?;

            items.push(item);
        }

        tx.commit().await?;

        Ok(PurchaseWithItems { purchase, items })
    }
}
