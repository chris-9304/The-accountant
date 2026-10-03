use sqlx::PgPool;
use uuid::Uuid;
use rust_decimal::Decimal;
use chrono::Utc;
use crate::models::debt::*;

pub async fn find_all(pool: &PgPool, debt_type: Option<&str>, status: Option<&str>) -> Result<Vec<Debt>, sqlx::Error> {
    sqlx::query_as::<_, Debt>(
        "SELECT * FROM debts WHERE ($1::text IS NULL OR debt_type = $1) AND ($2::text IS NULL OR status = $2) ORDER BY deadline ASC NULLS LAST"
    ).bind(debt_type).bind(status).fetch_all(pool).await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Debt>, sqlx::Error> {
    sqlx::query_as::<_, Debt>("SELECT * FROM debts WHERE id = $1").bind(id).fetch_optional(pool).await
}

pub async fn create(pool: &PgPool, req: &CreateDebtRequest) -> Result<Debt, sqlx::Error> {
    let interest = req.interest_rate.unwrap_or(Decimal::ZERO);
    let priority = req.priority.as_deref().unwrap_or("medium");
    sqlx::query_as::<_, Debt>(
        "INSERT INTO debts (counterparty, amount, remaining_amount, debt_type, reason, deadline, interest_rate, priority) VALUES ($1, $2, $2, $3, $4, $5, $6, $7) RETURNING *"
    ).bind(&req.counterparty).bind(req.amount).bind(&req.debt_type).bind(&req.reason)
     .bind(req.deadline).bind(interest).bind(priority).fetch_one(pool).await
}

pub async fn update(pool: &PgPool, id: Uuid, req: &UpdateDebtRequest) -> Result<Option<Debt>, sqlx::Error> {
    let current = find_by_id(pool, id).await?;
    let Some(current) = current else { return Ok(None) };
    let counterparty = req.counterparty.as_deref().unwrap_or(&current.counterparty);
    let reason = req.reason.as_ref().or(current.reason.as_ref());
    let deadline = req.deadline.or(current.deadline);
    let priority = req.priority.as_deref().unwrap_or(&current.priority);
    let status = req.status.as_deref().unwrap_or(&current.status);
    sqlx::query_as::<_, Debt>(
        "UPDATE debts SET counterparty=$1, reason=$2, deadline=$3, priority=$4, status=$5, updated_at=NOW() WHERE id=$6 RETURNING *"
    ).bind(counterparty).bind(reason).bind(deadline).bind(priority).bind(status).bind(id).fetch_optional(pool).await
}

pub async fn update_remaining(pool: &PgPool, id: Uuid, remaining: Decimal, status: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE debts SET remaining_amount=$1, status=$2, updated_at=NOW() WHERE id=$3")
        .bind(remaining).bind(status).bind(id).execute(pool).await?;
    Ok(())
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let r = sqlx::query("DELETE FROM debts WHERE id = $1").bind(id).execute(pool).await?;
    Ok(r.rows_affected() > 0)
}

pub async fn get_total_by_type(pool: &PgPool, debt_type: &str) -> Result<Decimal, sqlx::Error> {
    let row: (Option<Decimal>,) = sqlx::query_as(
        "SELECT COALESCE(SUM(remaining_amount), 0) FROM debts WHERE debt_type = $1 AND status IN ('active', 'partially_paid')"
    ).bind(debt_type).fetch_one(pool).await?;
    Ok(row.0.unwrap_or(Decimal::ZERO))
}

pub async fn find_upcoming_deadlines(pool: &PgPool, days: i32) -> Result<Vec<Debt>, sqlx::Error> {
    let future = Utc::now().date_naive() + chrono::Duration::days(days as i64);
    sqlx::query_as::<_, Debt>(
        "SELECT * FROM debts WHERE deadline IS NOT NULL AND deadline <= $1 AND status IN ('active','partially_paid') ORDER BY deadline"
    ).bind(future).fetch_all(pool).await
}

pub async fn find_overdue(pool: &PgPool) -> Result<Vec<Debt>, sqlx::Error> {
    let today = Utc::now().date_naive();
    sqlx::query_as::<_, Debt>(
        "SELECT * FROM debts WHERE deadline IS NOT NULL AND deadline < $1 AND status IN ('active','partially_paid')"
    ).bind(today).fetch_all(pool).await
}

pub async fn create_payment(pool: &PgPool, debt_id: Uuid, req: &CreateDebtPaymentRequest) -> Result<DebtPayment, sqlx::Error> {
    sqlx::query_as::<_, DebtPayment>(
        "INSERT INTO debt_payments (debt_id, amount, payment_date, notes) VALUES ($1, $2, $3, $4) RETURNING *"
    ).bind(debt_id).bind(req.amount).bind(req.payment_date).bind(&req.notes).fetch_one(pool).await
}

pub async fn find_payments(pool: &PgPool, debt_id: Uuid) -> Result<Vec<DebtPayment>, sqlx::Error> {
    sqlx::query_as::<_, DebtPayment>("SELECT * FROM debt_payments WHERE debt_id = $1 ORDER BY payment_date")
        .bind(debt_id).fetch_all(pool).await
}

pub async fn get_current_limit(pool: &PgPool) -> Result<Option<DebtLimit>, sqlx::Error> {
    let today = Utc::now().date_naive();
    sqlx::query_as::<_, DebtLimit>(
        "SELECT * FROM debt_limits WHERE effective_from <= $1 AND (effective_until IS NULL OR effective_until > $1) ORDER BY created_at DESC LIMIT 1"
    ).bind(today).fetch_optional(pool).await
}

pub async fn create_limit(pool: &PgPool, req: &CreateDebtLimitRequest) -> Result<DebtLimit, sqlx::Error> {
    sqlx::query_as::<_, DebtLimit>(
        "INSERT INTO debt_limits (max_total_debt, max_single_debt, effective_from, effective_until) VALUES ($1, $2, $3, $4) RETURNING *"
    ).bind(req.max_total_debt).bind(req.max_single_debt).bind(req.effective_from).bind(req.effective_until).fetch_one(pool).await
}
