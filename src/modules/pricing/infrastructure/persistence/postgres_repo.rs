use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::shared::errors::AppError;
use crate::modules::pricing::domain::entities::{CreatePriceDto, PriceHistory};
use crate::modules::pricing::domain::repositories::PriceRepository;

pub struct PgPriceRepository {
    pool: PgPool,
}

impl PgPriceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PriceRepository for PgPriceRepository {
    async fn get_current_price(
        &self,
        product_id: Uuid,
        flavor_id: Uuid,
        provider_id: Uuid,
    ) -> Result<Option<PriceHistory>, AppError> {
        Ok(sqlx::query_as::<_, PriceHistory>(
            r#"
            SELECT * FROM price_history
            WHERE product_id = $1 AND flavor_id = $2 AND provider_id = $3
            ORDER BY effective_date DESC
            LIMIT 1
            "#,
        )
        .bind(product_id)
        .bind(flavor_id)
        .bind(provider_id)
        .fetch_optional(&self.pool)
        .await?)
    }

    async fn get_price_at(
        &self,
        product_id: Uuid,
        flavor_id: Uuid,
        provider_id: Uuid,
        date: DateTime<Utc>,
    ) -> Result<Option<PriceHistory>, AppError> {
        Ok(sqlx::query_as::<_, PriceHistory>(
            r#"
            SELECT * FROM price_history
            WHERE product_id = $1 AND flavor_id = $2 AND provider_id = $3
              AND effective_date <= $4
            ORDER BY effective_date DESC
            LIMIT 1
            "#,
        )
        .bind(product_id)
        .bind(flavor_id)
        .bind(provider_id)
        .bind(date)
        .fetch_optional(&self.pool)
        .await?)
    }

    async fn create(&self, dto: &CreatePriceDto, created_by: Uuid) -> Result<PriceHistory, AppError> {
        let commission = dto.price_route - dto.price_base;
        Ok(sqlx::query_as::<_, PriceHistory>(
            r#"
            INSERT INTO price_history 
            (product_id, flavor_id, provider_id, cost_price, price_base, 
             price_route, price_local, commission, effective_date, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), $9)
            RETURNING *
            "#,
        )
        .bind(dto.product_id)
        .bind(dto.flavor_id)
        .bind(dto.provider_id)
        .bind(dto.cost_price)
        .bind(dto.price_base)
        .bind(dto.price_route)
        .bind(dto.price_local)
        .bind(commission)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await?)
    }

    async fn get_history(
        &self,
        product_id: Uuid,
        flavor_id: Uuid,
        provider_id: Uuid,
    ) -> Result<Vec<PriceHistory>, AppError> {
        Ok(sqlx::query_as::<_, PriceHistory>(
            r#"
            SELECT * FROM price_history
            WHERE product_id = $1 AND flavor_id = $2 AND provider_id = $3
            ORDER BY effective_date DESC
            "#,
        )
        .bind(product_id)
        .bind(flavor_id)
        .bind(provider_id)
        .fetch_all(&self.pool)
        .await?)
    }

    async fn list_current_prices(&self) -> Result<Vec<PriceHistory>, AppError> {
        Ok(sqlx::query_as::<_, PriceHistory>(
            r#"
            SELECT DISTINCT ON (product_id, flavor_id, provider_id) *
            FROM price_history
            ORDER BY product_id, flavor_id, provider_id, effective_date DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?)
    }
}
