use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::shared::auth::Role;
use crate::shared::errors::AppError;
use crate::modules::users::domain::entities::{CreateUserDto, UpdateUserDto, User};
use crate::modules::users::domain::repositories::UserRepository;

/// Implementación PostgreSQL del repositorio de usuarios.
pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(user)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;
        Ok(user)
    }

    async fn find_all(&self) -> Result<Vec<User>, AppError> {
        let users = sqlx::query_as::<_, User>(
            "SELECT * FROM users ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(users)
    }

    async fn find_active(&self) -> Result<Vec<User>, AppError> {
        let users = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE active = TRUE ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(users)
    }

    async fn create(&self, dto: &CreateUserDto, created_by: Option<Uuid>) -> Result<User, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (email, display_name, photo_url, role, notes, created_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(&dto.email)
        .bind(&dto.display_name)
        .bind(&dto.photo_url)
        .bind(dto.role.as_str())
        .bind(&dto.notes)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await?;
        Ok(user)
    }

    async fn update(&self, id: Uuid, dto: &UpdateUserDto) -> Result<User, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users SET
                display_name = COALESCE($1, display_name),
                photo_url    = COALESCE($2, photo_url),
                role         = COALESCE($3, role),
                active       = COALESCE($4, active),
                notes        = COALESCE($5, notes)
            WHERE id = $6
            RETURNING *
            "#,
        )
        .bind(&dto.display_name)
        .bind(&dto.photo_url)
        .bind(dto.role.map(|r| r.as_str()))
        .bind(dto.active)
        .bind(&dto.notes)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        Ok(user)
    }

    async fn update_last_login(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query("UPDATE users SET last_login = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn has_role(&self, id: Uuid, required: Role) -> Result<bool, AppError> {
        let user = self.find_by_id(id).await?;
        match user {
            Some(u) => Ok(u.role >= required),
            None => Ok(false),
        }
    }

    async fn count_by_role(&self, role: Role) -> Result<i64, AppError> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE role = $1 AND active = TRUE",
        )
        .bind(role.as_str())
        .fetch_one(&self.pool)
        .await?;
        Ok(count)
    }
}
