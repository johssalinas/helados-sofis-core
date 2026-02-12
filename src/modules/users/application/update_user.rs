use std::sync::Arc;
use uuid::Uuid;

use crate::modules::users::domain::entities::{UpdateUserDto, User};
use crate::modules::users::domain::repositories::UserRepository;
use crate::shared::errors::AppError;

/// Caso de uso: Actualizar un usuario existente.
/// Solo el Owner puede cambiar roles o desactivar usuarios.
pub async fn execute(
    repo: &Arc<dyn UserRepository>,
    id: Uuid,
    dto: UpdateUserDto,
) -> Result<User, AppError> {
    // Verificar que el usuario existe
    let _existing = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Usuario con id {id} no encontrado")))?;

    let user = repo.update(id, &dto).await?;
    Ok(user)
}
