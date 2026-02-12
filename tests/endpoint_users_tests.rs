mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use std::sync::Arc;
use tower::ServiceExt;

use common::db::{setup_test_db, teardown_test_db, test_app_state, test_jwt};
use common::seed::seed_test_data;
use helados_sofis_core::modules::users::infrastructure::persistence::postgres_repo::PgUserRepository;
use helados_sofis_core::shared::auth::Role;

// ═══════════════════════════════════════════════════════════
// Tests de Integración — Endpoints de Usuarios
// BD real exclusiva por test · Semilla · Patrón AAA
// ═══════════════════════════════════════════════════════════

/// Construye el router de usuarios con BD real de test.
fn build_users_router(
    pool: sqlx::PgPool,
) -> axum::Router {
    let app_state = test_app_state(pool.clone());
    let repo = Arc::new(PgUserRepository::new(pool))
        as Arc<dyn helados_sofis_core::modules::users::domain::repositories::UserRepository>;

    helados_sofis_core::modules::users::infrastructure::controllers::http_router::router(
        app_state, repo,
    )
}

#[tokio::test]
async fn listar_usuarios_con_auth_admin_retorna_200() {
    // Arrange
    let (pool, db_name) = setup_test_db().await;
    let seed = seed_test_data(&pool).await;
    let app = build_users_router(pool.clone());
    let token = test_jwt(seed.admin_id, "admin@test.com", Role::Admin);

    let request = Request::builder()
        .method("GET")
        .uri("/")
        .header("Authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let users: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(users.len() >= 2); // owner + admin de la semilla

    // Cleanup
    pool.close().await;
    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn listar_usuarios_sin_auth_retorna_401() {
    // Arrange
    let (pool, db_name) = setup_test_db().await;
    let _seed = seed_test_data(&pool).await;
    let app = build_users_router(pool.clone());

    let request = Request::builder()
        .method("GET")
        .uri("/")
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Cleanup
    pool.close().await;
    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn crear_usuario_como_owner_retorna_200() {
    // Arrange
    let (pool, db_name) = setup_test_db().await;
    let seed = seed_test_data(&pool).await;
    let app = build_users_router(pool.clone());
    let token = test_jwt(seed.owner_id, "owner@test.com", Role::Owner);

    let body = serde_json::json!({
        "email": "nuevo@test.com",
        "display_name": "Nuevo Usuario",
        "role": "admin"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/")
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let user: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(user["email"], "nuevo@test.com");
    assert_eq!(user["role"], "admin");

    // Cleanup
    pool.close().await;
    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn crear_usuario_como_admin_retorna_403() {
    // Arrange
    let (pool, db_name) = setup_test_db().await;
    let seed = seed_test_data(&pool).await;
    let app = build_users_router(pool.clone());
    let token = test_jwt(seed.admin_id, "admin@test.com", Role::Admin);

    let body = serde_json::json!({
        "email": "otro@test.com",
        "display_name": "No Debería",
        "role": "admin"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/")
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    // Cleanup
    pool.close().await;
    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn crear_usuario_duplicado_retorna_409() {
    // Arrange
    let (pool, db_name) = setup_test_db().await;
    let seed = seed_test_data(&pool).await;
    let app = build_users_router(pool.clone());
    let token = test_jwt(seed.owner_id, "owner@test.com", Role::Owner);

    // Intentar crear con email que ya existe
    let body = serde_json::json!({
        "email": "admin@test.com",
        "display_name": "Duplicado",
        "role": "admin"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/")
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::CONFLICT);

    // Cleanup
    pool.close().await;
    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn obtener_usuario_por_id_exitoso() {
    // Arrange
    let (pool, db_name) = setup_test_db().await;
    let seed = seed_test_data(&pool).await;
    let app = build_users_router(pool.clone());
    let token = test_jwt(seed.owner_id, "owner@test.com", Role::Owner);

    let request = Request::builder()
        .method("GET")
        .uri(&format!("/{}", seed.admin_id))
        .header("Authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let user: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(user["id"], seed.admin_id.to_string());
    assert_eq!(user["email"], "admin@test.com");

    // Cleanup
    pool.close().await;
    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn obtener_usuario_no_existente_retorna_404() {
    // Arrange
    let (pool, db_name) = setup_test_db().await;
    let seed = seed_test_data(&pool).await;
    let app = build_users_router(pool.clone());
    let token = test_jwt(seed.owner_id, "owner@test.com", Role::Owner);

    let fake_id = uuid::Uuid::new_v4();
    let request = Request::builder()
        .method("GET")
        .uri(&format!("/{fake_id}"))
        .header("Authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // Cleanup
    pool.close().await;
    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn actualizar_usuario_como_owner_exitoso() {
    // Arrange
    let (pool, db_name) = setup_test_db().await;
    let seed = seed_test_data(&pool).await;
    let app = build_users_router(pool.clone());
    let token = test_jwt(seed.owner_id, "owner@test.com", Role::Owner);

    let body = serde_json::json!({
        "display_name": "Nombre Actualizado"
    });

    let request = Request::builder()
        .method("PUT")
        .uri(&format!("/{}", seed.admin_id))
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let user: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(user["display_name"], "Nombre Actualizado");

    // Cleanup
    pool.close().await;
    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn endpoint_me_devuelve_usuario_autenticado() {
    // Arrange
    let (pool, db_name) = setup_test_db().await;
    let seed = seed_test_data(&pool).await;
    let app = build_users_router(pool.clone());
    let token = test_jwt(seed.owner_id, "owner@test.com", Role::Owner);

    let request = Request::builder()
        .method("GET")
        .uri("/me")
        .header("Authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let user: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(user["id"], seed.owner_id.to_string());

    // Cleanup
    pool.close().await;
    teardown_test_db(&db_name).await;
}
