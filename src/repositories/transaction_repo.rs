use sqlx::PgPool;
use uuid::Uuid;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use crate::models::transaction::*;

pub async fn find_all(pool: &PgPool, f: &TransactionFilters) -> Result<(Vec<Transaction>, i64), sqlx::Error> {
    let mut where_clauses = Vec::new();
    let page = f.page.unwrap_or(1).max(1);
    let per_page = f.per_page.unwrap_or(50).min(1000);
    let offset = ((page - 1) * per_page) as i64;
    let sort_by = f.sort_by.as_deref().unwrap_or("transaction_date");
    let sort_order = f.sort_order.as_deref().unwrap_or("desc");

    let _sql = String::from("SELECT * FROM transactions WHERE 1=1");
    let _count_sql = String::from("SELECT COUNT(*) FROM transactions WHERE 1=1");
    let _param_idx = 0u32;

    // We'll build queries with dynamic SQL and bind params after
    // For simplicity, use a Vec of boxed params approach or just inline
    // Since sqlx doesn't support dynamic params easily, we'll use query builder pattern

    if f.account_id.is_some() { where_clauses.push("account_id"); }
    if f.category_id.is_some() { where_clauses.push("category_id"); }
    if f.transaction_type.is_some() { where_clauses.push("transaction_type"); }
    if f.source.is_some() { where_clauses.push("source"); }

    // Build with raw SQL for flexibility
    let rows: Vec<Transaction> = sqlx::query_as::<_, Transaction>(
        &format!(
            "SELECT * FROM transactions WHERE \
             ($1::uuid IS NULL OR account_id = $1) AND \
             ($2::uuid IS NULL OR category_id = $2) AND \
             ($3::date IS NULL OR transaction_date >= $3) AND \
             ($4::date IS NULL OR transaction_date <= $4) AND \
             ($5::decimal IS NULL OR amount >= $5) AND \
             ($6::decimal IS NULL OR amount <= $6) AND \
             ($7::text IS NULL OR transaction_type = $7) AND \
             ($8::text IS NULL OR source = $8) AND \
             ($9::text IS NULL OR LOWER(description) LIKE '%' || LOWER($9) || '%' OR LOWER(merchant_name) LIKE '%' || LOWER($9) || '%') \
             ORDER BY {} {} LIMIT {} OFFSET {}",
            match sort_by { "amount" => "amount", "created_at" => "created_at", _ => "transaction_date" },
            if sort_order == "asc" { "ASC" } else { "DESC" },
            per_page, offset
        )
    )
    .bind(f.account_id).bind(f.category_id)
    .bind(f.date_from).bind(f.date_to)
    .bind(f.amount_min).bind(f.amount_max)
    .bind(&f.transaction_type).bind(&f.source)
    .bind(&f.search)
    .fetch_all(pool).await?;

    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM transactions WHERE \
         ($1::uuid IS NULL OR account_id = $1) AND \
         ($2::uuid IS NULL OR category_id = $2) AND \
         ($3::date IS NULL OR transaction_date >= $3) AND \
         ($4::date IS NULL OR transaction_date <= $4) AND \
         ($5::decimal IS NULL OR amount >= $5) AND \
         ($6::decimal IS NULL OR amount <= $6) AND \
         ($7::text IS NULL OR transaction_type = $7) AND \
         ($8::text IS NULL OR source = $8) AND \
         ($9::text IS NULL OR LOWER(description) LIKE '%' || LOWER($9) || '%' OR LOWER(merchant_name) LIKE '%' || LOWER($9) || '%')"
    )
    .bind(f.account_id).bind(f.category_id)
    .bind(f.date_from).bind(f.date_to)
    .bind(f.amount_min).bind(f.amount_max)
    .bind(&f.transaction_type).bind(&f.source)
    .bind(&f.search)
    .fetch_one(pool).await?;

    Ok((rows, count.0))
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Transaction>, sqlx::Error> {
    sqlx::query_as::<_, Transaction>("SELECT * FROM transactions WHERE id = $1")
        .bind(id).fetch_optional(pool).await
}

pub async fn create_from_request(pool: &PgPool, req: &CreateTransactionRequest) -> Result<Transaction, sqlx::Error> {
    sqlx::query_as::<_, Transaction>(
        "INSERT INTO transactions (account_id, category_id, amount, transaction_type, description, merchant_name, reference_number, transaction_date, source, notes) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'manual', $9) RETURNING *"
    )
    .bind(req.account_id).bind(req.category_id).bind(req.amount)
    .bind(&req.transaction_type).bind(&req.description).bind(&req.merchant_name)
    .bind(&req.reference_number).bind(req.transaction_date).bind(&req.notes)
    .fetch_one(pool).await
}

