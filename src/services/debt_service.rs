use sqlx::PgPool;
use uuid::Uuid;
use rust_decimal::Decimal;
use chrono::Utc;
use crate::errors::AppError;
use crate::models::debt::*;
use crate::repositories::{debt_repo, account_repo};

pub async fn list_debts(pool: &PgPool, dt: Option<String>, st: Option<String>) -> Result<Vec<Debt>, AppError> {
    Ok(debt_repo::find_all(pool, dt.as_deref(), st.as_deref()).await?)
}

pub async fn get_debt(pool: &PgPool, id: Uuid) -> Result<Debt, AppError> {
    debt_repo::find_by_id(pool, id).await?.ok_or_else(|| AppError::NotFound(format!("Debt {} not found", id)))
}

pub async fn create_debt(pool: &PgPool, req: CreateDebtRequest) -> Result<Debt, AppError> {
    if req.amount <= Decimal::ZERO { return Err(AppError::Validation("Amount must be positive".into())); }
    if !["i_owe","owed_to_me"].contains(&req.debt_type.as_str()) { return Err(AppError::Validation("debt_type must be 'i_owe' or 'owed_to_me'".into())); }
    if req.debt_type == "i_owe" {
        if let Some(limit) = debt_repo::get_current_limit(pool).await? {
            let current = debt_repo::get_total_by_type(pool, "i_owe").await?;
            if current + req.amount > limit.max_total_debt { return Err(AppError::Validation(format!("Debt limit exceeded. Current: {}, New: {}, Limit: {}", current, req.amount, limit.max_total_debt))); }
            if let Some(max_s) = limit.max_single_debt { if req.amount > max_s { return Err(AppError::Validation(format!("Amount {} exceeds single debt limit {}", req.amount, max_s))); } }
        }
    }
    Ok(debt_repo::create(pool, &req).await?)
}

pub async fn update_debt(pool: &PgPool, id: Uuid, req: UpdateDebtRequest) -> Result<Debt, AppError> {
    debt_repo::update(pool, id, &req).await?.ok_or_else(|| AppError::NotFound(format!("Debt {} not found", id)))
}

pub async fn delete_debt(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    if !debt_repo::delete(pool, id).await? { return Err(AppError::NotFound(format!("Debt {} not found", id))); }
    Ok(())
}

pub async fn record_payment(pool: &PgPool, debt_id: Uuid, req: CreateDebtPaymentRequest) -> Result<DebtPayment, AppError> {
    let debt = get_debt(pool, debt_id).await?;
    if req.amount <= Decimal::ZERO { return Err(AppError::Validation("Payment must be positive".into())); }
    if req.amount > debt.remaining_amount { return Err(AppError::Validation(format!("Payment {} exceeds remaining {}", req.amount, debt.remaining_amount))); }
    let payment = debt_repo::create_payment(pool, debt_id, &req).await?;
    let new_rem = debt.remaining_amount - req.amount;
    let status = if new_rem == Decimal::ZERO { "settled" } else { "partially_paid" };
    debt_repo::update_remaining(pool, debt_id, new_rem, status).await?;
    Ok(payment)
}

pub async fn get_payments(pool: &PgPool, debt_id: Uuid) -> Result<Vec<DebtPayment>, AppError> {
    let _ = get_debt(pool, debt_id).await?;
    Ok(debt_repo::find_payments(pool, debt_id).await?)
}

pub async fn get_summary(pool: &PgPool) -> Result<DebtSummary, AppError> {
    let owed = debt_repo::get_total_by_type(pool, "i_owe").await?;
    let receivable = debt_repo::get_total_by_type(pool, "owed_to_me").await?;
    let balance = account_repo::get_total_balance(pool).await?;
    let net = receivable - owed;
    let limit = debt_repo::get_current_limit(pool).await?;
    let today = Utc::now().date_naive();
    let upcoming = debt_repo::find_upcoming_deadlines(pool, 30).await?;
    let deadlines: Vec<DebtDeadline> = upcoming.into_iter().filter_map(|d| {
        d.deadline.map(|dl| DebtDeadline { debt_id: d.id, counterparty: d.counterparty, amount: d.amount, remaining: d.remaining_amount, debt_type: d.debt_type, deadline: dl, days_remaining: (dl - today).num_days() })
    }).collect();
    Ok(DebtSummary {
        total_owed: owed, total_receivable: receivable, net_position: net, current_balance: balance,
        projected_balance_after_settlements: balance + net,
        debt_limit: limit.as_ref().map(|l| l.max_total_debt),
        available_debt_capacity: limit.map(|l| (l.max_total_debt - owed).max(Decimal::ZERO)),
        upcoming_deadlines: deadlines,
    })
}

pub async fn get_limit(pool: &PgPool) -> Result<Option<DebtLimit>, AppError> { Ok(debt_repo::get_current_limit(pool).await?) }

pub async fn set_limit(pool: &PgPool, req: CreateDebtLimitRequest) -> Result<DebtLimit, AppError> {
    if req.max_total_debt <= Decimal::ZERO { return Err(AppError::Validation("Limit must be positive".into())); }
    Ok(debt_repo::create_limit(pool, &req).await?)
}
