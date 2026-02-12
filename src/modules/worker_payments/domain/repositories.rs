use async_trait::async_trait;
use uuid::Uuid;

use super::entities::WorkerPayment;
use crate::shared::errors::AppError;

#[async_trait]
pub trait WorkerPaymentRepository: Send + Sync {
    async fn find_by_worker(&self, worker_id: Uuid) -> Result<Vec<WorkerPayment>, AppError>;
    async fn find_by_trip(&self, trip_id: Uuid) -> Result<Option<WorkerPayment>, AppError>;
    /// Crea pago, actualiza deuda a 0, registra en caja. Todo en una transacción.
    async fn create_payment(
        &self,
        trip_id: Uuid,
        created_by: Uuid,
    ) -> Result<WorkerPayment, AppError>;
}
