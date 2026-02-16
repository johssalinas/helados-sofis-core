use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::worker_payments::domain::entities::WorkerPayment;
use crate::modules::worker_payments::domain::repositories::WorkerPaymentRepository;
use crate::shared::errors::AppError;

pub struct PgWorkerPaymentRepository {
    pool: PgPool,
}

impl PgWorkerPaymentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct WorkerDebt {
    current_debt: Decimal,
}

#[derive(sqlx::FromRow)]
struct TripAmountDue {
    worker_id: Uuid,
    amount_due: Option<Decimal>,
    status: String,
}

#[async_trait]
impl WorkerPaymentRepository for PgWorkerPaymentRepository {
    async fn find_by_worker(&self, worker_id: Uuid) -> Result<Vec<WorkerPayment>, AppError> {
        Ok(sqlx::query_as::<_, WorkerPayment>(
            "SELECT * FROM worker_payments WHERE worker_id = $1 ORDER BY created_at DESC",
        )
        .bind(worker_id)
        .fetch_all(&self.pool)
        .await?)
    }

    async fn find_by_trip(&self, trip_id: Uuid) -> Result<Option<WorkerPayment>, AppError> {
        Ok(
            sqlx::query_as::<_, WorkerPayment>("SELECT * FROM worker_payments WHERE trip_id = $1")
                .bind(trip_id)
                .fetch_optional(&self.pool)
                .await?,
        )
    }

    async fn create_payment(
        &self,
        trip_id: Uuid,
        created_by: Uuid,
    ) -> Result<WorkerPayment, AppError> {
        let mut tx = self.pool.begin().await?;

        // 1. Obtener el viaje y verificar estado
        let trip = sqlx::query_as::<_, TripAmountDue>(
            "SELECT worker_id, amount_due, status FROM worker_trips WHERE id = $1",
        )
        .bind(trip_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Viaje no encontrado".into()))?;

        if trip.status != "returned" {
            return Err(AppError::BadRequest(
                "El viaje debe estar completado para registrar pago".into(),
            ));
        }

        let amount_due = trip.amount_due.unwrap_or(Decimal::ZERO);

        // 2. Obtener deuda actual del trabajador
        let worker = sqlx::query_as::<_, WorkerDebt>(
            "SELECT current_debt FROM workers WHERE id = $1",
        )
        .bind(trip.worker_id)
        .fetch_one(&mut *tx)
        .await?;

        let new_debt = worker.current_debt - amount_due;

        // 3. Registrar pago
        let payment = sqlx::query_as::<_, WorkerPayment>(
            r#"
            INSERT INTO worker_payments 
            (worker_id, trip_id, amount, previous_debt, new_debt, created_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(trip.worker_id)
        .bind(trip_id)
        .bind(amount_due)
        .bind(worker.current_debt)
        .bind(new_debt)
        .bind(created_by)
        .fetch_one(&mut *tx)
        .await?;

        // 4. Actualizar deuda del trabajador
        sqlx::query("UPDATE workers SET current_debt = $1 WHERE id = $2")
            .bind(new_debt)
            .execute(&mut *tx)
            .await?;

        // 5. Registrar en caja (Event Sourcing)
        let current_balance = sqlx::query_scalar::<_, Option<Decimal>>(
            "SELECT balance FROM cash_register ORDER BY created_at DESC LIMIT 1 FOR UPDATE",
        )
        .fetch_optional(&mut *tx)
        .await?
        .flatten()
        .unwrap_or(Decimal::ZERO);

        let new_balance = current_balance + amount_due;

        sqlx::query(
            r#"
            INSERT INTO cash_register 
            (type, amount, balance, related_doc_type, related_doc_id, created_by)
            VALUES ('worker_payment', $1, $2, 'worker_payments', $3, $4)
            "#,
        )
        .bind(amount_due)
        .bind(new_balance)
        .bind(payment.id)
        .bind(created_by)
        .execute(&mut *tx)
        .await?;

        // 6. Auditoría
        sqlx::query(
            r#"
            INSERT INTO audit_log (action, table_name, record_id, changes_after, created_by)
            VALUES ('create', 'worker_payments', $1, $2, $3)
            "#,
        )
        .bind(payment.id)
        .bind(serde_json::to_value(&payment)?)
        .bind(created_by)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(payment)
    }
}
