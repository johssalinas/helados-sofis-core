mod common;

use std::sync::Arc;
use uuid::Uuid;

use common::mocks::*;
use helados_sofis_core::modules::inventory::application::manage_inventory;
use helados_sofis_core::modules::inventory::domain::entities::*;
use helados_sofis_core::modules::inventory::domain::repositories::InventoryRepository;

// ═══════════════════════════════════════════════════════════
// Tests de Casos de Uso — Inventario (con Mocks)
// Patrón AAA: Arrange → Act → Assert
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod inventario_tests {
    use super::*;

    #[tokio::test]
    async fn listar_todo_el_inventario() {
        // Arrange
        let mock = MockInventoryRepo::new();
        let items = vec![
            fake_inventory_item(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()),
            fake_inventory_item(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()),
        ];
        *mock.find_all_result.lock().unwrap() = Some(Ok(items));

        let repo: Arc<dyn InventoryRepository> = Arc::new(mock);

        // Act
        let result = manage_inventory::list_all(&repo).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn listar_inventario_por_congelador() {
        // Arrange
        let mock = MockInventoryRepo::new();
        let freezer_id = Uuid::new_v4();
        let item = fake_inventory_item(freezer_id, Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        *mock.find_by_freezer_result.lock().unwrap() = Some(Ok(vec![item]));

        let repo: Arc<dyn InventoryRepository> = Arc::new(mock);

        // Act
        let result = manage_inventory::list_by_freezer(&repo, freezer_id).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn listar_stock_vendible() {
        // Arrange
        let mock = MockInventoryRepo::new();
        *mock.find_sellable_result.lock().unwrap() = Some(Ok(vec![]));

        let repo: Arc<dyn InventoryRepository> = Arc::new(mock);

        // Act
        let result = manage_inventory::list_sellable(&repo).await;

        // Assert
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn listar_stock_bajo() {
        // Arrange
        let mock = MockInventoryRepo::new();
        let mut low_item = fake_inventory_item(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        );
        low_item.quantity = 5;
        low_item.min_stock_alert = 20;
        *mock.find_low_stock_result.lock().unwrap() = Some(Ok(vec![low_item]));

        let repo: Arc<dyn InventoryRepository> = Arc::new(mock);

        // Act
        let result = manage_inventory::list_low_stock(&repo).await;

        // Assert
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 1);
        assert!(items[0].quantity < items[0].min_stock_alert);
    }

    #[tokio::test]
    async fn agregar_stock_exitoso() {
        // Arrange
        let mock = MockInventoryRepo::new();
        let freezer_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let flavor_id = Uuid::new_v4();
        let provider_id = Uuid::new_v4();
        let updated_by = Uuid::new_v4();

        let mut expected = fake_inventory_item(freezer_id, product_id, flavor_id, provider_id);
        expected.quantity = 150; // 100 existente + 50 agregado
        *mock.add_stock_result.lock().unwrap() = Some(Ok(expected));

        let repo: Arc<dyn InventoryRepository> = Arc::new(mock);
        let dto = AddStockDto {
            freezer_id,
            product_id,
            flavor_id,
            provider_id,
            quantity: 50,
        };

        // Act
        let result = manage_inventory::add_stock(&repo, dto, updated_by).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().quantity, 150);
    }

    #[tokio::test]
    async fn actualizar_alerta_stock() {
        // Arrange
        let mock = MockInventoryRepo::new();
        let item_id = Uuid::new_v4();
        let mut updated = fake_inventory_item(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        );
        updated.id = item_id;
        updated.min_stock_alert = 50;
        *mock.update_alert_result.lock().unwrap() = Some(Ok(updated));

        let repo: Arc<dyn InventoryRepository> = Arc::new(mock);

        // Act
        let result = manage_inventory::update_alert(&repo, item_id, 50).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().min_stock_alert, 50);
    }

    #[tokio::test]
    async fn listar_deformados_por_trabajador() {
        // Arrange
        let mock = MockInventoryRepo::new();
        let worker_id = Uuid::new_v4();
        let mut deformed = fake_inventory_item(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        );
        deformed.is_deformed = true;
        deformed.assigned_worker_id = Some(worker_id);
        *mock.find_worker_deformed_result.lock().unwrap() = Some(Ok(vec![deformed]));

        let repo: Arc<dyn InventoryRepository> = Arc::new(mock);

        // Act
        let result = manage_inventory::list_worker_deformed(&repo, worker_id).await;

        // Assert
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 1);
        assert!(items[0].is_deformed);
        assert_eq!(items[0].assigned_worker_id, Some(worker_id));
    }
}
