use crate::modules::owner_sales::domain::entities::*;
use crate::modules::owner_sales::domain::repositories::OwnerSaleRepository;
use crate::shared::errors::AppError;
use uuid::Uuid;

pub async fn list_sales(
    repo: &dyn OwnerSaleRepository,
    limit: i64,
) -> Result<Vec<OwnerSale>, AppError> {
    repo.find_all(limit).await
}

pub async fn get_sale(
    repo: &dyn OwnerSaleRepository,
    id: Uuid,
) -> Result<OwnerSaleWithItems, AppError> {
    repo.find_by_id_with_items(id)
        .await?
        .ok_or_else(|| AppError::NotFound("Venta del dueño no encontrada".into()))
}

pub async fn create_sale(
    repo: &dyn OwnerSaleRepository,
    dto: &CreateOwnerSaleDto,
    owner_id: Uuid,
) -> Result<OwnerSale, AppError> {
    if dto.loaded_items.is_empty() {
        return Err(AppError::BadRequest("Debe cargar al menos un item".into()));
    }
    repo.create_sale(dto, owner_id).await
}

pub async fn complete_sale(
    repo: &dyn OwnerSaleRepository,
    sale_id: Uuid,
    dto: &CompleteOwnerSaleDto,
    owner_id: Uuid,
) -> Result<OwnerSale, AppError> {
    repo.complete_sale(sale_id, dto, owner_id).await
}
