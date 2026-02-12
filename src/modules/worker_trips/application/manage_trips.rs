use uuid::Uuid;

use crate::shared::errors::AppError;
use crate::modules::worker_trips::domain::entities::*;
use crate::modules::worker_trips::domain::repositories::WorkerTripRepository;

pub async fn list_active(
    repo: &dyn WorkerTripRepository,
) -> Result<Vec<WorkerTrip>, AppError> {
    repo.find_active().await
}

pub async fn list_by_worker(
    repo: &dyn WorkerTripRepository,
    worker_id: Uuid,
    limit: i64,
) -> Result<Vec<WorkerTrip>, AppError> {
    repo.find_by_worker(worker_id, limit).await
}

pub async fn get_trip(
    repo: &dyn WorkerTripRepository,
    id: Uuid,
) -> Result<TripWithItems, AppError> {
    repo.find_by_id_with_items(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Viaje {id} no encontrado")))
}

pub async fn create_trip(
    repo: &dyn WorkerTripRepository,
    dto: &CreateTripDto,
    created_by: Uuid,
) -> Result<WorkerTrip, AppError> {
    if dto.loaded_items.is_empty() {
        return Err(AppError::BadRequest(
            "El viaje debe tener al menos un item cargado".into(),
        ));
    }
    repo.create_trip(dto, created_by).await
}

pub async fn complete_trip(
    repo: &dyn WorkerTripRepository,
    trip_id: Uuid,
    dto: &CompleteTripDto,
    created_by: Uuid,
) -> Result<WorkerTrip, AppError> {
    repo.complete_trip(trip_id, dto, created_by).await
}

pub async fn todays_returned(
    repo: &dyn WorkerTripRepository,
) -> Result<Vec<WorkerTrip>, AppError> {
    repo.find_todays_returned().await
}
