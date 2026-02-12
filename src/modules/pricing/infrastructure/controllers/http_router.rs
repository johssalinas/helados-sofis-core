use std::sync::Arc;

use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::modules::pricing::application::manage_prices;
use crate::modules::pricing::domain::entities::{CreatePriceDto, PriceHistory};
use crate::modules::pricing::domain::repositories::PriceRepository;
use crate::shared::auth::{AppState, AuthUser, Role};
use crate::shared::errors::AppError;

#[derive(OpenApi)]
#[openapi(
    paths(
        list_current_handler,
        create_handler,
        current_price_handler,
        history_handler
    ),
    components(schemas(
        crate::modules::pricing::domain::entities::PriceHistory,
        crate::modules::pricing::domain::entities::CreatePriceDto,
    ))
)]
pub struct PricingApiDoc;

#[derive(Clone)]
pub struct PricingState {
    pub app: AppState,
    pub repo: Arc<dyn PriceRepository>,
}

impl axum::extract::FromRef<PricingState> for AppState {
    fn from_ref(s: &PricingState) -> AppState {
        s.app.clone()
    }
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct PriceLookupQuery {
    pub product_id: Uuid,
    pub flavor_id: Uuid,
    pub provider_id: Uuid,
}

pub fn router(app_state: AppState, repo: Arc<dyn PriceRepository>) -> Router {
    let state = PricingState {
        app: app_state,
        repo,
    };

    Router::new()
        .route("/", get(list_current_handler).post(create_handler))
        .route("/current", get(current_price_handler))
        .route("/history", get(history_handler))
        .with_state(state)
}

/// GET /prices — Listar todos los precios actuales.
#[utoipa::path(
    get, path = "/", tag = "Precios",
    responses((status = 200, description = "Precios actuales", body = Vec<PriceHistory>)),
    security(("bearer_auth" = []))
)]
async fn list_current_handler(
    auth: AuthUser,
    State(state): State<PricingState>,
) -> Result<Json<Vec<PriceHistory>>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(manage_prices::list_current_prices(&state.repo).await?))
}

/// POST /prices — Crear un nuevo precio (Temporal Data: nunca updatear).
#[utoipa::path(
    post, path = "/", tag = "Precios",
    request_body = CreatePriceDto,
    responses((status = 200, description = "Precio creado", body = PriceHistory)),
    security(("bearer_auth" = []))
)]
async fn create_handler(
    auth: AuthUser,
    State(state): State<PricingState>,
    Json(dto): Json<CreatePriceDto>,
) -> Result<Json<PriceHistory>, AppError> {
    auth.require_owner()?;
    Ok(Json(
        manage_prices::create_price(&state.repo, dto, auth.user_id()).await?,
    ))
}

/// GET /prices/current?product_id=...&flavor_id=...&provider_id=...
#[utoipa::path(
    get, path = "/current", tag = "Precios",
    params(PriceLookupQuery),
    responses(
        (status = 200, description = "Precio actual", body = PriceHistory),
        (status = 404, description = "No encontrado")
    ),
    security(("bearer_auth" = []))
)]
async fn current_price_handler(
    auth: AuthUser,
    State(state): State<PricingState>,
    Query(q): Query<PriceLookupQuery>,
) -> Result<Json<PriceHistory>, AppError> {
    auth.require_role(Role::Admin)?;
    Ok(Json(
        manage_prices::get_current_price(&state.repo, q.product_id, q.flavor_id, q.provider_id)
            .await?,
    ))
}

/// GET /prices/history?product_id=...&flavor_id=...&provider_id=...
#[utoipa::path(
    get, path = "/history", tag = "Precios",
    params(PriceLookupQuery),
    responses((status = 200, description = "Historial de precios", body = Vec<PriceHistory>)),
    security(("bearer_auth" = []))
)]
async fn history_handler(
    auth: AuthUser,
    State(state): State<PricingState>,
    Query(q): Query<PriceLookupQuery>,
) -> Result<Json<Vec<PriceHistory>>, AppError> {
    auth.require_owner()?;
    Ok(Json(
        manage_prices::get_price_history(&state.repo, q.product_id, q.flavor_id, q.provider_id)
            .await?,
    ))
}
