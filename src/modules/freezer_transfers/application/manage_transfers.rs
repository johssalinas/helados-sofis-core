use crate::modules::freezer_transfers::domain::entities::*;
use crate::modules::freezer_transfers::domain::repositories::FreezerTransferRepository;
use crate::shared::errors::AppError;
use uuid::Uuid;

pub async fn list_transfers(
    repo: &dyn FreezerTransferRepository,
    limit: i64,
) -> Result<Vec<FreezerTransfer>, AppError> {
    repo.find_all(limit).await
}

pub async fn get_transfer(
    repo: &dyn FreezerTransferRepository,
    id: Uuid,
) -> Result<TransferWithItems, AppError> {
    repo.find_by_id_with_items(id)
        .await?
        .ok_or_else(|| AppError::NotFound("Transferencia no encontrada".into()))
}

pub async fn list_by_freezer(
    repo: &dyn FreezerTransferRepository,
    freezer_id: Uuid,
) -> Result<Vec<FreezerTransfer>, AppError> {
    repo.find_by_freezer(freezer_id).await
}

pub async fn create_transfer(
    repo: &dyn FreezerTransferRepository,
    dto: &CreateTransferDto,
    created_by: Uuid,
) -> Result<FreezerTransfer, AppError> {
    if dto.items.is_empty() {
        return Err(AppError::BadRequest(
            "Debe transferir al menos un item".into(),
        ));
    }
    if dto.from_freezer_id == dto.to_freezer_id {
        return Err(AppError::BadRequest(
            "Origen y destino deben ser diferentes".into(),
        ));
    }
    repo.create_transfer(dto, created_by).await
}
