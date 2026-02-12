use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::auth::Role;

/// Entidad de dominio: Usuario del sistema.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub photo_url: Option<String>,
    pub role: Role,
    pub active: bool,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub last_login: Option<DateTime<Utc>>,
}

/// DTO para crear un nuevo usuario.
#[derive(Debug, Deserialize)]
pub struct CreateUserDto {
    pub email: String,
    pub display_name: String,
    pub photo_url: Option<String>,
    pub role: Role,
    pub notes: Option<String>,
}

/// DTO para actualizar un usuario existente.
#[derive(Debug, Deserialize)]
pub struct UpdateUserDto {
    pub display_name: Option<String>,
    pub photo_url: Option<String>,
    pub role: Option<Role>,
    pub active: Option<bool>,
    pub notes: Option<String>,
}

/// Respuesta pública de usuario (sin campos internos sensibles).
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub photo_url: Option<String>,
    pub role: Role,
    pub active: bool,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            email: u.email,
            display_name: u.display_name,
            photo_url: u.photo_url,
            role: u.role,
            active: u.active,
            notes: u.notes,
            created_at: u.created_at,
            last_login: u.last_login,
        }
    }
}
