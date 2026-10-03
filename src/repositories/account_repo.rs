use sqlx::PgPool;
use uuid::Uuid;
use rust_decimal::Decimal;
use crate::models::account::*;

pub async fn find_all(pool: &PgPool) -> Result<Vec<Account>, sqlx::Error> {
    sqlx::query_as::<_, Account>("SELECT * FROM accounts ORDER BY created_at")
        .fetch_all(pool).await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Account>, sqlx::Error> {
    sqlx::query_as::<_, Account>("SELECT * FROM accounts WHERE id = $1")
        .bind(id).fetch_optional(pool).await
}

pub async fn create(pool: &PgPool, req: &CreateAccountRequest) -> Result<Account, sqlx::Error> {
    let balance = req.initial_balance.unwrap_or(Decimal::ZERO);
    let currency = req.currency.as_deref().unwrap_or("INR");
    sqlx::query_as::<_, Account>(
        "INSERT INTO accounts (name, account_type, balance, currency, bank_name, account_number) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"
    ).bind(&req.name).bind(&req.account_type).bind(balance).bind(currency)
     .bind(&req.bank_name).bind(&req.account_number)
     .fetch_one(pool).await
}

pub async fn update(pool: &PgPool, id: Uuid, req: &UpdateAccountRequest) -> Result<Option<Account>, sqlx::Error> {
    let current = find_by_id(pool, id).await?;
    let Some(current) = current else { return Ok(None) };
    let name = req.name.as_deref().unwrap_or(&current.name);
    let is_active = req.is_active.unwrap_or(current.is_active);
    sqlx::query_as::<_, Account>(
        "UPDATE accounts SET name = $1, is_active = $2, updated_at = NOW() WHERE id = $3 RETURNING *"
    ).bind(name).bind(is_active).bind(id).fetch_optional(pool).await
}

pub async fn update_balance(pool: &PgPool, id: Uuid, balance: Decimal) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE accounts SET balance = $1, updated_at = NOW() WHERE id = $2")
        .bind(balance).bind(id).execute(pool).await?;
    Ok(())
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let r = sqlx::query("UPDATE accounts SET is_active = false, updated_at = NOW() WHERE id = $1")
        .bind(id).execute(pool).await?;
    Ok(r.rows_affected() > 0)
}

pub async fn get_total_balance(pool: &PgPool) -> Result<Decimal, sqlx::Error> {
    let row: (Option<Decimal>,) = sqlx::query_as("SELECT COALESCE(SUM(balance), 0) FROM accounts WHERE is_active = true")
        .fetch_one(pool).await?;
    Ok(row.0.unwrap_or(Decimal::ZERO))
}
