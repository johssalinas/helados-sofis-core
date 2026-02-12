mod common;

use std::sync::Arc;
use uuid::Uuid;

use common::mocks::*;
use helados_sofis_core::modules::pricing::application::manage_prices;
use helados_sofis_core::modules::pricing::domain::repositories::PriceRepository;

// ═══════════════════════════════════════════════════════════
// Tests de Casos de Uso — Precios (con Mocks)
// Patrón AAA: Arrange → Act → Assert
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod precios_tests {
    use super::*;
    use helados_sofis_core::modules::pricing::domain::entities::CreatePriceDto;
    use rust_decimal::Decimal;

    #[tokio::test]
    async fn crear_precio_exitoso() {
        // Arrange
        let mut mock = MockPriceRepo::new();
        let product_id = Uuid::new_v4();
        let flavor_id = Uuid::new_v4();
        let provider_id = Uuid::new_v4();
        let price = fake_price(product_id, flavor_id, provider_id);
        let price_clone = price.clone();

        mock.expect_create()
            .times(1)
            .returning(move |_, _| Ok(price_clone.clone()));

        let repo: Arc<dyn PriceRepository> = Arc::new(mock);
        let dto = CreatePriceDto {
            product_id,
            flavor_id,
            provider_id,
            cost_price: Decimal::new(500, 2),
            price_base: Decimal::new(1000, 2),
            price_route: Decimal::new(1200, 2),
            price_local: Decimal::new(1500, 2),
        };

        // Act
        let result = manage_prices::create_price(&repo, dto, Uuid::new_v4()).await;

        // Assert
        assert!(result.is_ok());
        let created = result.unwrap();
        assert_eq!(created.product_id, product_id);
        assert_eq!(created.flavor_id, flavor_id);
    }

    #[tokio::test]
    async fn obtener_precio_actual_existente() {
        // Arrange
        let mut mock = MockPriceRepo::new();
        let product_id = Uuid::new_v4();
        let flavor_id = Uuid::new_v4();
        let provider_id = Uuid::new_v4();
        let price = fake_price(product_id, flavor_id, provider_id);
        let price_clone = price.clone();

        mock.expect_get_current_price()
            .times(1)
            .returning(move |_, _, _| Ok(Some(price_clone.clone())));

        let repo: Arc<dyn PriceRepository> = Arc::new(mock);

        // Act
        let result =
            manage_prices::get_current_price(&repo, product_id, flavor_id, provider_id).await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn obtener_precio_actual_no_registrado_falla() {
        // Arrange
        let mut mock = MockPriceRepo::new();

        mock.expect_get_current_price()
            .times(1)
            .returning(|_, _, _| Ok(None));

        let repo: Arc<dyn PriceRepository> = Arc::new(mock);

        // Act
        let result = manage_prices::get_current_price(
            &repo,
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        )
        .await;

        // Assert
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("No hay precio"));
    }

    #[tokio::test]
    async fn listar_precios_actuales() {
        // Arrange
        let mut mock = MockPriceRepo::new();
        let prices = vec![
            fake_price(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()),
            fake_price(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()),
        ];
        let prices_clone = prices.clone();

        mock.expect_list_current_prices()
            .times(1)
            .returning(move || Ok(prices_clone.clone()));

        let repo: Arc<dyn PriceRepository> = Arc::new(mock);

        // Act
        let result = manage_prices::list_current_prices(&repo).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn obtener_historial_precios() {
        // Arrange
        let mut mock = MockPriceRepo::new();
        let product_id = Uuid::new_v4();
        let flavor_id = Uuid::new_v4();
        let provider_id = Uuid::new_v4();

        let history = vec![
            fake_price(product_id, flavor_id, provider_id),
            fake_price(product_id, flavor_id, provider_id),
            fake_price(product_id, flavor_id, provider_id),
        ];
        let history_clone = history.clone();

        mock.expect_get_history()
            .times(1)
            .returning(move |_, _, _| Ok(history_clone.clone()));

        let repo: Arc<dyn PriceRepository> = Arc::new(mock);

        // Act
        let result =
            manage_prices::get_price_history(&repo, product_id, flavor_id, provider_id).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 3);
    }
}
