use std::sync::Arc;

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use utoipa::OpenApi;
use uuid::Uuid;

use axum::extract::FromRef;

use crate::modules::users::application::{create_user, get_user, list_users, update_user};
use crate::modules::users::domain::entities::{CreateUserDto, UpdateUserDto, UserResponse};
use crate::modules::users::domain::repositories::UserRepository;
use crate::shared::auth::{AppState, AuthUser};
use crate::shared::errors::AppError;

#[derive(OpenApi)]
#[openapi(
    paths(list_handler, create_handler, get_handler, update_handler, me_handler),
    components(schemas(
        crate::modules::users::domain::entities::UserResponse,
        crate::modules::users::domain::entities::CreateUserDto,
        crate::modules::users::domain::entities::UpdateUserDto,
        crate::shared::auth::Role,
    ))
)]
pub struct UsersApiDoc;

/// Estado específico del módulo de usuarios compartido con los handlers.
#[derive(Clone)]
pub struct UsersState {
    pub app: AppState,
    pub repo: Arc<dyn UserRepository>,
}

impl FromRef<UsersState> for AppState {
    fn from_ref(s: &UsersState) -> AppState {
        s.app.clone()
    }
}

/// Crea el router del módulo de usuarios.
pub fn router(app_state: AppState, repo: Arc<dyn UserRepository>) -> Router {
    let state = UsersState {
        app: app_state,
        repo,
    };

    Router::new()
        .route("/", get(list_handler).post(create_handler))
        .route("/{id}", get(get_handler).put(update_handler))
        .route("/me", get(me_handler))
        .with_state(state)
}

// ─── Handlers ──────────────────────────────────────────

/// GET /users — Listar todos los usuarios (solo admin+).
#[utoipa::path(
    get,
    path = "/",
    tag = "Usuarios",
    responses(
        (status = 200, description = "Lista de usuarios", body = Vec<UserResponse>),
        (status = 401, description = "No autorizado")
    ),
    security(("bearer_auth" = []))
)]
async fn list_handler(
    auth: AuthUser,
    State(state): State<UsersState>,
) -> Result<Json<Vec<UserResponse>>, AppError> {
    auth.require_role(crate::shared::auth::Role::Admin)?;
    let users = list_users::all(&state.repo).await?;
    let response: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();
    Ok(Json(response))
}

/// POST /users — Crear un nuevo usuario (solo owner).
#[utoipa::path(
    post,
    path = "/",
    tag = "Usuarios",
    request_body = CreateUserDto,
    responses(
        (status = 200, description = "Usuario creado", body = UserResponse),
        (status = 401, description = "No autorizado")
    ),
    security(("bearer_auth" = []))
)]
async fn create_handler(
    auth: AuthUser,
    State(state): State<UsersState>,
    Json(dto): Json<CreateUserDto>,
) -> Result<Json<UserResponse>, AppError> {
    auth.require_owner()?;
    let user = create_user::execute(&state.repo, dto, auth.user_id()).await?;
    Ok(Json(UserResponse::from(user)))
}

/// GET /users/:id — Obtener un usuario por ID.
#[utoipa::path(
    get,
    path = "/{id}",
    tag = "Usuarios",
    params(("id" = Uuid, Path, description = "ID del usuario")),
    responses(
        (status = 200, description = "Usuario encontrado", body = UserResponse),
        (status = 404, description = "Usuario no encontrado")
    ),
    security(("bearer_auth" = []))
)]
async fn get_handler(
    auth: AuthUser,
    State(state): State<UsersState>,
    Path(id): Path<Uuid>,
) -> Result<Json<UserResponse>, AppError> {
    auth.require_role(crate::shared::auth::Role::Admin)?;
    let user = get_user::by_id(&state.repo, id).await?;
    Ok(Json(UserResponse::from(user)))
}

/// PUT /users/:id — Actualizar un usuario (solo owner).
#[utoipa::path(
    put,
    path = "/{id}",
    tag = "Usuarios",
    params(("id" = Uuid, Path, description = "ID del usuario")),
    request_body = UpdateUserDto,
    responses(
        (status = 200, description = "Usuario actualizado", body = UserResponse),
        (status = 404, description = "Usuario no encontrado")
    ),
    security(("bearer_auth" = []))
)]
async fn update_handler(
    auth: AuthUser,
    State(state): State<UsersState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateUserDto>,
) -> Result<Json<UserResponse>, AppError> {
    auth.require_owner()?;
    let user = update_user::execute(&state.repo, id, dto).await?;
    Ok(Json(UserResponse::from(user)))
}

/// GET /users/me — Obtener datos del usuario autenticado.
#[utoipa::path(
    get,
    path = "/me",
    tag = "Usuarios",
    responses(
        (status = 200, description = "Datos del usuario autenticado", body = UserResponse),
        (status = 401, description = "No autorizado")
    ),
    security(("bearer_auth" = []))
)]
async fn me_handler(
    auth: AuthUser,
    State(state): State<UsersState>,
) -> Result<Json<UserResponse>, AppError> {
    let user = get_user::by_id(&state.repo, auth.user_id()).await?;
    Ok(Json(UserResponse::from(user)))
}
