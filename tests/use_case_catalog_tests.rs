mod common;

use std::sync::Arc;
use uuid::Uuid;

use common::mocks::*;
use helados_sofis_core::modules::catalog::application::crud;
use helados_sofis_core::modules::catalog::domain::entities::*;
use helados_sofis_core::modules::catalog::domain::repositories::*;

// ═══════════════════════════════════════════════════════════
// Tests de Casos de Uso — Catálogo (con Mocks)
// Patrón AAA: Arrange → Act → Assert
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod productos_tests {
    use super::*;

    #[tokio::test]
    async fn listar_productos_activos() {
        // Arrange
        let mut mock = MockProductRepo::new();
        let products = vec![fake_product(), fake_product()];
        let products_clone = products.clone();

        mock.expect_find_active()
            .times(1)
            .returning(move || Ok(products_clone.clone()));

        let repo: Arc<dyn ProductRepository> = Arc::new(mock);

        // Act
        let result = crud::list_products(&repo).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn obtener_producto_existente() {
        // Arrange
        let mut mock = MockProductRepo::new();
        let product = fake_product();
        let product_id = product.id;
        let product_clone = product.clone();

        mock.expect_find_by_id()
            .withf(move |id| *id == product_id)
            .times(1)
            .returning(move |_| Ok(Some(product_clone.clone())));

        let repo: Arc<dyn ProductRepository> = Arc::new(mock);

        // Act
        let result = crud::get_product(&repo, product_id).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, product_id);
    }

    #[tokio::test]
    async fn obtener_producto_no_existente_retorna_not_found() {
        // Arrange
        let mut mock = MockProductRepo::new();

        mock.expect_find_by_id()
            .times(1)
            .returning(|_| Ok(None));

        let repo: Arc<dyn ProductRepository> = Arc::new(mock);

        // Act
        let result = crud::get_product(&repo, Uuid::new_v4()).await;

        // Assert
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("no encontrado"));
    }

    #[tokio::test]
    async fn crear_producto_exitoso() {
        // Arrange
        let mut mock = MockProductRepo::new();
        let new_product = fake_product();
        let new_clone = new_product.clone();
        let created_by = Uuid::new_v4();

        mock.expect_create()
            .times(1)
            .returning(move |_, _| Ok(new_clone.clone()));

        let repo: Arc<dyn ProductRepository> = Arc::new(mock);
        let dto = CreateProductDto {
            name: "Helado Nuevo".into(),
        };

        // Act
        let result = crud::create_product(&repo, dto, created_by).await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn actualizar_producto_existente_ok() {
        // Arrange
        let mut mock = MockProductRepo::new();
        let existing = fake_product();
        let product_id = existing.id;
        let existing_clone = existing.clone();
        let mut updated = existing.clone();
        updated.name = "Nombre Actualizado".into();
        let updated_clone = updated.clone();
        let modified_by = Uuid::new_v4();

        mock.expect_find_by_id()
            .times(1)
            .returning(move |_| Ok(Some(existing_clone.clone())));

        mock.expect_update()
            .times(1)
            .returning(move |_, _, _| Ok(updated_clone.clone()));

        let repo: Arc<dyn ProductRepository> = Arc::new(mock);
        let dto = UpdateProductDto {
            name: Some("Nombre Actualizado".into()),
            active: None,
        };

        // Act
        let result = crud::update_product(&repo, product_id, dto, modified_by).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "Nombre Actualizado");
    }

    #[tokio::test]
    async fn actualizar_producto_no_existente_falla() {
        // Arrange
        let mut mock = MockProductRepo::new();

        mock.expect_find_by_id()
            .times(1)
            .returning(|_| Ok(None));
        mock.expect_update().times(0);

        let repo: Arc<dyn ProductRepository> = Arc::new(mock);
        let dto = UpdateProductDto {
            name: Some("No debe".into()),
            active: None,
        };

        // Act
        let result = crud::update_product(&repo, Uuid::new_v4(), dto, Uuid::new_v4()).await;

        // Assert
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod sabores_tests {
    use super::*;

    #[tokio::test]
    async fn listar_todos_los_sabores() {
        // Arrange
        let mut mock = MockFlavorRepo::new();
        let product_id = Uuid::new_v4();
        let flavors = vec![
            fake_flavor(product_id),
            fake_flavor(product_id),
        ];
        let flavors_clone = flavors.clone();

        mock.expect_find_all()
            .times(1)
            .returning(move || Ok(flavors_clone.clone()));

        let repo: Arc<dyn FlavorRepository> = Arc::new(mock);

        // Act
        let result = crud::list_flavors(&repo).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn listar_sabores_por_producto() {
        // Arrange
        let mut mock = MockFlavorRepo::new();
        let product_id = Uuid::new_v4();
        let flavor = fake_flavor(product_id);
        let flavor_clone = flavor.clone();

        mock.expect_find_by_product()
            .withf(move |pid| *pid == product_id)
            .times(1)
            .returning(move |_| Ok(vec![flavor_clone.clone()]));

        let repo: Arc<dyn FlavorRepository> = Arc::new(mock);

        // Act
        let result = crud::list_flavors_by_product(&repo, product_id).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn crear_sabor_exitoso() {
        // Arrange
        let mut mock = MockFlavorRepo::new();
        let product_id = Uuid::new_v4();
        let new_flavor = fake_flavor(product_id);
        let new_clone = new_flavor.clone();

        mock.expect_create()
            .times(1)
            .returning(move |_, _| Ok(new_clone.clone()));

        let repo: Arc<dyn FlavorRepository> = Arc::new(mock);
        let dto = CreateFlavorDto {
            name: "Vainilla".into(),
            product_id,
        };

        // Act
        let result = crud::create_flavor(&repo, dto, Uuid::new_v4()).await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn actualizar_sabor_no_existente_falla() {
        // Arrange
        let mut mock = MockFlavorRepo::new();

        mock.expect_find_by_id()
            .times(1)
            .returning(|_| Ok(None));
        mock.expect_update().times(0);

        let repo: Arc<dyn FlavorRepository> = Arc::new(mock);
        let dto = UpdateFlavorDto {
            name: Some("Fresa".into()),
            active: None,
        };

        // Act
        let result = crud::update_flavor(&repo, Uuid::new_v4(), dto).await;

        // Assert
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod proveedores_tests {
    use super::*;

    #[tokio::test]
    async fn listar_proveedores_activos() {
        // Arrange
        let mut mock = MockProviderRepo::new();
        let providers = vec![fake_provider()];
        let providers_clone = providers.clone();

        mock.expect_find_active()
            .times(1)
            .returning(move || Ok(providers_clone.clone()));

        let repo: Arc<dyn ProviderRepository> = Arc::new(mock);

        // Act
        let result = crud::list_providers(&repo).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn crear_proveedor_exitoso() {
        // Arrange
        let mut mock = MockProviderRepo::new();
        let provider = fake_provider();
        let provider_clone = provider.clone();

        mock.expect_create()
            .times(1)
            .returning(move |_, _| Ok(provider_clone.clone()));

        let repo: Arc<dyn ProviderRepository> = Arc::new(mock);
        let dto = CreateProviderDto {
            name: "Nuevo Proveedor".into(),
            contact_info: Some("info@proveedor.com".into()),
        };

        // Act
        let result = crud::create_provider(&repo, dto, Uuid::new_v4()).await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn actualizar_proveedor_no_existente_falla() {
        // Arrange
        let mut mock = MockProviderRepo::new();

        mock.expect_find_by_id()
            .times(1)
            .returning(|_| Ok(None));

        let repo: Arc<dyn ProviderRepository> = Arc::new(mock);
        let dto = UpdateProviderDto {
            name: Some("Actualizado".into()),
            contact_info: None,
            active: None,
        };

        // Act
        let result = crud::update_provider(&repo, Uuid::new_v4(), dto).await;

        // Assert
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod trabajadores_tests {
    use super::*;

    #[tokio::test]
    async fn listar_trabajadores_activos() {
        // Arrange
        let mut mock = MockWorkerRepo::new();
        let workers = vec![fake_worker(), fake_worker()];
        let workers_clone = workers.clone();

        mock.expect_find_active()
            .times(1)
            .returning(move || Ok(workers_clone.clone()));

        let repo: Arc<dyn WorkerRepository> = Arc::new(mock);

        // Act
        let result = crud::list_workers(&repo).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn obtener_trabajador_existente() {
        // Arrange
        let mut mock = MockWorkerRepo::new();
        let worker = fake_worker();
        let worker_id = worker.id;
        let worker_clone = worker.clone();

        mock.expect_find_by_id()
            .times(1)
            .returning(move |_| Ok(Some(worker_clone.clone())));

        let repo: Arc<dyn WorkerRepository> = Arc::new(mock);

        // Act
        let result = crud::get_worker(&repo, worker_id).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, worker_id);
    }

    #[tokio::test]
    async fn crear_trabajador_exitoso() {
        // Arrange
        let mut mock = MockWorkerRepo::new();
        let worker = fake_worker();
        let worker_clone = worker.clone();

        mock.expect_create()
            .times(1)
            .returning(move |_, _| Ok(worker_clone.clone()));

        let repo: Arc<dyn WorkerRepository> = Arc::new(mock);
        let dto = CreateWorkerDto {
            name: "Nuevo Trabajador".into(),
            phone: Some("555-9999".into()),
            address: None,
        };

        // Act
        let result = crud::create_worker(&repo, dto, Uuid::new_v4()).await;

        // Assert
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod congeladores_tests {
    use super::*;

    #[tokio::test]
    async fn listar_congeladores() {
        // Arrange
        let mut mock = MockFreezerRepo::new();
        let freezers = vec![fake_freezer(1), fake_freezer(2)];
        let freezers_clone = freezers.clone();

        mock.expect_find_all()
            .times(1)
            .returning(move || Ok(freezers_clone.clone()));

        let repo: Arc<dyn FreezerRepository> = Arc::new(mock);

        // Act
        let result = crud::list_freezers(&repo).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn obtener_congelador_existente() {
        // Arrange
        let mut mock = MockFreezerRepo::new();
        let freezer = fake_freezer(1);
        let freezer_id = freezer.id;
        let freezer_clone = freezer.clone();

        mock.expect_find_by_id()
            .times(1)
            .returning(move |_| Ok(Some(freezer_clone.clone())));

        let repo: Arc<dyn FreezerRepository> = Arc::new(mock);

        // Act
        let result = crud::get_freezer(&repo, freezer_id).await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn toggle_congelador_no_existente_falla() {
        // Arrange
        let mut mock = MockFreezerRepo::new();

        mock.expect_find_by_id()
            .times(1)
            .returning(|_| Ok(None));

        let repo: Arc<dyn FreezerRepository> = Arc::new(mock);

        // Act
        let result = crud::toggle_freezer(&repo, Uuid::new_v4()).await;

        // Assert
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn toggle_congelador_existente_ok() {
        // Arrange
        let mut mock = MockFreezerRepo::new();
        let freezer = fake_freezer(1);
        let freezer_id = freezer.id;
        let freezer_clone = freezer.clone();
        let mut toggled = freezer.clone();
        toggled.is_on = false;
        let toggled_clone = toggled.clone();

        mock.expect_find_by_id()
            .times(1)
            .returning(move |_| Ok(Some(freezer_clone.clone())));

        mock.expect_toggle_power()
            .withf(move |id| *id == freezer_id)
            .times(1)
            .returning(move |_| Ok(toggled_clone.clone()));

        let repo: Arc<dyn FreezerRepository> = Arc::new(mock);

        // Act
        let result = crud::toggle_freezer(&repo, freezer_id).await;

        // Assert
        assert!(result.is_ok());
        assert!(!result.unwrap().is_on);
    }
}

#[cfg(test)]
mod rutas_tests {
    use super::*;

    #[tokio::test]
    async fn listar_rutas() {
        // Arrange
        let mut mock = MockRouteRepo::new();
        mock.expect_find_all()
            .times(1)
            .returning(|| Ok(vec![]));

        let repo: Arc<dyn RouteRepository> = Arc::new(mock);

        // Act
        let result = crud::list_routes(&repo).await;

        // Assert
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
