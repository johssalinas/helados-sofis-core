use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::shared::errors::AppError;
use crate::modules::catalog::domain::entities::*;
use crate::modules::catalog::domain::repositories::*;

// ═══════════════════════════════════════════════════════════
// Products
// ═══════════════════════════════════════════════════════════

pub struct PgProductRepository {
    pool: PgPool,
}

impl PgProductRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProductRepository for PgProductRepository {
    async fn find_all(&self) -> Result<Vec<Product>, AppError> {
        Ok(sqlx::query_as::<_, Product>("SELECT * FROM products ORDER BY name")
            .fetch_all(&self.pool)
            .await?)
    }

    async fn find_active(&self) -> Result<Vec<Product>, AppError> {
        Ok(sqlx::query_as::<_, Product>(
            "SELECT * FROM products WHERE active = TRUE ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await?)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Product>, AppError> {
        Ok(sqlx::query_as::<_, Product>("SELECT * FROM products WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?)
    }

    async fn create(&self, dto: &CreateProductDto, created_by: Uuid) -> Result<Product, AppError> {
        Ok(sqlx::query_as::<_, Product>(
            "INSERT INTO products (name, created_by) VALUES ($1, $2) RETURNING *",
        )
        .bind(&dto.name)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await?)
    }

    async fn update(&self, id: Uuid, dto: &UpdateProductDto, modified_by: Uuid) -> Result<Product, AppError> {
        Ok(sqlx::query_as::<_, Product>(
            r#"
            UPDATE products SET
                name = COALESCE($1, name),
                active = COALESCE($2, active),
                modified_at = NOW(),
                modified_by = $3
            WHERE id = $4
            RETURNING *
            "#,
        )
        .bind(&dto.name)
        .bind(dto.active)
        .bind(modified_by)
        .bind(id)
        .fetch_one(&self.pool)
        .await?)
    }
}

// ═══════════════════════════════════════════════════════════
// Flavors
// ═══════════════════════════════════════════════════════════

pub struct PgFlavorRepository {
    pool: PgPool,
}

impl PgFlavorRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FlavorRepository for PgFlavorRepository {
    async fn find_all(&self) -> Result<Vec<Flavor>, AppError> {
        Ok(sqlx::query_as::<_, Flavor>("SELECT * FROM flavors ORDER BY name")
            .fetch_all(&self.pool)
            .await?)
    }

    async fn find_by_product(&self, product_id: Uuid) -> Result<Vec<Flavor>, AppError> {
        Ok(sqlx::query_as::<_, Flavor>(
            "SELECT * FROM flavors WHERE product_id = $1 AND active = TRUE ORDER BY name",
        )
        .bind(product_id)
        .fetch_all(&self.pool)
        .await?)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Flavor>, AppError> {
        Ok(sqlx::query_as::<_, Flavor>("SELECT * FROM flavors WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?)
    }

    async fn create(&self, dto: &CreateFlavorDto, created_by: Uuid) -> Result<Flavor, AppError> {
        Ok(sqlx::query_as::<_, Flavor>(
            "INSERT INTO flavors (name, product_id, created_by) VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(&dto.name)
        .bind(dto.product_id)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await?)
    }

    async fn update(&self, id: Uuid, dto: &UpdateFlavorDto) -> Result<Flavor, AppError> {
        Ok(sqlx::query_as::<_, Flavor>(
            r#"
            UPDATE flavors SET
                name = COALESCE($1, name),
                active = COALESCE($2, active)
            WHERE id = $3
            RETURNING *
            "#,
        )
        .bind(&dto.name)
        .bind(dto.active)
        .bind(id)
        .fetch_one(&self.pool)
        .await?)
    }
}

// ═══════════════════════════════════════════════════════════
// Providers
// ═══════════════════════════════════════════════════════════

pub struct PgProviderRepository {
    pool: PgPool,
}

impl PgProviderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProviderRepository for PgProviderRepository {
    async fn find_all(&self) -> Result<Vec<Provider>, AppError> {
        Ok(sqlx::query_as::<_, Provider>("SELECT * FROM providers ORDER BY name")
            .fetch_all(&self.pool)
            .await?)
    }

    async fn find_active(&self) -> Result<Vec<Provider>, AppError> {
        Ok(sqlx::query_as::<_, Provider>(
            "SELECT * FROM providers WHERE active = TRUE ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await?)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Provider>, AppError> {
        Ok(sqlx::query_as::<_, Provider>("SELECT * FROM providers WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?)
    }

    async fn create(&self, dto: &CreateProviderDto, created_by: Uuid) -> Result<Provider, AppError> {
        Ok(sqlx::query_as::<_, Provider>(
            r#"
            INSERT INTO providers (name, contact_info, created_by) 
            VALUES ($1, $2, $3) RETURNING *
            "#,
        )
        .bind(&dto.name)
        .bind(&dto.contact_info)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await?)
    }

    async fn update(&self, id: Uuid, dto: &UpdateProviderDto) -> Result<Provider, AppError> {
        Ok(sqlx::query_as::<_, Provider>(
            r#"
            UPDATE providers SET
                name = COALESCE($1, name),
                contact_info = COALESCE($2, contact_info),
                active = COALESCE($3, active)
            WHERE id = $4
            RETURNING *
            "#,
        )
        .bind(&dto.name)
        .bind(&dto.contact_info)
        .bind(dto.active)
        .bind(id)
        .fetch_one(&self.pool)
        .await?)
    }
}

// ═══════════════════════════════════════════════════════════
// Workers
// ═══════════════════════════════════════════════════════════

pub struct PgWorkerRepository {
    pool: PgPool,
}

impl PgWorkerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WorkerRepository for PgWorkerRepository {
    async fn find_all(&self) -> Result<Vec<Worker>, AppError> {
        Ok(sqlx::query_as::<_, Worker>("SELECT * FROM workers ORDER BY name")
            .fetch_all(&self.pool)
            .await?)
    }

    async fn find_active(&self) -> Result<Vec<Worker>, AppError> {
        Ok(sqlx::query_as::<_, Worker>(
            "SELECT * FROM workers WHERE active = TRUE ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await?)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Worker>, AppError> {
        Ok(sqlx::query_as::<_, Worker>("SELECT * FROM workers WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?)
    }

    async fn create(&self, dto: &CreateWorkerDto, created_by: Uuid) -> Result<Worker, AppError> {
        Ok(sqlx::query_as::<_, Worker>(
            r#"
            INSERT INTO workers (name, phone, address, created_by) 
            VALUES ($1, $2, $3, $4) RETURNING *
            "#,
        )
        .bind(&dto.name)
        .bind(&dto.phone)
        .bind(&dto.address)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await?)
    }

    async fn update(&self, id: Uuid, dto: &UpdateWorkerDto) -> Result<Worker, AppError> {
        Ok(sqlx::query_as::<_, Worker>(
            r#"
            UPDATE workers SET
                name = COALESCE($1, name),
                phone = COALESCE($2, phone),
                address = COALESCE($3, address),
                active = COALESCE($4, active)
            WHERE id = $5
            RETURNING *
            "#,
        )
        .bind(&dto.name)
        .bind(&dto.phone)
        .bind(&dto.address)
        .bind(dto.active)
        .bind(id)
        .fetch_one(&self.pool)
        .await?)
    }
}

// ═══════════════════════════════════════════════════════════
// Routes
// ═══════════════════════════════════════════════════════════

pub struct PgRouteRepository {
    pool: PgPool,
}

impl PgRouteRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RouteRepository for PgRouteRepository {
    async fn find_all(&self) -> Result<Vec<Route>, AppError> {
        Ok(sqlx::query_as::<_, Route>("SELECT * FROM routes ORDER BY usage_count DESC")
            .fetch_all(&self.pool)
            .await?)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Route>, AppError> {
        Ok(sqlx::query_as::<_, Route>("SELECT * FROM routes WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?)
    }

    async fn create(&self, dto: &CreateRouteDto, created_by: Uuid) -> Result<Route, AppError> {
        Ok(sqlx::query_as::<_, Route>(
            "INSERT INTO routes (name, created_by) VALUES ($1, $2) RETURNING *",
        )
        .bind(&dto.name)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await?)
    }
}

// ═══════════════════════════════════════════════════════════
// Freezers
// ═══════════════════════════════════════════════════════════

pub struct PgFreezerRepository {
    pool: PgPool,
}

impl PgFreezerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FreezerRepository for PgFreezerRepository {
    async fn find_all(&self) -> Result<Vec<Freezer>, AppError> {
        Ok(sqlx::query_as::<_, Freezer>("SELECT * FROM freezers ORDER BY number")
            .fetch_all(&self.pool)
            .await?)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Freezer>, AppError> {
        Ok(sqlx::query_as::<_, Freezer>("SELECT * FROM freezers WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?)
    }

    async fn create(&self, dto: &CreateFreezerDto, created_by: Uuid) -> Result<Freezer, AppError> {
        let cap = dto.max_capacity.clone().unwrap_or(serde_json::json!({}));
        Ok(sqlx::query_as::<_, Freezer>(
            r#"
            INSERT INTO freezers (number, max_capacity, created_by) 
            VALUES ($1, $2, $3) RETURNING *
            "#,
        )
        .bind(dto.number)
        .bind(&cap)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await?)
    }

    async fn update(&self, id: Uuid, dto: &UpdateFreezerDto) -> Result<Freezer, AppError> {
        Ok(sqlx::query_as::<_, Freezer>(
            r#"
            UPDATE freezers SET
                max_capacity = COALESCE($1, max_capacity),
                is_on = COALESCE($2, is_on)
            WHERE id = $3
            RETURNING *
            "#,
        )
        .bind(&dto.max_capacity)
        .bind(dto.is_on)
        .bind(id)
        .fetch_one(&self.pool)
        .await?)
    }

    async fn toggle_power(&self, id: Uuid) -> Result<Freezer, AppError> {
        Ok(sqlx::query_as::<_, Freezer>(
            r#"
            UPDATE freezers SET 
                is_on = NOT is_on,
                last_toggle = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?)
    }
}
