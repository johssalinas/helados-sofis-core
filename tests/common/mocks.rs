#![allow(dead_code)]

use async_trait::async_trait;
use chrono::Utc;
use mockall::mock;
use rust_decimal::Decimal;
use std::sync::Mutex;
use uuid::Uuid;

use helados_sofis_core::modules::catalog::domain::entities::*;
use helados_sofis_core::modules::catalog::domain::repositories::*;
use helados_sofis_core::modules::inventory::domain::entities::*;
use helados_sofis_core::modules::inventory::domain::repositories::InventoryRepository;
use helados_sofis_core::modules::pricing::domain::entities::*;
use helados_sofis_core::modules::pricing::domain::repositories::PriceRepository;
use helados_sofis_core::modules::users::domain::entities::*;
use helados_sofis_core::modules::users::domain::repositories::UserRepository;
use helados_sofis_core::shared::auth::Role;
use helados_sofis_core::shared::errors::AppError;

// ═══════════════════════════════════════════════════════════
// Mocks de repositorios usando mockall
// ═══════════════════════════════════════════════════════════

mock! {
    pub UserRepo {}

    #[async_trait]
    impl UserRepository for UserRepo {
        async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError>;
        async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
        async fn find_all(&self) -> Result<Vec<User>, AppError>;
        async fn find_active(&self) -> Result<Vec<User>, AppError>;
        async fn create(&self, dto: &CreateUserDto, created_by: Option<Uuid>) -> Result<User, AppError>;
        async fn update(&self, id: Uuid, dto: &UpdateUserDto) -> Result<User, AppError>;
        async fn update_last_login(&self, id: Uuid) -> Result<(), AppError>;
        async fn has_role(&self, id: Uuid, required: Role) -> Result<bool, AppError>;
        async fn count_by_role(&self, role: Role) -> Result<i64, AppError>;
    }
}

mock! {
    pub ProductRepo {}

    #[async_trait]
    impl ProductRepository for ProductRepo {
        async fn find_all(&self) -> Result<Vec<Product>, AppError>;
        async fn find_active(&self) -> Result<Vec<Product>, AppError>;
        async fn find_by_id(&self, id: Uuid) -> Result<Option<Product>, AppError>;
        async fn create(&self, dto: &CreateProductDto, created_by: Uuid) -> Result<Product, AppError>;
        async fn update(&self, id: Uuid, dto: &UpdateProductDto, modified_by: Uuid) -> Result<Product, AppError>;
    }
}

mock! {
    pub FlavorRepo {}

    #[async_trait]
    impl FlavorRepository for FlavorRepo {
        async fn find_all(&self) -> Result<Vec<Flavor>, AppError>;
        async fn find_by_product(&self, product_id: Uuid) -> Result<Vec<Flavor>, AppError>;
        async fn find_by_id(&self, id: Uuid) -> Result<Option<Flavor>, AppError>;
        async fn create(&self, dto: &CreateFlavorDto, created_by: Uuid) -> Result<Flavor, AppError>;
        async fn update(&self, id: Uuid, dto: &UpdateFlavorDto) -> Result<Flavor, AppError>;
    }
}

mock! {
    pub ProviderRepo {}

    #[async_trait]
    impl ProviderRepository for ProviderRepo {
        async fn find_all(&self) -> Result<Vec<Provider>, AppError>;
        async fn find_active(&self) -> Result<Vec<Provider>, AppError>;
        async fn find_by_id(&self, id: Uuid) -> Result<Option<Provider>, AppError>;
        async fn create(&self, dto: &CreateProviderDto, created_by: Uuid) -> Result<Provider, AppError>;
        async fn update(&self, id: Uuid, dto: &UpdateProviderDto) -> Result<Provider, AppError>;
    }
}

mock! {
    pub WorkerRepo {}

    #[async_trait]
    impl WorkerRepository for WorkerRepo {
        async fn find_all(&self) -> Result<Vec<Worker>, AppError>;
        async fn find_active(&self) -> Result<Vec<Worker>, AppError>;
        async fn find_by_id(&self, id: Uuid) -> Result<Option<Worker>, AppError>;
        async fn create(&self, dto: &CreateWorkerDto, created_by: Uuid) -> Result<Worker, AppError>;
        async fn update(&self, id: Uuid, dto: &UpdateWorkerDto) -> Result<Worker, AppError>;
    }
}

mock! {
    pub RouteRepo {}

    #[async_trait]
    impl RouteRepository for RouteRepo {
        async fn find_all(&self) -> Result<Vec<Route>, AppError>;
        async fn find_by_id(&self, id: Uuid) -> Result<Option<Route>, AppError>;
        async fn create(&self, dto: &CreateRouteDto, created_by: Uuid) -> Result<Route, AppError>;
    }
}

