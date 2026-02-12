use std::sync::Arc;
use uuid::Uuid;

use crate::modules::users::domain::entities::{CreateUserDto, User};
use crate::modules::users::domain::repositories::UserRepository;
use crate::shared::errors::AppError;

/// Caso de uso: Crear un nuevo usuario.
/// Solo el Owner puede crear usuarios.
pub async fn execute(
    repo: &Arc<dyn UserRepository>,
    dto: CreateUserDto,
    created_by: Uuid,
) -> Result<User, AppError> {
    // Verificar que no exista ya un usuario con ese email
    if let Some(_existing) = repo.find_by_email(&dto.email).await? {
        return Err(AppError::Conflict(format!(
            "Ya existe un usuario con el email: {}",
            dto.email
        )));
    }

    let user = repo.create(&dto, Some(created_by)).await?;
    Ok(user)
}
