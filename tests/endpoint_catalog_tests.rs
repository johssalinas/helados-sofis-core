mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use std::sync::Arc;
use tower::ServiceExt;

use common::db::{setup_test_db, teardown_test_db, test_app_state, test_jwt};
use common::seed::seed_test_data;
use helados_sofis_core::modules::catalog::domain::entities::*;
use helados_sofis_core::modules::catalog::infrastructure::controllers::http_router::{
    self, CatalogState,
};
use helados_sofis_core::modules::catalog::infrastructure::persistence::postgres_repo::*;
use helados_sofis_core::shared::auth::Role;

// ═══════════════════════════════════════════════════════════
// Tests de Integración — Endpoints de Catálogo
// BD real exclusiva por test · Semilla · Patrón AAA
// ═══════════════════════════════════════════════════════════

fn build_catalog_router(pool: sqlx::PgPool) -> axum::Router {
    let app_state = test_app_state(pool.clone());

    let catalog_state = CatalogState {
        app: app_state,
        products: Arc::new(PgProductRepository::new(pool.clone())),
        flavors: Arc::new(PgFlavorRepository::new(pool.clone())),
        providers: Arc::new(PgProviderRepository::new(pool.clone())),
        workers: Arc::new(PgWorkerRepository::new(pool.clone())),
        routes: Arc::new(PgRouteRepository::new(pool.clone())),
        freezers: Arc::new(PgFreezerRepository::new(pool)),
    };

    http_router::router(catalog_state)
}

// ─── Products ───────────────────────────────────────────

