use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::shared::errors::AppError;
use crate::modules::cash_register::domain::entities::*;
use crate::modules::cash_register::domain::repositories::CashRegisterRepository;

pub async fn get_balance(
    repo: &dyn CashRegisterRepository,
) -> Result<BalanceInfo, AppError> {
    let current = repo.get_current_balance().await?;
    let calculated = repo.calculate_balance_from_scratch().await?;
    Ok(BalanceInfo {
        current_balance: current,
        calculated_balance: calculated,
        is_consistent: current == calculated,
    })
}

pub async fn add_expense(
    repo: &dyn CashRegisterRepository,
    dto: &CreateExpenseDto,
    created_by: Uuid,
) -> Result<CashTransaction, AppError> {
    if dto.amount <= Decimal::ZERO {
        return Err(AppError::BadRequest("El monto del gasto debe ser positivo".into()));
    }
    // Gastos son negativos en la caja
    repo.add_transaction(
        CashTransactionType::Expense,
        -dto.amount,
        dto.description.clone(),
        Some(dto.category.clone()),
        None,
        None,
        created_by,
    )
    .await
}

pub async fn add_withdrawal(
    repo: &dyn CashRegisterRepository,
    dto: &CreateWithdrawalDto,
    created_by: Uuid,
) -> Result<CashTransaction, AppError> {
    if dto.amount <= Decimal::ZERO {
        return Err(AppError::BadRequest("El monto del retiro debe ser positivo".into()));
    }
    repo.add_transaction(
        CashTransactionType::OwnerWithdrawal,
        -dto.amount,
        dto.description.clone(),
        None,
        None,
        None,
        created_by,
    )
    .await
}

pub async fn todays_transactions(
    repo: &dyn CashRegisterRepository,
) -> Result<Vec<CashTransaction>, AppError> {
    repo.get_todays_transactions().await
}

pub async fn transactions_by_range(
    repo: &dyn CashRegisterRepository,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<Vec<CashTransaction>, AppError> {
    repo.get_transactions_by_range(from, to).await
}
