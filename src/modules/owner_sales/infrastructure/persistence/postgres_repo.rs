use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::modules::owner_sales::domain::entities::*;
use crate::modules::owner_sales::domain::repositories::OwnerSaleRepository;
use crate::shared::errors::AppError;

pub struct PgOwnerSaleRepository {
    pool: PgPool,
}

impl PgOwnerSaleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OwnerSaleRepository for PgOwnerSaleRepository {
    async fn find_all(&self, limit: i64) -> Result<Vec<OwnerSale>, AppError> {
        Ok(sqlx::query_as::<_, OwnerSale>(
            "SELECT * FROM owner_sales ORDER BY created_at DESC LIMIT $1",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?)
    }

    async fn find_by_id_with_items(
        &self,
        id: Uuid,
    ) -> Result<Option<OwnerSaleWithItems>, AppError> {
        let sale = sqlx::query_as::<_, OwnerSale>("SELECT * FROM owner_sales WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        match sale {
            Some(s) => {
                let loaded = sqlx::query_as::<_, OwnerSaleLoadedItem>(
                    "SELECT * FROM owner_sale_loaded_items WHERE sale_id = $1",
                )
                .bind(id)
                .fetch_all(&self.pool)
                .await?;

                let returned = sqlx::query_as::<_, OwnerSaleReturnedItem>(
                    "SELECT * FROM owner_sale_returned_items WHERE sale_id = $1",
                )
                .bind(id)
                .fetch_all(&self.pool)
                .await?;

                Ok(Some(OwnerSaleWithItems {
                    sale: s,
                    loaded_items: loaded,
                    returned_items: returned,
                }))
            }
            None => Ok(None),
        }
    }

    async fn create_sale(
        &self,
        dto: &CreateOwnerSaleDto,
        owner_id: Uuid,
    ) -> Result<OwnerSale, AppError> {
        let mut tx = self.pool.begin().await?;

        // 1. Crear sale
        let sale = sqlx::query_as::<_, OwnerSale>(
            r#"
            INSERT INTO owner_sales (owner_id, departure_time, route_id, created_by)
            VALUES ($1, $2, $3, $1)
            RETURNING *
            "#,
        )
        .bind(owner_id)
        .bind(dto.departure_time)
        .bind(dto.route_id)
        .fetch_one(&mut *tx)
        .await?;

        // 2. Insertar loaded_items y restar inventario
        for item in &dto.loaded_items {
            sqlx::query(
                r#"
                INSERT INTO owner_sale_loaded_items 
                (sale_id, inventory_id, product_id, flavor_id, freezer_id, quantity, unit_price, is_deformed)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
            )
            .bind(sale.id)
            .bind(item.inventory_id)
            .bind(item.product_id)
            .bind(item.flavor_id)
            .bind(item.freezer_id)
            .bind(item.quantity)
            .bind(item.unit_price)
            .bind(item.is_deformed)
            .execute(&mut *tx)
            .await?;

            let rows = sqlx::query(
                "UPDATE inventory SET quantity = quantity - $1, last_updated = NOW(), updated_by = $2 WHERE id = $3 AND quantity >= $1",
            )
            .bind(item.quantity)
            .bind(owner_id)
            .bind(item.inventory_id)
            .execute(&mut *tx)
            .await?
            .rows_affected();

            if rows == 0 {
                return Err(AppError::InsufficientStock(item.inventory_id));
            }
        }

        // 3. Ruta usage
        if let Some(route_id) = dto.route_id {
            sqlx::query("UPDATE routes SET usage_count = usage_count + 1 WHERE id = $1")
                .bind(route_id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(sale)
    }

    async fn complete_sale(
        &self,
        sale_id: Uuid,
        dto: &CompleteOwnerSaleDto,
        owner_id: Uuid,
    ) -> Result<OwnerSale, AppError> {
        let mut tx = self.pool.begin().await?;

        // 1. Verificar que la venta existe y no está completada
        let _existing = sqlx::query_as::<_, OwnerSale>(
            "SELECT * FROM owner_sales WHERE id = $1 AND return_time IS NULL",
        )
        .bind(sale_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Venta no encontrada o ya completada".into()))?;

        // 2. Obtener loaded_items
        let loaded_items = sqlx::query_as::<_, OwnerSaleLoadedItem>(
            "SELECT * FROM owner_sale_loaded_items WHERE sale_id = $1",
        )
        .bind(sale_id)
        .fetch_all(&mut *tx)
        .await?;

        // 3. Procesar returned_items
        for returned in &dto.returned_items {
            sqlx::query(
                r#"
                INSERT INTO owner_sale_returned_items 
                (sale_id, product_id, flavor_id, quantity, is_deformed, destination_freezer_id)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(sale_id)
            .bind(returned.product_id)
            .bind(returned.flavor_id)
            .bind(returned.quantity)
            .bind(returned.is_deformed)
            .bind(returned.destination_freezer_id)
            .execute(&mut *tx)
            .await?;

            // Buscar provider_id
            let provider_id = sqlx::query_scalar::<_, Uuid>(
                r#"
                SELECT i.provider_id FROM inventory i
                JOIN owner_sale_loaded_items li ON i.id = li.inventory_id
                WHERE li.sale_id = $1 AND li.product_id = $2 AND li.flavor_id = $3
                LIMIT 1
                "#,
            )
            .bind(sale_id)
            .bind(returned.product_id)
            .bind(returned.flavor_id)
            .fetch_optional(&mut *tx)
            .await?;

            let provider_id = match provider_id {
                Some(pid) => pid,
                None => {
                    sqlx::query_scalar::<_, Uuid>(
                        "SELECT provider_id FROM inventory WHERE product_id = $1 AND flavor_id = $2 LIMIT 1",
                    )
                    .bind(returned.product_id)
                    .bind(returned.flavor_id)
                    .fetch_one(&mut *tx)
                    .await?
                }
            };

            if returned.is_deformed {
                // Deformados: nuevo item asignado al dueño (owner como worker)
                sqlx::query(
                    r#"
                    INSERT INTO inventory 
                    (freezer_id, product_id, flavor_id, provider_id, quantity, is_deformed, min_stock_alert, updated_by)
                    VALUES ($1, $2, $3, $4, $5, TRUE, 0, $6)
                    "#,
                )
                .bind(returned.destination_freezer_id)
                .bind(returned.product_id)
                .bind(returned.flavor_id)
                .bind(provider_id)
                .bind(returned.quantity)
                .bind(owner_id)
                .execute(&mut *tx)
                .await?;
            } else {
                // Buenos: UPSERT
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
                .bind(returned.destination_freezer_id)
                .bind(returned.product_id)
                .bind(returned.flavor_id)
                .bind(provider_id)
                .bind(returned.quantity)
                .bind(owner_id)
                .execute(&mut *tx)
                .await?;
            }
        }

        // 4. Calcular ventas
        let (sold_quantity, total_amount) =
            calculate_owner_sales(&loaded_items, &dto.returned_items);

        // 5. Actualizar sale
        let sale = sqlx::query_as::<_, OwnerSale>(
            r#"
            UPDATE owner_sales 
            SET return_time = NOW(),
                sold_quantity = $1,
                total_amount = $2,
                auto_withdrawal = $2
            WHERE id = $3
            RETURNING *
            "#,
        )
        .bind(sold_quantity)
        .bind(total_amount)
        .bind(sale_id)
        .fetch_one(&mut *tx)
        .await?;

        // 6. CAJA: 2 registros (ingreso + retiro automático)
        let current_balance = sqlx::query_scalar::<_, Option<Decimal>>(
            "SELECT balance FROM cash_register ORDER BY created_at DESC LIMIT 1 FOR UPDATE",
        )
        .fetch_optional(&mut *tx)
        .await?
        .flatten()
        .unwrap_or(Decimal::ZERO);

        // Evento 1: Ingreso
        sqlx::query(
            r#"
            INSERT INTO cash_register 
            (type, amount, balance, description, related_doc_type, related_doc_id, created_by)
            VALUES ('owner_sale', $1, $2, 'Venta del dueño', 'owner_sales', $3, $4)
            "#,
        )
        .bind(total_amount)
        .bind(current_balance + total_amount)
        .bind(sale.id)
        .bind(owner_id)
        .execute(&mut *tx)
        .await?;

        // Evento 2: Retiro inmediato
        sqlx::query(
            r#"
            INSERT INTO cash_register 
            (type, amount, balance, description, related_doc_type, related_doc_id, created_by)
            VALUES ('owner_withdrawal', $1, $2, 'Retiro automático por venta del dueño', 'owner_sales', $3, $4)
            "#,
        )
        .bind(-total_amount)
        .bind(current_balance) // Vuelve al balance original
        .bind(sale.id)
        .bind(owner_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(sale)
    }
}

fn calculate_owner_sales(
    loaded: &[OwnerSaleLoadedItem],
    returned: &[OwnerReturnedItemDto],
) -> (i32, Decimal) {
    let mut returned_map: HashMap<(Uuid, Uuid), i32> = HashMap::new();
    for r in returned {
        *returned_map.entry((r.product_id, r.flavor_id)).or_insert(0) += r.quantity;
    }

    let mut total_sold = 0i32;
    let mut total_amount = Decimal::ZERO;

    for item in loaded {
        let returned_qty = returned_map
            .get(&(item.product_id, item.flavor_id))
            .copied()
            .unwrap_or(0);
        let sold = (item.quantity - returned_qty).max(0);
        total_sold += sold;
        total_amount += item.unit_price * Decimal::from(sold);
    }

    (total_sold, total_amount)
}
