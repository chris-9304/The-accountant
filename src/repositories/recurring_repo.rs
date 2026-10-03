use sqlx::PgPool;
use uuid::Uuid;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use crate::models::recurring_expense::*;

pub async fn find_all(pool: &PgPool, active_only: bool) -> Result<Vec<RecurringExpense>, sqlx::Error> {
    if active_only {
        sqlx::query_as::<_, RecurringExpense>("SELECT * FROM recurring_expenses WHERE is_active = true ORDER BY average_amount DESC").fetch_all(pool).await
    } else {
        sqlx::query_as::<_, RecurringExpense>("SELECT * FROM recurring_expenses ORDER BY average_amount DESC").fetch_all(pool).await
    }
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<RecurringExpense>, sqlx::Error> {
    sqlx::query_as::<_, RecurringExpense>("SELECT * FROM recurring_expenses WHERE id = $1").bind(id).fetch_optional(pool).await
}

pub async fn find_by_merchant(pool: &PgPool, merchant: &str) -> Result<Option<RecurringExpense>, sqlx::Error> {
    sqlx::query_as::<_, RecurringExpense>("SELECT * FROM recurring_expenses WHERE LOWER(merchant_name) = LOWER($1)")
        .bind(merchant).fetch_optional(pool).await
}

pub async fn create(pool: &PgPool, e: &RecurringExpense) -> Result<RecurringExpense, sqlx::Error> {
    sqlx::query_as::<_, RecurringExpense>(
        "INSERT INTO recurring_expenses (id, merchant_name, category_id, average_amount, min_amount, max_amount, frequency, last_seen, next_expected, account_id, occurrence_count, is_active, confidence) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13) RETURNING *"
    ).bind(e.id).bind(&e.merchant_name).bind(e.category_id).bind(e.average_amount).bind(e.min_amount).bind(e.max_amount)
     .bind(&e.frequency).bind(e.last_seen).bind(e.next_expected).bind(e.account_id).bind(e.occurrence_count).bind(e.is_active).bind(e.confidence)
     .fetch_one(pool).await
}

pub async fn update(pool: &PgPool, id: Uuid, req: &UpdateRecurringRequest) -> Result<Option<RecurringExpense>, sqlx::Error> {
    let current = find_by_id(pool, id).await?;
    let Some(c) = current else { return Ok(None) };
    let category_id = req.category_id.or(c.category_id);
    let is_active = req.is_active.unwrap_or(c.is_active);
    let frequency = req.frequency.as_deref().unwrap_or(&c.frequency);
    sqlx::query_as::<_, RecurringExpense>(
        "UPDATE recurring_expenses SET category_id=$1, is_active=$2, frequency=$3, updated_at=NOW() WHERE id=$4 RETURNING *"
    ).bind(category_id).bind(is_active).bind(frequency).bind(id).fetch_optional(pool).await
}

pub async fn update_detection(pool: &PgPool, id: Uuid, avg: Decimal, min: Decimal, max: Decimal,
    count: i32, last_seen: NaiveDate, next_expected: NaiveDate, confidence: Decimal) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE recurring_expenses SET average_amount=$1, min_amount=$2, max_amount=$3, occurrence_count=$4, last_seen=$5, next_expected=$6, confidence=$7, updated_at=NOW() WHERE id=$8"
    ).bind(avg).bind(min).bind(max).bind(count).bind(last_seen).bind(next_expected).bind(confidence).bind(id).execute(pool).await?;
    Ok(())
}
