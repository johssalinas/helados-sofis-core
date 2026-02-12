use crate::modules::local_sales::domain::entities::*;
use crate::modules::local_sales::domain::repositories::LocalSaleRepository;
use crate::shared::errors::AppError;
use uuid::Uuid;

pub async fn list_sales(
    repo: &dyn LocalSaleRepository,
    limit: i64,
) -> Result<Vec<LocalSale>, AppError> {
    repo.find_all(limit).await
}

pub async fn get_sale(
    repo: &dyn LocalSaleRepository,
    id: Uuid,
) -> Result<LocalSaleWithItems, AppError> {
    repo.find_by_id_with_items(id)
        .await?
        .ok_or_else(|| AppError::NotFound("Venta local no encontrada".into()))
}

pub async fn todays_sales(repo: &dyn LocalSaleRepository) -> Result<Vec<LocalSale>, AppError> {
    repo.find_todays().await
}

pub async fn create_sale(
    repo: &dyn LocalSaleRepository,
    dto: &CreateLocalSaleDto,
    created_by: Uuid,
) -> Result<LocalSale, AppError> {
    if dto.items.is_empty() {
        return Err(AppError::BadRequest(
            "La venta debe tener al menos un item".into(),
        ));
    }
    let valid_types = ["local", "custom", "gift", "family"];
    if !valid_types.contains(&dto.sale_type.as_str()) {
        return Err(AppError::BadRequest(format!(
            "Tipo de venta inválido. Debe ser uno de: {:?}",
            valid_types
        )));
    }
    repo.create_sale(dto, created_by).await
}
