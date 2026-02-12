use async_trait::async_trait;
use uuid::Uuid;

use crate::shared::errors::AppError;
use super::entities::*;

#[async_trait]
pub trait WorkerTripRepository: Send + Sync {
    /// Listar viajes activos (en progreso).
    async fn find_active(&self) -> Result<Vec<WorkerTrip>, AppError>;

    /// Listar viajes de un trabajador.
    async fn find_by_worker(&self, worker_id: Uuid, limit: i64) -> Result<Vec<WorkerTrip>, AppError>;

    /// Obtener un viaje con todos sus items.
    async fn find_by_id_with_items(&self, id: Uuid) -> Result<Option<TripWithItems>, AppError>;

    /// Crear un nuevo viaje con items cargados (TRANSACCIÓN: resta inventario).
    async fn create_trip(
        &self,
        dto: &CreateTripDto,
        created_by: Uuid,
    ) -> Result<WorkerTrip, AppError>;

    /// Completar un viaje (TRANSACCIÓN: registrar devoluciones, calcular ventas, actualizar worker).
    async fn complete_trip(
        &self,
        trip_id: Uuid,
        dto: &CompleteTripDto,
        created_by: Uuid,
    ) -> Result<WorkerTrip, AppError>;

    /// Viajes retornados de hoy.
    async fn find_todays_returned(&self) -> Result<Vec<WorkerTrip>, AppError>;
}
