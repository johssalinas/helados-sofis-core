use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CashTransaction {
    pub id: Uuid,
    #[sqlx(rename = "type")]
    #[serde(rename = "type")]
    pub tx_type: String,
    pub amount: Decimal,
    pub balance: Decimal,
    pub description: Option<String>,
    pub category: Option<String>,
    pub related_doc_type: Option<String>,
    pub related_doc_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CashTransactionType {
    WorkerPayment,
    LocalSale,
    OwnerSale,
    OwnerWithdrawal,
    Expense,
}

impl CashTransactionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WorkerPayment => "worker_payment",
            Self::LocalSale => "local_sale",
            Self::OwnerSale => "owner_sale",
            Self::OwnerWithdrawal => "owner_withdrawal",
            Self::Expense => "expense",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateExpenseDto {
    pub amount: Decimal,
    pub description: Option<String>,
    pub category: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateWithdrawalDto {
    pub amount: Decimal,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BalanceInfo {
    pub current_balance: Decimal,
    pub calculated_balance: Decimal,
    pub is_consistent: bool,
}

#[derive(Debug, Deserialize)]
pub struct DateRangeQuery {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}
