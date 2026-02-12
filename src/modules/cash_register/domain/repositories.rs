use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use super::entities::{CashTransaction, CashTransactionType};
use crate::shared::errors::AppError;

#[async_trait]
pub trait CashRegisterRepository: Send + Sync {
    async fn get_current_balance(&self) -> Result<Decimal, AppError>;
    async fn calculate_balance_from_scratch(&self) -> Result<Decimal, AppError>;
    async fn add_transaction(
        &self,
        tx_type: CashTransactionType,
        amount: Decimal,
        description: Option<String>,
        category: Option<String>,
        related_doc_type: Option<String>,
        related_doc_id: Option<Uuid>,
        created_by: Uuid,
    ) -> Result<CashTransaction, AppError>;
    async fn get_todays_transactions(&self) -> Result<Vec<CashTransaction>, AppError>;
    async fn get_transactions_by_range(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<CashTransaction>, AppError>;
    async fn get_monthly_summary(
        &self,
        year: i32,
        month: u32,
    ) -> Result<Vec<CashTransaction>, AppError>;
}
