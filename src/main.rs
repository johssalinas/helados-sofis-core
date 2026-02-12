mod shared;
mod modules;

use std::sync::Arc;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use shared::auth::AppState;
use shared::config::AppConfig;
use shared::db::create_pool;

// ─── Repositorios (adaptadores) ─────────────────────────
use modules::users::infrastructure::persistence::postgres_repo::PgUserRepository;
use modules::audit_log::infrastructure::persistence::postgres_repo::PgAuditLogRepository;
use modules::catalog::infrastructure::persistence::postgres_repo::*;
use modules::pricing::infrastructure::persistence::postgres_repo::PgPriceRepository;
use modules::inventory::infrastructure::persistence::postgres_repo::PgInventoryRepository;
use modules::purchases::infrastructure::persistence::postgres_repo::PgPurchaseRepository;
use modules::worker_trips::infrastructure::persistence::postgres_repo::PgWorkerTripRepository;
use modules::worker_payments::infrastructure::persistence::postgres_repo::PgWorkerPaymentRepository;
use modules::cash_register::infrastructure::persistence::postgres_repo::PgCashRegisterRepository;
use modules::local_sales::infrastructure::persistence::postgres_repo::PgLocalSaleRepository;
use modules::owner_sales::infrastructure::persistence::postgres_repo::PgOwnerSaleRepository;
use modules::freezer_transfers::infrastructure::persistence::postgres_repo::PgFreezerTransferRepository;

// ─── Routers (controladores) ────────────────────────────
use modules::users::infrastructure::controllers::http_router as users_router;
use modules::auth::infrastructure::controllers::http_router as auth_router;
use modules::audit_log::infrastructure::controllers::http_router as audit_router;
use modules::catalog::infrastructure::controllers::http_router as catalog_router;
use modules::pricing::infrastructure::controllers::http_router as pricing_router;
use modules::inventory::infrastructure::controllers::http_router as inventory_router;
use modules::purchases::infrastructure::controllers::http_router as purchases_router;
use modules::worker_trips::infrastructure::controllers::http_router as trips_router;
use modules::worker_payments::infrastructure::controllers::http_router as payments_router;
use modules::cash_register::infrastructure::controllers::http_router as cash_router;
use modules::local_sales::infrastructure::controllers::http_router as local_sales_router;
use modules::owner_sales::infrastructure::controllers::http_router as owner_sales_router;
use modules::freezer_transfers::infrastructure::controllers::http_router as transfers_router;

