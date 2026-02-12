use std::sync::Arc;

use crate::modules::users::domain::entities::User;
use crate::modules::users::domain::repositories::UserRepository;
use crate::shared::errors::AppError;

/// Caso de uso: Listar todos los usuarios.
pub async fn all(repo: &Arc<dyn UserRepository>) -> Result<Vec<User>, AppError> {
    repo.find_all().await
}

/// Caso de uso: Listar solo usuarios activos.
pub async fn active_only(repo: &Arc<dyn UserRepository>) -> Result<Vec<User>, AppError> {
    repo.find_active().await
}
