use sqlx::PgPool;
use uuid::Uuid;
use chrono::NaiveDate;
use crate::models::budget::*;

pub async fn find_all(pool: &PgPool) -> Result<Vec<Budget>, sqlx::Error> {
    sqlx::query_as::<_, Budget>("SELECT * FROM budgets ORDER BY month DESC").fetch_all(pool).await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Budget>, sqlx::Error> {
    sqlx::query_as::<_, Budget>("SELECT * FROM budgets WHERE id = $1").bind(id).fetch_optional(pool).await
}

pub async fn find_by_month(pool: &PgPool, month: NaiveDate) -> Result<Option<Budget>, sqlx::Error> {
    sqlx::query_as::<_, Budget>("SELECT * FROM budgets WHERE month = $1").bind(month).fetch_optional(pool).await
}

pub async fn create(pool: &PgPool, req: &CreateBudgetRequest) -> Result<Budget, sqlx::Error> {
    sqlx::query_as::<_, Budget>(
        "INSERT INTO budgets (month, total_spending_limit, savings_target, notes) VALUES ($1, $2, $3, $4) RETURNING *"
    ).bind(req.month).bind(req.total_spending_limit).bind(req.savings_target).bind(&req.notes).fetch_one(pool).await
}

pub async fn update(pool: &PgPool, id: Uuid, req: &UpdateBudgetRequest) -> Result<Option<Budget>, sqlx::Error> {
    let current = find_by_id(pool, id).await?;
    let Some(current) = current else { return Ok(None) };
    let limit = req.total_spending_limit.unwrap_or(current.total_spending_limit);
    let savings = req.savings_target.unwrap_or(current.savings_target);
    let notes = req.notes.as_ref().or(current.notes.as_ref());
    sqlx::query_as::<_, Budget>(
        "UPDATE budgets SET total_spending_limit=$1, savings_target=$2, notes=$3, updated_at=NOW() WHERE id=$4 RETURNING *"
    ).bind(limit).bind(savings).bind(notes).bind(id).fetch_optional(pool).await
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let r = sqlx::query("DELETE FROM budgets WHERE id = $1").bind(id).execute(pool).await?;
    Ok(r.rows_affected() > 0)
}

pub async fn find_categories(pool: &PgPool, budget_id: Uuid) -> Result<Vec<BudgetCategory>, sqlx::Error> {
    sqlx::query_as::<_, BudgetCategory>("SELECT * FROM budget_categories WHERE budget_id = $1")
        .bind(budget_id).fetch_all(pool).await
}

pub async fn set_categories(pool: &PgPool, budget_id: Uuid, allocations: &[BudgetCategoryAllocation]) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM budget_categories WHERE budget_id = $1").bind(budget_id).execute(pool).await?;
    for a in allocations {
        sqlx::query("INSERT INTO budget_categories (budget_id, category_id, allocated_amount) VALUES ($1, $2, $3)")
            .bind(budget_id).bind(a.category_id).bind(a.allocated_amount).execute(pool).await?;
    }
    Ok(())
}