#[tokio::main]
async fn main() {
    // Inicializar dotenv y tracing
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "helados_sofis_core=debug,tower_http=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Cargar configuración
    let config = AppConfig::from_env();
    let pool = create_pool(&config.database_url).await;

    // Ejecutar migraciones
    tracing::info!("Ejecutando migraciones...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Error ejecutando migraciones");
    tracing::info!("Migraciones completadas");

    // Estado compartido de la aplicación
    let app_state = AppState {
        db: pool.clone(),
        config: config.clone(),
    };

    // ─── Instanciar repositorios ────────────────────────
    let user_repo = Arc::new(PgUserRepository::new(pool.clone()))
        as Arc<dyn modules::users::domain::repositories::UserRepository>;

    let audit_repo = Arc::new(PgAuditLogRepository::new(pool.clone()))
        as Arc<dyn modules::audit_log::domain::repositories::AuditLogRepository>;

    let product_repo = Arc::new(PgProductRepository::new(pool.clone()))
        as Arc<dyn modules::catalog::domain::repositories::ProductRepository>;
    let flavor_repo = Arc::new(PgFlavorRepository::new(pool.clone()))
        as Arc<dyn modules::catalog::domain::repositories::FlavorRepository>;
    let provider_repo = Arc::new(PgProviderRepository::new(pool.clone()))
        as Arc<dyn modules::catalog::domain::repositories::ProviderRepository>;
    let worker_repo = Arc::new(PgWorkerRepository::new(pool.clone()))
        as Arc<dyn modules::catalog::domain::repositories::WorkerRepository>;
    let route_repo = Arc::new(PgRouteRepository::new(pool.clone()))
        as Arc<dyn modules::catalog::domain::repositories::RouteRepository>;
    let freezer_repo = Arc::new(PgFreezerRepository::new(pool.clone()))
        as Arc<dyn modules::catalog::domain::repositories::FreezerRepository>;

    let price_repo = Arc::new(PgPriceRepository::new(pool.clone()))
        as Arc<dyn modules::pricing::domain::repositories::PriceRepository>;

    let inventory_repo = Arc::new(PgInventoryRepository::new(pool.clone()))
        as Arc<dyn modules::inventory::domain::repositories::InventoryRepository>;

    let purchase_repo = Arc::new(PgPurchaseRepository::new(pool.clone()))
        as Arc<dyn modules::purchases::domain::repositories::PurchaseRepository>;

    let trip_repo = Arc::new(PgWorkerTripRepository::new(pool.clone()))
        as Arc<dyn modules::worker_trips::domain::repositories::WorkerTripRepository>;

    let payment_repo = Arc::new(PgWorkerPaymentRepository::new(pool.clone()))
        as Arc<dyn modules::worker_payments::domain::repositories::WorkerPaymentRepository>;

    let cash_repo = Arc::new(PgCashRegisterRepository::new(pool.clone()))
        as Arc<dyn modules::cash_register::domain::repositories::CashRegisterRepository>;

    let local_sale_repo = Arc::new(PgLocalSaleRepository::new(pool.clone()))
        as Arc<dyn modules::local_sales::domain::repositories::LocalSaleRepository>;

    let owner_sale_repo = Arc::new(PgOwnerSaleRepository::new(pool.clone()))
        as Arc<dyn modules::owner_sales::domain::repositories::OwnerSaleRepository>;

    let transfer_repo = Arc::new(PgFreezerTransferRepository::new(pool.clone()))
        as Arc<dyn modules::freezer_transfers::domain::repositories::FreezerTransferRepository>;

    // ─── Construir catálogo state ───────────────────────
    let catalog_state = catalog_router::CatalogState {
        app: app_state.clone(),
        products: product_repo,
        flavors: flavor_repo,
        providers: provider_repo,
        workers: worker_repo,
        routes: route_repo,
        freezers: freezer_repo,
    };

    // ─── CORS ───────────────────────────────────────────
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // ─── Router principal ───────────────────────────────
    let app = Router::new()
        .nest("/api/users", users_router::router(app_state.clone(), user_repo.clone()))
        .nest("/api/auth", auth_router::router(app_state.clone(), user_repo.clone()))
        .nest("/api/audit", audit_router::router(app_state.clone(), audit_repo))
        .nest("/api/catalog", catalog_router::router(catalog_state))
        .nest("/api/prices", pricing_router::router(app_state.clone(), price_repo))
        .nest("/api/inventory", inventory_router::router(app_state.clone(), inventory_repo))
        .nest("/api/purchases", purchases_router::router(app_state.clone(), purchase_repo))
        .nest("/api/trips", trips_router::router(app_state.clone(), trip_repo))
        .nest("/api/payments", payments_router::router(app_state.clone(), payment_repo))
        .nest("/api/cash", cash_router::router(app_state.clone(), cash_repo))
        .nest("/api/local-sales", local_sales_router::router(app_state.clone(), local_sale_repo))
        .nest("/api/owner-sales", owner_sales_router::router(app_state.clone(), owner_sale_repo))
        .nest("/api/transfers", transfers_router::router(app_state.clone(), transfer_repo))
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    // ─── Servidor ───────────────────────────────────────
    let addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("No se pudo enlazar la dirección");

    tracing::info!("🍦 Helados Sofis API escuchando en {addr}");

    axum::serve(listener, app)
        .await
        .expect("Error en el servidor");
}
