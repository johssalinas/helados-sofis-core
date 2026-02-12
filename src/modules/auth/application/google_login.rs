use std::sync::Arc;

use crate::modules::auth::domain::entities::{AuthUserInfo, GoogleTokenInfo, LoginResponse};
use crate::modules::users::domain::entities::CreateUserDto;
use crate::modules::users::domain::repositories::UserRepository;
use crate::shared::auth::{create_jwt, Role};
use crate::shared::errors::AppError;

/// Caso de uso: Login con Google OAuth.
///
/// 1. Valida el id_token de Google contra la API de tokeninfo.
/// 2. Si el usuario ya existe en BD, genera JWT propio.
/// 3. Si NO existe, el primer usuario se crea como Owner,
///    los siguientes NO se auto-registran (solo Owner puede crear usuarios).
pub async fn execute(
    user_repo: &Arc<dyn UserRepository>,
    google_client_id: &str,
    jwt_secret: &str,
    token_info: GoogleTokenInfo,
) -> Result<LoginResponse, AppError> {
    // Verificar que el audience coincida con nuestro client_id
    if token_info.aud != google_client_id {
        return Err(AppError::Unauthorized(
            "Token de Google no es para esta aplicación".into(),
        ));
    }

    // Verificar que el email esté verificado
    if token_info.email_verified.as_deref() != Some("true") {
        return Err(AppError::Unauthorized(
            "El email de Google no está verificado".into(),
        ));
    }

    // Buscar usuario en BD por email
    let user = match user_repo.find_by_email(&token_info.email).await? {
        Some(user) => {
            if !user.active {
                return Err(AppError::Forbidden(
                    "Tu cuenta ha sido desactivada. Contacta al dueño.".into(),
                ));
            }
            // Actualizar último login
            user_repo.update_last_login(user.id).await?;
            user
        }
        None => {
            // Si NO hay usuarios en la BD, el primero es Owner (bootstrap)
            let total_users = user_repo.count_by_role(Role::Owner).await?
                + user_repo.count_by_role(Role::Admin).await?;

            if total_users == 0 {
                // Primer usuario del sistema → Owner automáticamente
                let dto = CreateUserDto {
                    email: token_info.email.clone(),
                    display_name: token_info.name.unwrap_or_else(|| token_info.email.clone()),
                    photo_url: token_info.picture.clone(),
                    role: Role::Owner,
                    notes: Some("Primer usuario del sistema (auto-registrado)".into()),
                };
                user_repo.create(&dto, None).await?
            } else {
                return Err(AppError::Forbidden(
                    "No tienes una cuenta registrada. Contacta al dueño para que te cree una."
                        .into(),
                ));
            }
        }
    };

    // Generar JWT propio
    let token = create_jwt(user.id, &user.email, user.role, jwt_secret)?;

    Ok(LoginResponse {
        token,
        user: AuthUserInfo {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
            photo_url: user.photo_url,
            role: user.role,
        },
    })
}
