use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::shared::errors::AppError;
use crate::modules::worker_trips::domain::entities::*;
use crate::modules::worker_trips::domain::repositories::WorkerTripRepository;

pub struct PgWorkerTripRepository {
    pool: PgPool,
}

impl PgWorkerTripRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WorkerTripRepository for PgWorkerTripRepository {
    async fn find_active(&self) -> Result<Vec<WorkerTrip>, AppError> {
        Ok(sqlx::query_as::<_, WorkerTrip>(
            "SELECT * FROM worker_trips WHERE status = 'in_progress' ORDER BY departure_time DESC",
        )
        .fetch_all(&self.pool)
        .await?)
    }

    async fn find_by_worker(&self, worker_id: Uuid, limit: i64) -> Result<Vec<WorkerTrip>, AppError> {
        Ok(sqlx::query_as::<_, WorkerTrip>(
            "SELECT * FROM worker_trips WHERE worker_id = $1 ORDER BY departure_time DESC LIMIT $2",
        )
        .bind(worker_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?)
    }

    async fn find_by_id_with_items(&self, id: Uuid) -> Result<Option<TripWithItems>, AppError> {
        let trip = sqlx::query_as::<_, WorkerTrip>("SELECT * FROM worker_trips WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        match trip {
            Some(t) => {
                let loaded = sqlx::query_as::<_, LoadedItem>(
                    "SELECT * FROM worker_trip_loaded_items WHERE trip_id = $1",
                )
                .bind(id)
                .fetch_all(&self.pool)
                .await?;

                let returned = sqlx::query_as::<_, ReturnedItem>(
                    "SELECT * FROM worker_trip_returned_items WHERE trip_id = $1",
                )
                .bind(id)
                .fetch_all(&self.pool)
                .await?;

                Ok(Some(TripWithItems {
                    trip: t,
                    loaded_items: loaded,
                    returned_items: returned,
                }))
            }
            None => Ok(None),
        }
    }

    async fn create_trip(
        &self,
        dto: &CreateTripDto,
        created_by: Uuid,
    ) -> Result<WorkerTrip, AppError> {
        let mut tx = self.pool.begin().await?;

        // 1. Crear el viaje
        let trip = sqlx::query_as::<_, WorkerTrip>(
            r#"
            INSERT INTO worker_trips (worker_id, departure_time, route_id, status, created_by)
            VALUES ($1, $2, $3, 'in_progress', $4)
            RETURNING *
            "#,
        )
        .bind(dto.worker_id)
        .bind(dto.departure_time)
        .bind(dto.route_id)
        .bind(created_by)
        .fetch_one(&mut *tx)
        .await?;

        // 2. Incrementar usage_count de la ruta si hay
        if let Some(route_id) = dto.route_id {
            sqlx::query("UPDATE routes SET usage_count = usage_count + 1 WHERE id = $1")
                .bind(route_id)
                .execute(&mut *tx)
                .await?;
        }

        // 3. Insertar items cargados y restar inventario
        for item in &dto.loaded_items {
            sqlx::query(
                r#"
                INSERT INTO worker_trip_loaded_items 
                (trip_id, inventory_id, product_id, flavor_id, freezer_id, quantity, unit_price, is_deformed)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
            )
            .bind(trip.id)
            .bind(item.inventory_id)
            .bind(item.product_id)
            .bind(item.flavor_id)
            .bind(item.freezer_id)
            .bind(item.quantity)
            .bind(item.unit_price)
            .bind(item.is_deformed)
            .execute(&mut *tx)
            .await?;

            // Restar inventario con verificación
            let rows = sqlx::query(
                r#"
                UPDATE inventory 
                SET quantity = quantity - $1, last_updated = NOW(), updated_by = $2
                WHERE id = $3 AND quantity >= $1
                "#,
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

            // Si era deformado y quedó en 0, eliminar
            sqlx::query(
                "DELETE FROM inventory WHERE id = $1 AND quantity = 0 AND is_deformed = TRUE",
            )
            .bind(item.inventory_id)
            .execute(&mut *tx)
            .await?;
        }

        // 4. Auditoría
        sqlx::query(
            r#"
            INSERT INTO audit_log (action, table_name, record_id, changes_after, created_by)
            VALUES ('create', 'worker_trips', $1, $2, $3)
            "#,
        )
        .bind(trip.id)
        .bind(serde_json::to_value(&trip)?)
        .bind(created_by)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(trip)
    }

    async fn complete_trip(
        &self,
        trip_id: Uuid,
        dto: &CompleteTripDto,
        created_by: Uuid,
    ) -> Result<WorkerTrip, AppError> {
        let mut tx = self.pool.begin().await?;

        // 1. Verificar que el viaje existe y está in_progress
        let existing = sqlx::query_as::<_, WorkerTrip>(
            "SELECT * FROM worker_trips WHERE id = $1 AND status = 'in_progress'",
        )
        .bind(trip_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| {
            AppError::NotFound("Viaje no encontrado o ya fue completado".into())
        })?;

        // 2. Obtener loaded_items para calcular ventas
        let loaded_items = sqlx::query_as::<_, LoadedItem>(
            "SELECT * FROM worker_trip_loaded_items WHERE trip_id = $1",
        )
        .bind(trip_id)
        .fetch_all(&mut *tx)
        .await?;

        // 3. Insertar returned_items y procesar inventario
        for returned in &dto.returned_items {
            sqlx::query(
                r#"
                INSERT INTO worker_trip_returned_items 
                (trip_id, product_id, flavor_id, quantity, is_deformed, destination_freezer_id)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(trip_id)
            .bind(returned.product_id)
            .bind(returned.flavor_id)
            .bind(returned.quantity)
            .bind(returned.is_deformed)
            .bind(returned.destination_freezer_id)
            .execute(&mut *tx)
            .await?;

            // Obtener provider_id del item original
            let provider_id = sqlx::query_scalar::<_, Uuid>(
                r#"
                SELECT i.provider_id FROM inventory i
                JOIN worker_trip_loaded_items li ON i.id = li.inventory_id
                WHERE li.trip_id = $1 AND li.product_id = $2 AND li.flavor_id = $3
                LIMIT 1
                "#,
            )
            .bind(trip_id)
            .bind(returned.product_id)
            .bind(returned.flavor_id)
            .fetch_optional(&mut *tx)
            .await?;

            // Si no encontramos provider_id (inventario ya eliminado), usar primer loaded_item match
            let provider_id = match provider_id {
                Some(pid) => pid,
                None => {
                    // Fallback: buscar en inventory global
                    sqlx::query_scalar::<_, Uuid>(
                        r#"
                        SELECT provider_id FROM inventory 
                        WHERE product_id = $1 AND flavor_id = $2
                        LIMIT 1
                        "#,
                    )
                    .bind(returned.product_id)
                    .bind(returned.flavor_id)
                    .fetch_one(&mut *tx)
                    .await?
                }
            };

            if returned.is_deformed {
                // Deformados → nuevo item asignado al trabajador
                sqlx::query(
                    r#"
                    INSERT INTO inventory 
                    (freezer_id, product_id, flavor_id, provider_id, quantity, 
                     is_deformed, assigned_worker_id, min_stock_alert, updated_by)
                    VALUES ($1, $2, $3, $4, $5, TRUE, $6, 0, $7)
                    "#,
                )
                .bind(returned.destination_freezer_id)
                .bind(returned.product_id)
                .bind(returned.flavor_id)
                .bind(provider_id)
                .bind(returned.quantity)
                .bind(existing.worker_id)
                .bind(created_by)
                .execute(&mut *tx)
                .await?;
            } else {
                // Buenos → UPSERT al inventario normal
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
                .bind(created_by)
                .execute(&mut *tx)
                .await?;
            }
        }

        // 4. Calcular sold_quantity y amount_due
        let (sold_quantity, amount_due) =
            calculate_sales(&loaded_items, &dto.returned_items);

        // 5. Actualizar el viaje
        let trip = sqlx::query_as::<_, WorkerTrip>(
            r#"
            UPDATE worker_trips 
            SET return_time = NOW(), status = 'returned',
                sold_quantity = $1, amount_due = $2
            WHERE id = $3
            RETURNING *
            "#,
        )
        .bind(sold_quantity)
        .bind(amount_due)
        .bind(trip_id)
        .fetch_one(&mut *tx)
        .await?;

        // 6. Actualizar denormalizados del trabajador
        sqlx::query(
            r#"
            UPDATE workers 
            SET current_debt = current_debt + $1,
                total_sales = total_sales + $2,
                last_sale = NOW()
            WHERE id = $3
            "#,
        )
        .bind(amount_due)
        .bind(sold_quantity)
        .bind(existing.worker_id)
        .execute(&mut *tx)
        .await?;

        // 7. Auditoría
        sqlx::query(
            r#"
            INSERT INTO audit_log (action, table_name, record_id, changes_before, changes_after, created_by)
            VALUES ('update', 'worker_trips', $1, $2, $3, $4)
            "#,
        )
        .bind(trip_id)
        .bind(serde_json::to_value(&existing)?)
        .bind(serde_json::to_value(&trip)?)
        .bind(created_by)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(trip)
    }

    async fn find_todays_returned(&self) -> Result<Vec<WorkerTrip>, AppError> {
        Ok(sqlx::query_as::<_, WorkerTrip>(
            r#"
            SELECT * FROM worker_trips 
            WHERE departure_time >= CURRENT_DATE
              AND departure_time < CURRENT_DATE + INTERVAL '1 day'
              AND status = 'returned'
            ORDER BY departure_time DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?)
    }
}

/// Calcula la cantidad vendida y el monto adeudado.
fn calculate_sales(loaded: &[LoadedItem], returned: &[ReturnedItemDto]) -> (i32, Decimal) {
    // Agrupar devoluciones por (product_id, flavor_id)
    let mut returned_map: HashMap<(Uuid, Uuid), i32> = HashMap::new();
    for r in returned {
        *returned_map
            .entry((r.product_id, r.flavor_id))
            .or_insert(0) += r.quantity;
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