mock! {
    pub FreezerRepo {}

    #[async_trait]
    impl FreezerRepository for FreezerRepo {
        async fn find_all(&self) -> Result<Vec<Freezer>, AppError>;
        async fn find_by_id(&self, id: Uuid) -> Result<Option<Freezer>, AppError>;
        async fn create(&self, dto: &CreateFreezerDto, created_by: Uuid) -> Result<Freezer, AppError>;
        async fn update(&self, id: Uuid, dto: &UpdateFreezerDto) -> Result<Freezer, AppError>;
        async fn toggle_power(&self, id: Uuid) -> Result<Freezer, AppError>;
    }
}

mock! {
    pub PriceRepo {}

    #[async_trait]
    impl PriceRepository for PriceRepo {
        async fn get_current_price(
            &self,
            product_id: Uuid,
            flavor_id: Uuid,
            provider_id: Uuid,
        ) -> Result<Option<PriceHistory>, AppError>;

        async fn get_price_at(
            &self,
            product_id: Uuid,
            flavor_id: Uuid,
            provider_id: Uuid,
            date: chrono::DateTime<Utc>,
        ) -> Result<Option<PriceHistory>, AppError>;

        async fn create(
            &self,
            dto: &CreatePriceDto,
            created_by: Uuid,
        ) -> Result<PriceHistory, AppError>;

        async fn get_history(
            &self,
            product_id: Uuid,
            flavor_id: Uuid,
            provider_id: Uuid,
        ) -> Result<Vec<PriceHistory>, AppError>;

        async fn list_current_prices(&self) -> Result<Vec<PriceHistory>, AppError>;
    }
}

/// Mock manual de InventoryRepository.
/// Necesario porque los métodos _tx usan lifetimes incompatibles con mockall.
pub struct MockInventoryRepo {
    pub find_all_result: Mutex<Option<Result<Vec<InventoryItem>, AppError>>>,
    pub find_by_freezer_result: Mutex<Option<Result<Vec<InventoryItem>, AppError>>>,
    pub find_by_id_result: Mutex<Option<Result<Option<InventoryItem>, AppError>>>,
    pub find_sellable_result: Mutex<Option<Result<Vec<InventoryItem>, AppError>>>,
    pub find_low_stock_result: Mutex<Option<Result<Vec<InventoryItem>, AppError>>>,
    pub find_worker_deformed_result: Mutex<Option<Result<Vec<InventoryItem>, AppError>>>,
    pub add_stock_result: Mutex<Option<Result<InventoryItem, AppError>>>,
    pub update_alert_result: Mutex<Option<Result<InventoryItem, AppError>>>,
}

impl MockInventoryRepo {
    pub fn new() -> Self {
        Self {
            find_all_result: Mutex::new(None),
            find_by_freezer_result: Mutex::new(None),
            find_by_id_result: Mutex::new(None),
            find_sellable_result: Mutex::new(None),
            find_low_stock_result: Mutex::new(None),
            find_worker_deformed_result: Mutex::new(None),
            add_stock_result: Mutex::new(None),
            update_alert_result: Mutex::new(None),
        }
    }
}

#[async_trait]
impl InventoryRepository for MockInventoryRepo {
    async fn find_all(&self) -> Result<Vec<InventoryItem>, AppError> {
        self.find_all_result.lock().unwrap().take().unwrap_or(Ok(vec![]))
    }
    async fn find_by_freezer(&self, _freezer_id: Uuid) -> Result<Vec<InventoryItem>, AppError> {
        self.find_by_freezer_result.lock().unwrap().take().unwrap_or(Ok(vec![]))
    }
    async fn find_by_id(&self, _id: Uuid) -> Result<Option<InventoryItem>, AppError> {
        self.find_by_id_result.lock().unwrap().take().unwrap_or(Ok(None))
    }
    async fn find_sellable(&self) -> Result<Vec<InventoryItem>, AppError> {
        self.find_sellable_result.lock().unwrap().take().unwrap_or(Ok(vec![]))
    }
    async fn find_low_stock(&self) -> Result<Vec<InventoryItem>, AppError> {
        self.find_low_stock_result.lock().unwrap().take().unwrap_or(Ok(vec![]))
    }
    async fn find_worker_deformed(&self, _worker_id: Uuid) -> Result<Vec<InventoryItem>, AppError> {
        self.find_worker_deformed_result.lock().unwrap().take().unwrap_or(Ok(vec![]))
    }
    async fn add_stock(
        &self,
        _freezer_id: Uuid,
        _product_id: Uuid,
        _flavor_id: Uuid,
        _provider_id: Uuid,
        _quantity: i32,
        _updated_by: Uuid,
    ) -> Result<InventoryItem, AppError> {
        self.add_stock_result.lock().unwrap().take()
            .unwrap_or(Err(AppError::Internal("Mock no configurado".into())))
    }
    async fn subtract_stock_tx(
        &self,
        _tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        _inventory_id: Uuid,
        _quantity: i32,
        _updated_by: Uuid,
    ) -> Result<(), AppError> {
        Ok(())
    }
    async fn add_deformed_tx(
        &self,
        _tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        _freezer_id: Uuid,
        _product_id: Uuid,
        _flavor_id: Uuid,
        _provider_id: Uuid,
        _quantity: i32,
        _worker_id: Uuid,
        _updated_by: Uuid,
    ) -> Result<InventoryItem, AppError> {
        Err(AppError::Internal("Mock no configurado para _tx".into()))
    }
    async fn return_stock_tx(
        &self,
        _tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        _freezer_id: Uuid,
        _product_id: Uuid,
        _flavor_id: Uuid,
        _provider_id: Uuid,
        _quantity: i32,
        _updated_by: Uuid,
    ) -> Result<(), AppError> {
        Ok(())
    }
    async fn update_alert(&self, _id: Uuid, _min_stock: i32) -> Result<InventoryItem, AppError> {
        self.update_alert_result.lock().unwrap().take()
            .unwrap_or(Err(AppError::Internal("Mock no configurado".into())))
    }
}