#[tokio::test]
async fn listar_productos_con_auth_retorna_200() {
    // Arrange
    let (pool, db_name) = setup_test_db().await;
    let seed = seed_test_data(&pool).await;
    let app = build_catalog_router(pool.clone());
    let token = test_jwt(seed.admin_id, "admin@test.com", Role::Admin);

    let request = Request::builder()
        .method("GET")
        .uri("/products")
        .header("Authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let products: Vec<Product> = serde_json::from_slice(&body).unwrap();
    assert!(!products.is_empty());
    assert_eq!(products[0].name, "Paleta");

    // Cleanup
    pool.close().await;
    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn crear_producto_como_admin_retorna_200() {
    // Arrange
    let (pool, db_name) = setup_test_db().await;
    let seed = seed_test_data(&pool).await;
    let app = build_catalog_router(pool.clone());
    let token = test_jwt(seed.admin_id, "admin@test.com", Role::Admin);

    let body = serde_json::json!({ "name": "Bolis" });

    let request = Request::builder()
        .method("POST")
        .uri("/products")
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let product: Product = serde_json::from_slice(&body).unwrap();
    assert_eq!(product.name, "Bolis");
    assert!(product.active);

    // Cleanup
    pool.close().await;
    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn obtener_producto_por_id() {
    // Arrange
    let (pool, db_name) = setup_test_db().await;
    let seed = seed_test_data(&pool).await;
    let app = build_catalog_router(pool.clone());
    let token = test_jwt(seed.admin_id, "admin@test.com", Role::Admin);

    let request = Request::builder()
        .method("GET")
        .uri(&format!("/products/{}", seed.product_id))
        .header("Authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let product: Product = serde_json::from_slice(&body).unwrap();
    assert_eq!(product.id, seed.product_id);

    // Cleanup
    pool.close().await;
    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn actualizar_producto_como_owner_retorna_200() {
    // Arrange
    let (pool, db_name) = setup_test_db().await;
    let seed = seed_test_data(&pool).await;
    let app = build_catalog_router(pool.clone());
    let token = test_jwt(seed.owner_id, "owner@test.com", Role::Owner);

    let body = serde_json::json!({ "name": "Paleta Renombrada" });

    let request = Request::builder()
        .method("PUT")
        .uri(&format!("/products/{}", seed.product_id))
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let product: Product = serde_json::from_slice(&body).unwrap();
    assert_eq!(product.name, "Paleta Renombrada");

    // Cleanup
    pool.close().await;
    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn actualizar_producto_como_admin_retorna_403() {
    // Arrange
    let (pool, db_name) = setup_test_db().await;
    let seed = seed_test_data(&pool).await;
    let app = build_catalog_router(pool.clone());
    let token = test_jwt(seed.admin_id, "admin@test.com", Role::Admin);

    let body = serde_json::json!({ "name": "No Permitido" });

    let request = Request::builder()
        .method("PUT")
        .uri(&format!("/products/{}", seed.product_id))
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

// ─── Flavors ────────────────────────────────────────────

#[tokio::test]
async fn listar_sabores_retorna_200() {
    // Arrange
    let (pool, db_name) = setup_test_db().await;
    let seed = seed_test_data(&pool).await;
    let app = build_catalog_router(pool.clone());
    let token = test_jwt(seed.admin_id, "admin@test.com", Role::Admin);

    let request = Request::builder()
        .method("GET")
        .uri("/flavors")
        .header("Authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let flavors: Vec<Flavor> = serde_json::from_slice(&body).unwrap();
    assert!(!flavors.is_empty());

    // Cleanup
    pool.close().await;
    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn listar_sabores_por_producto_retorna_200() {
    // Arrange
    let (pool, db_name) = setup_test_db().await;
    let seed = seed_test_data(&pool).await;
    let app = build_catalog_router(pool.clone());
    let token = test_jwt(seed.admin_id, "admin@test.com", Role::Admin);

    let request = Request::builder()
        .method("GET")
        .uri(&format!("/products/{}/flavors", seed.product_id))
        .header("Authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let flavors: Vec<Flavor> = serde_json::from_slice(&body).unwrap();
    assert_eq!(flavors.len(), 1);
    assert_eq!(flavors[0].name, "Chocolate");

    // Cleanup
    pool.close().await;
    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn crear_sabor_como_admin_retorna_200() {
    // Arrange
    let (pool, db_name) = setup_test_db().await;
    let seed = seed_test_data(&pool).await;
    let app = build_catalog_router(pool.clone());
    let token = test_jwt(seed.admin_id, "admin@test.com", Role::Admin);

    let body = serde_json::json!({
        "name": "Vainilla",
        "product_id": seed.product_id
    });

    let request = Request::builder()
        .method("POST")
        .uri("/flavors")
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let flavor: Flavor = serde_json::from_slice(&body).unwrap();
    assert_eq!(flavor.name, "Vainilla");
    assert_eq!(flavor.product_id, seed.product_id);

    // Cleanup
    pool.close().await;
    teardown_test_db(&db_name).await;
}

// ─── Providers ──────────────────────────────────────────

#[tokio::test]
async fn listar_proveedores_retorna_200() {
    // Arrange
    let (pool, db_name) = setup_test_db().await;
    let seed = seed_test_data(&pool).await;
    let app = build_catalog_router(pool.clone());
    let token = test_jwt(seed.admin_id, "admin@test.com", Role::Admin);

    let request = Request::builder()
        .method("GET")
        .uri("/providers")
        .header("Authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let providers: Vec<Provider> = serde_json::from_slice(&body).unwrap();
    assert!(!providers.is_empty());

    // Cleanup
    pool.close().await;
    teardown_test_db(&db_name).await;
}

// ─── Workers ────────────────────────────────────────────

#[tokio::test]
async fn listar_trabajadores_retorna_200() {
    // Arrange
    let (pool, db_name) = setup_test_db().await;
    let seed = seed_test_data(&pool).await;
    let app = build_catalog_router(pool.clone());
    let token = test_jwt(seed.admin_id, "admin@test.com", Role::Admin);

    let request = Request::builder()
        .method("GET")
        .uri("/workers")
        .header("Authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let workers: Vec<Worker> = serde_json::from_slice(&body).unwrap();
    assert!(!workers.is_empty());
    assert_eq!(workers[0].name, "Juan Pérez");

    // Cleanup
    pool.close().await;
    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn crear_trabajador_como_admin_retorna_200() {
    // Arrange
    let (pool, db_name) = setup_test_db().await;
    let seed = seed_test_data(&pool).await;
    let app = build_catalog_router(pool.clone());
    let token = test_jwt(seed.admin_id, "admin@test.com", Role::Admin);

    let body = serde_json::json!({
        "name": "María López",
        "phone": "555-4444"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/workers")
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let worker: Worker = serde_json::from_slice(&body).unwrap();
    assert_eq!(worker.name, "María López");

    // Cleanup
    pool.close().await;
    teardown_test_db(&db_name).await;
}

// ─── Routes ─────────────────────────────────────────────

#[tokio::test]
async fn listar_rutas_retorna_200() {
    // Arrange
    let (pool, db_name) = setup_test_db().await;
    let seed = seed_test_data(&pool).await;
    let app = build_catalog_router(pool.clone());
    let token = test_jwt(seed.admin_id, "admin@test.com", Role::Admin);

    let request = Request::builder()
        .method("GET")
        .uri("/routes")
        .header("Authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let routes: Vec<Route> = serde_json::from_slice(&body).unwrap();
    assert!(!routes.is_empty());

    // Cleanup
    pool.close().await;
    teardown_test_db(&db_name).await;
}

// ─── Freezers ───────────────────────────────────────────

#[tokio::test]
async fn listar_congeladores_retorna_200() {
    // Arrange
    let (pool, db_name) = setup_test_db().await;
    let seed = seed_test_data(&pool).await;
    let app = build_catalog_router(pool.clone());
    let token = test_jwt(seed.admin_id, "admin@test.com", Role::Admin);

    let request = Request::builder()
        .method("GET")
        .uri("/freezers")
        .header("Authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let freezers: Vec<Freezer> = serde_json::from_slice(&body).unwrap();
    assert!(!freezers.is_empty());

    // Cleanup
    pool.close().await;
    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn toggle_congelador_retorna_200() {
    // Arrange
    let (pool, db_name) = setup_test_db().await;
    let seed = seed_test_data(&pool).await;
    let app = build_catalog_router(pool.clone());
    let token = test_jwt(seed.admin_id, "admin@test.com", Role::Admin);

    let request = Request::builder()
        .method("POST")
        .uri(&format!("/freezers/{}/toggle", seed.freezer_id))
        .header("Authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();

    // Act
    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let freezer: Freezer = serde_json::from_slice(&body).unwrap();
    // Se invirtió el estado (era true, ahora false)
    assert!(!freezer.is_on);

    // Cleanup
    pool.close().await;
    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn todos_los_endpoints_sin_auth_retornan_401() {
    // Arrange
    let (pool, db_name) = setup_test_db().await;
    let _seed = seed_test_data(&pool).await;
    let app = build_catalog_router(pool.clone());

    let uris = vec![
        ("GET", "/products"),
        ("GET", "/flavors"),
        ("GET", "/providers"),
        ("GET", "/workers"),
        ("GET", "/routes"),
        ("GET", "/freezers"),
    ];

    for (method, uri) in uris {
        let app_clone = app.clone();
        let request = Request::builder()
            .method(method)
            .uri(uri)
            .body(Body::empty())
            .unwrap();

        // Act
        let response = app_clone.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "Endpoint {method} {uri} debería retornar 401 sin auth"
        );
    }

    // Cleanup
    pool.close().await;
    teardown_test_db(&db_name).await;
}
