use std::sync::Arc;
use uuid::Uuid;

use crate::shared::errors::AppError;
use crate::modules::users::domain::entities::User;
use crate::modules::users::domain::repositories::UserRepository;

/// Caso de uso: Obtener un usuario por ID.
pub async fn by_id(
    repo: &Arc<dyn UserRepository>,
    id: Uuid,
) -> Result<User, AppError> {
    repo.find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Usuario con id {id} no encontrado")))
}

/// Caso de uso: Obtener un usuario por email.
pub async fn by_email(
    repo: &Arc<dyn UserRepository>,
    email: &str,
) -> Result<User, AppError> {
    repo.find_by_email(email)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Usuario con email {email} no encontrado")))
}
