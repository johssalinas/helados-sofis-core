use serde::{Deserialize, Serialize};

/// Solicitud de login con Google ID token.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct GoogleLoginRequest {
    pub id_token: String,
}

/// Respuesta de Google tokeninfo.
#[derive(Debug, Deserialize)]
pub struct GoogleTokenInfo {
    pub sub: String,
    pub email: String,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub email_verified: Option<String>,
    pub aud: String,
}

/// Respuesta de login exitoso: JWT propio del sistema.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct LoginResponse {
    pub token: String,
    pub user: AuthUserInfo,
}

/// Info pública del usuario autenticado.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AuthUserInfo {
    pub id: uuid::Uuid,
    pub email: String,
    pub display_name: String,
    pub photo_url: Option<String>,
    pub role: crate::shared::auth::Role,
}