// ═══════════════════════════════════════════════════════════
// Factories de entidades para tests
// ═══════════════════════════════════════════════════════════

/// Crea un User de prueba con valores por defecto.
pub fn fake_user(role: Role) -> User {
    User {
        id: Uuid::new_v4(),
        email: format!("test-{}@helados.com", Uuid::new_v4()),
        display_name: "Test User".into(),
        photo_url: None,
        role,
        active: true,
        notes: None,
        created_at: Utc::now(),
        created_by: None,
        last_login: None,
    }
}

/// Crea un Product de prueba.
pub fn fake_product() -> Product {
    Product {
        id: Uuid::new_v4(),
        name: "Paleta Test".into(),
        active: true,
        created_at: Utc::now(),
        created_by: Uuid::new_v4(),
        modified_at: None,
        modified_by: None,
    }
}

/// Crea un Flavor de prueba.
pub fn fake_flavor(product_id: Uuid) -> Flavor {
    Flavor {
        id: Uuid::new_v4(),
        name: "Chocolate Test".into(),
        product_id,
        active: true,
        created_at: Utc::now(),
        created_by: Uuid::new_v4(),
    }
}

/// Crea un Provider de prueba.
pub fn fake_provider() -> Provider {
    Provider {
        id: Uuid::new_v4(),
        name: "Proveedor Test".into(),
        contact_info: Some("555-0000".into()),
        active: true,
        created_at: Utc::now(),
        created_by: Uuid::new_v4(),
    }
}

/// Crea un Worker de prueba.
pub fn fake_worker() -> Worker {
    Worker {
        id: Uuid::new_v4(),
        name: "Trabajador Test".into(),
        phone: Some("555-1111".into()),
        address: None,
        active: true,
        current_debt: Decimal::ZERO,
        total_sales: 0,
        last_sale: None,
        created_at: Utc::now(),
        created_by: Uuid::new_v4(),
    }
}

/// Crea un Freezer de prueba.
pub fn fake_freezer(number: i32) -> Freezer {
    Freezer {
        id: Uuid::new_v4(),
        number,
        max_capacity: serde_json::json!({"paletas": 500}),
        is_on: true,
        last_toggle: None,
        created_at: Utc::now(),
        created_by: Uuid::new_v4(),
    }
}

/// Crea un PriceHistory de prueba.
pub fn fake_price(product_id: Uuid, flavor_id: Uuid, provider_id: Uuid) -> PriceHistory {
    PriceHistory {
        id: Uuid::new_v4(),
        product_id,
        flavor_id,
        provider_id,
        cost_price: Decimal::new(500, 2),
        price_base: Decimal::new(1000, 2),
        price_route: Decimal::new(1200, 2),
        price_local: Decimal::new(1500, 2),
        commission: Decimal::new(200, 2),
        effective_date: Utc::now(),
        created_by: Uuid::new_v4(),
        created_at: Utc::now(),
    }
}

/// Crea un InventoryItem de prueba.
pub fn fake_inventory_item(
    freezer_id: Uuid,
    product_id: Uuid,
    flavor_id: Uuid,
    provider_id: Uuid,
) -> InventoryItem {
    InventoryItem {
        id: Uuid::new_v4(),
        freezer_id,
        product_id,
        flavor_id,
        provider_id,
        quantity: 100,
        min_stock_alert: 20,
        is_deformed: false,
        assigned_worker_id: None,
        last_updated: Utc::now(),
        updated_by: Uuid::new_v4(),
    }
}
