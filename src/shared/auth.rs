use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::errors::AppError;

// ─── Roles ──────────────────────────────────────────────

/// Roles del sistema. Owner tiene acceso total, Admin acceso operativo.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    sqlx::Type,
    utoipa::ToSchema,
)]
#[sqlx(type_name = "VARCHAR")]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin = 0,
    Owner = 1,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Admin => write!(f, "admin"),
            Role::Owner => write!(f, "owner"),
        }
    }
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Admin => "admin",
            Role::Owner => "owner",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, AppError> {
        match s {
            "admin" => Ok(Role::Admin),
            "owner" => Ok(Role::Owner),
            _ => Err(AppError::BadRequest(format!("Rol inválido: {s}"))),
        }
    }
}

// ─── JWT Claims ─────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid, // user id
    pub email: String,
    pub role: Role,
    pub exp: usize, // expiration (unix timestamp)
    pub iat: usize, // issued at
}

/// Genera un JWT para el usuario autenticado.
pub fn create_jwt(
    user_id: Uuid,
    email: &str,
    role: Role,
    secret: &str,
) -> Result<String, AppError> {
    let now = chrono::Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: user_id,
        email: email.to_string(),
        role,
        iat: now,
        exp: now + 60 * 60 * 24 * 365, // 1 year (negocio pequeño, sesión larga)
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;
    Ok(token)
}

/// Valida un JWT y devuelve las claims.
pub fn verify_jwt(token: &str, secret: &str) -> Result<Claims, AppError> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(data.claims)
}

// ─── Axum App State ─────────────────────────────────────

/// Estado de la aplicación compartido por todos los handlers.
#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub config: crate::shared::config::AppConfig,
}

// ─── Axum Extractor ─────────────────────────────────────

/// Extractor que valida el JWT del header Authorization y extrae las claims.
/// Uso: `AuthUser(claims)` como parámetro de un handler.
#[derive(Debug, Clone)]
pub struct AuthUser(pub Claims);

impl<S> FromRequestParts<S> for AuthUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jwt_secret = AppState::from_ref(state).config.jwt_secret.clone();

        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Token no proporcionado".into()))?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Unauthorized("Formato de token inválido".into()))?;

        let claims = verify_jwt(token, &jwt_secret)?;
        Ok(AuthUser(claims))
    }
}

// ─── Role Guard Helpers ─────────────────────────────────

impl AuthUser {
    /// Verifica que el usuario tenga al menos el rol requerido.
    pub fn require_role(&self, required: Role) -> Result<(), AppError> {
        if self.0.role >= required {
            Ok(())
        } else {
            Err(AppError::Forbidden("Permisos insuficientes".into()))
        }
    }

    /// Verifica que el usuario sea Owner.
    pub fn require_owner(&self) -> Result<(), AppError> {
        self.require_role(Role::Owner)
    }

    pub fn user_id(&self) -> Uuid {
        self.0.sub
    }

    pub fn role(&self) -> Role {
        self.0.role
    }
}
