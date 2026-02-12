use uuid::Uuid;
use crate::shared::errors::AppError;
use crate::modules::worker_payments::domain::entities::WorkerPayment;
use crate::modules::worker_payments::domain::repositories::WorkerPaymentRepository;

pub async fn list_by_worker(
    repo: &dyn WorkerPaymentRepository,
    worker_id: Uuid,
) -> Result<Vec<WorkerPayment>, AppError> {
    repo.find_by_worker(worker_id).await
}

pub async fn get_by_trip(
    repo: &dyn WorkerPaymentRepository,
    trip_id: Uuid,
) -> Result<WorkerPayment, AppError> {
    repo.find_by_trip(trip_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Pago no encontrado para este viaje".into()))
}

pub async fn create_payment(
    repo: &dyn WorkerPaymentRepository,
    trip_id: Uuid,
    created_by: Uuid,
) -> Result<WorkerPayment, AppError> {
    // Verificar que no exista pago para este viaje
    if repo.find_by_trip(trip_id).await?.is_some() {
        return Err(AppError::Conflict("Ya existe un pago para este viaje".into()));
    }
    repo.create_payment(trip_id, created_by).await
}
