use async_trait::async_trait;
use uuid::Uuid;

use crate::shared::auth::Role;
use crate::shared::errors::AppError;

use super::entities::{CreateUserDto, UpdateUserDto, User};

/// Puerto de salida (hexagonal): contrato de persistencia para usuarios.
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Buscar usuario por ID.
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError>;

    /// Buscar usuario por email.
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError>;

    /// Listar todos los usuarios (activos e inactivos).
    async fn find_all(&self) -> Result<Vec<User>, AppError>;

    /// Listar solo usuarios activos.
    async fn find_active(&self) -> Result<Vec<User>, AppError>;

    /// Crear un nuevo usuario.
    async fn create(&self, dto: &CreateUserDto, created_by: Option<Uuid>) -> Result<User, AppError>;

    /// Actualizar un usuario existente.
    async fn update(&self, id: Uuid, dto: &UpdateUserDto) -> Result<User, AppError>;

    /// Actualizar la fecha de último login.
    async fn update_last_login(&self, id: Uuid) -> Result<(), AppError>;

    /// Verificar si un usuario tiene un rol específico o superior.
    async fn has_role(&self, id: Uuid, required: Role) -> Result<bool, AppError>;

    /// Contar usuarios con cierto rol.
    async fn count_by_role(&self, role: Role) -> Result<i64, AppError>;
}
