use sqlx::PgPool;
use uuid::Uuid;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use crate::models::statement::*;

pub async fn find_all(pool: &PgPool) -> Result<Vec<Statement>, sqlx::Error> {
    sqlx::query_as::<_, Statement>("SELECT * FROM statements ORDER BY created_at DESC").fetch_all(pool).await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Statement>, sqlx::Error> {
    sqlx::query_as::<_, Statement>("SELECT * FROM statements WHERE id = $1").bind(id).fetch_optional(pool).await
}

pub async fn find_by_hash(pool: &PgPool, hash: &str) -> Result<Option<Statement>, sqlx::Error> {
    sqlx::query_as::<_, Statement>("SELECT * FROM statements WHERE file_hash = $1").bind(hash).fetch_optional(pool).await
}

pub async fn create(pool: &PgPool, account_id: Uuid, file_name: &str, file_path: &str, file_format: &str, file_hash: &str) -> Result<Statement, sqlx::Error> {
    sqlx::query_as::<_, Statement>(
        "INSERT INTO statements (account_id, file_name, file_path, file_format, file_hash, status) VALUES ($1, $2, $3, $4, $5, 'processing') RETURNING *"
    ).bind(account_id).bind(file_name).bind(file_path).bind(file_format).bind(file_hash).fetch_one(pool).await
}

pub async fn update_status(pool: &PgPool, id: Uuid, status: &str, error_message: Option<&str>) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE statements SET status = $1, error_message = $2 WHERE id = $3")
        .bind(status).bind(error_message).bind(id).execute(pool).await?;
    Ok(())
}

pub async fn update_parsed(pool: &PgPool, id: Uuid, bank_format: &str, period_start: Option<NaiveDate>, period_end: Option<NaiveDate>,
    total_transactions: i32, total_credits: Decimal, total_debits: Decimal) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE statements SET bank_format = $1, period_start = $2, period_end = $3, total_transactions = $4, total_credits = $5, total_debits = $6, status = 'completed', parsed_at = NOW() WHERE id = $7")
        .bind(bank_format).bind(period_start).bind(period_end).bind(total_transactions).bind(total_credits).bind(total_debits).bind(id)
        .execute(pool).await?;
    Ok(())
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    sqlx::query("DELETE FROM transactions WHERE statement_id = $1").bind(id).execute(pool).await?;
    let r = sqlx::query("DELETE FROM statements WHERE id = $1").bind(id).execute(pool).await?;
    Ok(r.rows_affected() > 0)
}
