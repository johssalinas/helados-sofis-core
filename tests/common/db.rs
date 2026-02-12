use sqlx::PgPool;
use std::sync::atomic::{AtomicU32, Ordering};

static DB_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Genera un nombre de base de datos de test único por ejecución.
fn unique_db_name() -> String {
    let id = DB_COUNTER.fetch_add(1, Ordering::SeqCst);
    let pid = std::process::id();
    format!("helados_sofis_test_{pid}_{id}")
}

/// Obtiene la URL base de PostgreSQL (sin nombre de BD).
/// Espera la variable `TEST_DATABASE_URL` con formato:
///   postgres://user:password@host:port
/// Si no existe, usa valores por defecto para desarrollo local.
fn base_database_url() -> String {
    std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432".into())
}

/// Crea una base de datos de test exclusiva, ejecuta las migraciones y
/// devuelve un pool conectado. Cada test obtiene su propia BD aislada.
pub async fn setup_test_db() -> (PgPool, String) {
    dotenvy::dotenv().ok();

    let base_url = base_database_url();
    let db_name = unique_db_name();

    // Conectar a la BD "postgres" para crear la BD de test
    let admin_url = format!("{base_url}/postgres");
    let admin_pool = sqlx::PgPool::connect(&admin_url)
        .await
        .expect("No se pudo conectar a PostgreSQL para crear BD de test");

    // Crear la BD de test
    sqlx::query(&format!("CREATE DATABASE \"{db_name}\""))
        .execute(&admin_pool)
        .await
        .expect("No se pudo crear la BD de test");

    admin_pool.close().await;

    // Conectar a la BD de test
    let test_url = format!("{base_url}/{db_name}");
    let pool = sqlx::PgPool::connect(&test_url)
        .await
        .expect("No se pudo conectar a la BD de test");

    // Ejecutar migraciones
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Error ejecutando migraciones en BD de test");

    (pool, db_name)
}

/// Elimina la base de datos de test tras finalizar las pruebas.
pub async fn teardown_test_db(db_name: &str) {
    let base_url = base_database_url();
    let admin_url = format!("{base_url}/postgres");

    let admin_pool = sqlx::PgPool::connect(&admin_url)
        .await
        .expect("No se pudo conectar para eliminar BD de test");

    // Terminar conexiones activas
    let _ = sqlx::query(&format!(
        "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{db_name}' AND pid <> pg_backend_pid()"
    ))
    .execute(&admin_pool)
    .await;

    // Eliminar la BD
    let _ = sqlx::query(&format!("DROP DATABASE IF EXISTS \"{db_name}\""))
        .execute(&admin_pool)
        .await;

    admin_pool.close().await;
}

/// Helper: crea un AppState de test con la BD proporcionada.
pub fn test_app_state(pool: sqlx::PgPool) -> helados_sofis_core::shared::auth::AppState {
    helados_sofis_core::shared::auth::AppState {
        db: pool,
        config: helados_sofis_core::shared::config::AppConfig {
            database_url: String::new(),
            jwt_secret: "test-secret-key-super-segura-para-tests".into(),
            google_client_id: String::new(),
            server_host: "127.0.0.1".into(),
            server_port: 0,
        },
    }
}

/// Helper: genera un JWT de test válido para un usuario con rol específico.
pub fn test_jwt(
    user_id: uuid::Uuid,
    email: &str,
    role: helados_sofis_core::shared::auth::Role,
) -> String {
    helados_sofis_core::shared::auth::create_jwt(
        user_id,
        email,
        role,
        "test-secret-key-super-segura-para-tests",
    )
    .expect("Error creando JWT de test")
}