pub async fn create_from_statement(pool: &PgPool, account_id: Uuid, statement_id: Uuid, category_id: Option<Uuid>,
    amount: Decimal, txn_type: &str, description: &str, merchant: Option<&str>, ref_no: Option<&str>,
    date: NaiveDate, balance_after: Option<Decimal>) -> Result<Transaction, sqlx::Error> {
    sqlx::query_as::<_, Transaction>(
        "INSERT INTO transactions (account_id, statement_id, category_id, amount, transaction_type, description, merchant_name, reference_number, transaction_date, source, balance_after) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'statement', $10) RETURNING *"
    )
    .bind(account_id).bind(statement_id).bind(category_id).bind(amount)
    .bind(txn_type).bind(description).bind(merchant).bind(ref_no)
    .bind(date).bind(balance_after)
    .fetch_one(pool).await
}

pub async fn update(pool: &PgPool, id: Uuid, req: &UpdateTransactionRequest) -> Result<Option<Transaction>, sqlx::Error> {
    let current = find_by_id(pool, id).await?;
    let Some(current) = current else { return Ok(None) };
    let category_id = req.category_id.or(current.category_id);
    let description = req.description.as_ref().or(current.description.as_ref());
    let merchant = req.merchant_name.as_ref().or(current.merchant_name.as_ref());
    let notes = req.notes.as_ref().or(current.notes.as_ref());
    sqlx::query_as::<_, Transaction>(
        "UPDATE transactions SET category_id = $1, description = $2, merchant_name = $3, notes = $4, updated_at = NOW() WHERE id = $5 RETURNING *"
    ).bind(category_id).bind(description).bind(merchant).bind(notes).bind(id)
     .fetch_optional(pool).await
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let r = sqlx::query("DELETE FROM transactions WHERE id = $1").bind(id).execute(pool).await?;
    Ok(r.rows_affected() > 0)
}

pub async fn add_tag(pool: &PgPool, transaction_id: Uuid, tag_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO transaction_tags (transaction_id, tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
        .bind(transaction_id).bind(tag_id).execute(pool).await?;
    Ok(())
}

pub async fn remove_tag(pool: &PgPool, transaction_id: Uuid, tag_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM transaction_tags WHERE transaction_id = $1 AND tag_id = $2")
        .bind(transaction_id).bind(tag_id).execute(pool).await?;
    Ok(())
}

pub async fn set_tags(pool: &PgPool, transaction_id: Uuid, tag_ids: &[Uuid]) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM transaction_tags WHERE transaction_id = $1")
        .bind(transaction_id).execute(pool).await?;
    for tag_id in tag_ids {
        add_tag(pool, transaction_id, *tag_id).await?;
    }
    Ok(())
}

pub async fn find_duplicate(pool: &PgPool, account_id: Uuid, date: NaiveDate, amount: Decimal, ref_no: Option<&str>) -> Result<bool, sqlx::Error> {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM transactions WHERE account_id = $1 AND transaction_date = $2 AND amount = $3 AND ($4::text IS NULL OR reference_number = $4)"
    ).bind(account_id).bind(date).bind(amount).bind(ref_no).fetch_one(pool).await?;
    Ok(count.0 > 0)
}

pub async fn get_spending_by_category(pool: &PgPool, from: NaiveDate, to: NaiveDate) -> Result<Vec<(String, Decimal)>, sqlx::Error> {
    let rows: Vec<(String, Decimal)> = sqlx::query_as(
        "SELECT COALESCE(c.name, 'Uncategorized'), COALESCE(SUM(t.amount), 0) \
         FROM transactions t LEFT JOIN categories c ON t.category_id = c.id \
         WHERE t.transaction_type = 'debit' AND t.transaction_date >= $1 AND t.transaction_date < $2 \
         GROUP BY c.name ORDER BY SUM(t.amount) DESC"
    ).bind(from).bind(to).fetch_all(pool).await?;
    Ok(rows)
}

pub async fn get_total_by_type(pool: &PgPool, account_id: Option<Uuid>, from: NaiveDate, to: NaiveDate, txn_type: &str) -> Result<Decimal, sqlx::Error> {
    let row: (Option<Decimal>,) = sqlx::query_as(
        "SELECT COALESCE(SUM(amount), 0) FROM transactions WHERE transaction_type = $1 AND transaction_date >= $2 AND transaction_date < $3 AND ($4::uuid IS NULL OR account_id = $4)"
    ).bind(txn_type).bind(from).bind(to).bind(account_id).fetch_one(pool).await?;
    Ok(row.0.unwrap_or(Decimal::ZERO))
}
