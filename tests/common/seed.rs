#![allow(dead_code)]

use sqlx::PgPool;
use uuid::Uuid;

/// Datos sembrados para las pruebas.
#[derive(Debug, Clone)]
pub struct SeedData {
    pub owner_id: Uuid,
    pub admin_id: Uuid,
    pub product_id: Uuid,
    pub flavor_id: Uuid,
    pub provider_id: Uuid,
    pub worker_id: Uuid,
    pub route_id: Uuid,
    pub freezer_id: Uuid,
}

/// Inserta datos semilla en la BD de test.
/// Crea un owner, un admin, un producto, un sabor, un proveedor,
/// un trabajador, una ruta y un congelador.
pub async fn seed_test_data(pool: &PgPool) -> SeedData {
    let owner_id = Uuid::new_v4();
    let admin_id = Uuid::new_v4();

    // ─── Usuarios ───────────────────────────────
    sqlx::query(
        r#"INSERT INTO users (id, email, display_name, role, active)
           VALUES ($1, 'owner@test.com', 'Owner Test', 'owner', TRUE)"#,
    )
    .bind(owner_id)
    .execute(pool)
    .await
    .expect("Error sembrando owner");

    sqlx::query(
        r#"INSERT INTO users (id, email, display_name, role, active, created_by)
           VALUES ($1, 'admin@test.com', 'Admin Test', 'admin', TRUE, $2)"#,
    )
    .bind(admin_id)
    .bind(owner_id)
    .execute(pool)
    .await
    .expect("Error sembrando admin");

    // ─── Producto ───────────────────────────────
    let product_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO products (id, name, active, created_by)
           VALUES ($1, 'Paleta', TRUE, $2)"#,
    )
    .bind(product_id)
    .bind(owner_id)
    .execute(pool)
    .await
    .expect("Error sembrando producto");

    // ─── Sabor ──────────────────────────────────
    let flavor_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO flavors (id, name, product_id, active, created_by)
           VALUES ($1, 'Chocolate', $2, TRUE, $3)"#,
    )
    .bind(flavor_id)
    .bind(product_id)
    .bind(owner_id)
    .execute(pool)
    .await
    .expect("Error sembrando sabor");

    // ─── Proveedor ──────────────────────────────
    let provider_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO providers (id, name, contact_info, active, created_by)
           VALUES ($1, 'Proveedor Test', 'Tel: 555-0001', TRUE, $2)"#,
    )
    .bind(provider_id)
    .bind(owner_id)
    .execute(pool)
    .await
    .expect("Error sembrando proveedor");

    // ─── Trabajador ─────────────────────────────
    let worker_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO workers (id, name, phone, active, created_by)
           VALUES ($1, 'Juan Pérez', '555-1234', TRUE, $2)"#,
    )
    .bind(worker_id)
    .bind(owner_id)
    .execute(pool)
    .await
    .expect("Error sembrando trabajador");

    // ─── Ruta ───────────────────────────────────
    let route_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO routes (id, name, created_by)
           VALUES ($1, 'Ruta Centro', $2)"#,
    )
    .bind(route_id)
    .bind(owner_id)
    .execute(pool)
    .await
    .expect("Error sembrando ruta");

    // ─── Congelador ─────────────────────────────
    let freezer_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO freezers (id, number, max_capacity, is_on, created_by)
           VALUES ($1, 1, '{"paletas": 500}', TRUE, $2)"#,
    )
    .bind(freezer_id)
    .bind(owner_id)
    .execute(pool)
    .await
    .expect("Error sembrando congelador");

    // ─── Precio ─────────────────────────────────
    sqlx::query(
        r#"INSERT INTO price_history 
           (product_id, flavor_id, provider_id, cost_price, price_base, 
            price_route, price_local, commission, effective_date, created_by)
           VALUES ($1, $2, $3, 5.00, 10.00, 12.00, 15.00, 2.00, NOW(), $4)"#,
    )
    .bind(product_id)
    .bind(flavor_id)
    .bind(provider_id)
    .bind(owner_id)
    .execute(pool)
    .await
    .expect("Error sembrando precio");

    // ─── Inventario ─────────────────────────────
    sqlx::query(
        r#"INSERT INTO inventory 
           (freezer_id, product_id, flavor_id, provider_id, quantity, 
            min_stock_alert, is_deformed, updated_by)
           VALUES ($1, $2, $3, $4, 100, 20, FALSE, $5)"#,
    )
    .bind(freezer_id)
    .bind(product_id)
    .bind(flavor_id)
    .bind(provider_id)
    .bind(owner_id)
    .execute(pool)
    .await
    .expect("Error sembrando inventario");

    SeedData {
        owner_id,
        admin_id,
        product_id,
        flavor_id,
        provider_id,
        worker_id,
        route_id,
        freezer_id,
    }
}
