use std::sync::Arc;

use axum::{extract::State, routing::post, Json, Router};
use utoipa::OpenApi;

use crate::modules::auth::application::google_login;
use crate::modules::auth::domain::entities::{GoogleLoginRequest, GoogleTokenInfo, LoginResponse};
use crate::modules::users::domain::repositories::UserRepository;
use crate::shared::auth::AppState;
use crate::shared::errors::AppError;

#[derive(OpenApi)]
#[openapi(
    paths(google_login_handler),
    components(schemas(
        crate::modules::auth::domain::entities::GoogleLoginRequest,
        crate::modules::auth::domain::entities::LoginResponse,
        crate::modules::auth::domain::entities::AuthUserInfo,
    ))
)]
pub struct AuthApiDoc;

/// Estado del módulo auth.
#[derive(Clone)]
pub struct AuthState {
    pub app: AppState,
    pub user_repo: Arc<dyn UserRepository>,
}

impl axum::extract::FromRef<AuthState> for AppState {
    fn from_ref(s: &AuthState) -> AppState {
        s.app.clone()
    }
}

/// Crea el router del módulo auth.
pub fn router(app_state: AppState, user_repo: Arc<dyn UserRepository>) -> Router {
    let state = AuthState {
        app: app_state,
        user_repo,
    };

    Router::new()
        .route("/google", post(google_login_handler))
        .with_state(state)
}

/// POST /auth/google — Login con Google ID token.
///
/// Body: `{ "id_token": "eyJhbGci..." }`
///
/// El backend valida el token con Google tokeninfo, verifica
/// que el usuario exista (o lo crea si es el primero), y devuelve
/// un JWT propio del sistema.
#[utoipa::path(
    post,
    path = "/google",
    tag = "Auth",
    request_body = GoogleLoginRequest,
    responses(
        (status = 200, description = "Login exitoso", body = LoginResponse),
        (status = 401, description = "Token de Google inválido")
    )
)]
async fn google_login_handler(
    State(state): State<AuthState>,
    Json(req): Json<GoogleLoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    // Validar el token de Google contra su API
    let token_info = validate_google_token(&req.id_token).await?;

    let response = google_login::execute(
        &state.user_repo,
        &state.app.config.google_client_id,
        &state.app.config.jwt_secret,
        token_info,
    )
    .await?;

    Ok(Json(response))
}

/// Valida un ID token de Google contra la API tokeninfo.
async fn validate_google_token(id_token: &str) -> Result<GoogleTokenInfo, AppError> {
    let url = format!(
        "https://oauth2.googleapis.com/tokeninfo?id_token={}",
        id_token
    );

    let response = reqwest::get(&url)
        .await
        .map_err(|e| AppError::Internal(format!("Error contactando Google: {e}")))?;

    if !response.status().is_success() {
        return Err(AppError::Unauthorized(
            "Token de Google inválido o expirado".into(),
        ));
    }

    let token_info: GoogleTokenInfo = response
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("Error parseando respuesta de Google: {e}")))?;

    Ok(token_info)
}
