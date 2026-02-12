use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::cash_register::domain::entities::{CashTransaction, CashTransactionType};
use crate::modules::cash_register::domain::repositories::CashRegisterRepository;
use crate::shared::errors::AppError;

pub struct PgCashRegisterRepository {
    pool: PgPool,
}

impl PgCashRegisterRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CashRegisterRepository for PgCashRegisterRepository {
    async fn get_current_balance(&self) -> Result<Decimal, AppError> {
        let balance = sqlx::query_scalar::<_, Option<Decimal>>(
            "SELECT balance FROM cash_register ORDER BY created_at DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?
        .flatten()
        .unwrap_or(Decimal::ZERO);
        Ok(balance)
    }

    async fn calculate_balance_from_scratch(&self) -> Result<Decimal, AppError> {
        let balance = sqlx::query_scalar::<_, Option<Decimal>>(
            "SELECT COALESCE(SUM(amount), 0) FROM cash_register",
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(Decimal::ZERO);
        Ok(balance)
    }

    async fn add_transaction(
        &self,
        tx_type: CashTransactionType,
        amount: Decimal,
        description: Option<String>,
        category: Option<String>,
        related_doc_type: Option<String>,
        related_doc_id: Option<Uuid>,
        created_by: Uuid,
    ) -> Result<CashTransaction, AppError> {
        let mut tx = self.pool.begin().await?;

        // Obtener balance actual CON LOCK para evitar race conditions
        let current_balance = sqlx::query_scalar::<_, Option<Decimal>>(
            "SELECT balance FROM cash_register ORDER BY created_at DESC LIMIT 1 FOR UPDATE",
        )
        .fetch_optional(&mut *tx)
        .await?
        .flatten()
        .unwrap_or(Decimal::ZERO);

        let new_balance = current_balance + amount;

        let transaction = sqlx::query_as::<_, CashTransaction>(
            r#"
            INSERT INTO cash_register 
            (type, amount, balance, description, category, related_doc_type, related_doc_id, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(tx_type.as_str())
        .bind(amount)
        .bind(new_balance)
        .bind(description)
        .bind(category)
        .bind(related_doc_type)
        .bind(related_doc_id)
        .bind(created_by)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(transaction)
    }

    async fn get_todays_transactions(&self) -> Result<Vec<CashTransaction>, AppError> {
        Ok(sqlx::query_as::<_, CashTransaction>(
            "SELECT * FROM cash_register WHERE created_at >= CURRENT_DATE ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?)
    }

    async fn get_transactions_by_range(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<CashTransaction>, AppError> {
        Ok(sqlx::query_as::<_, CashTransaction>(
            "SELECT * FROM cash_register WHERE created_at >= $1 AND created_at < $2 ORDER BY created_at DESC",
        )
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await?)
    }

    async fn get_monthly_summary(
        &self,
        year: i32,
        month: u32,
    ) -> Result<Vec<CashTransaction>, AppError> {
        let start = NaiveDate::from_ymd_opt(year, month, 1)
            .ok_or_else(|| AppError::BadRequest("Fecha inválida".into()))?;
        let end = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
        };

        Ok(sqlx::query_as::<_, CashTransaction>(
            "SELECT * FROM cash_register WHERE created_at >= $1 AND created_at < $2 ORDER BY created_at DESC",
        )
        .bind(start.and_hms_opt(0, 0, 0).unwrap().and_utc())
        .bind(end.and_hms_opt(0, 0, 0).unwrap().and_utc())
        .fetch_all(&self.pool)
        .await?)
    }
}
