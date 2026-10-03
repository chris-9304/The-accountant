use sqlx::PgPool;
use chrono::NaiveDate;
use crate::models::snapshot::*;

pub async fn find_by_month(pool: &PgPool, month: NaiveDate) -> Result<Option<MonthlySnapshot>, sqlx::Error> {
    sqlx::query_as::<_, MonthlySnapshot>("SELECT * FROM monthly_snapshots WHERE month = $1").bind(month).fetch_optional(pool).await
}

pub async fn create(pool: &PgPool, s: &MonthlySnapshot) -> Result<MonthlySnapshot, sqlx::Error> {
    sqlx::query_as::<_, MonthlySnapshot>(
        "INSERT INTO monthly_snapshots (id, month, total_income, total_expenses, total_savings, net_worth, total_debt_owed, total_debt_receivable, account_balances, category_breakdown, top_expenses, health_score, budget_adherence_pct, savings_rate_pct) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14) RETURNING *"
    ).bind(s.id).bind(s.month).bind(s.total_income).bind(s.total_expenses).bind(s.total_savings).bind(s.net_worth)
     .bind(s.total_debt_owed).bind(s.total_debt_receivable).bind(&s.account_balances).bind(&s.category_breakdown)
     .bind(&s.top_expenses).bind(s.health_score).bind(s.budget_adherence_pct).bind(s.savings_rate_pct)
     .fetch_one(pool).await
}

pub async fn find_recent(pool: &PgPool, limit: i64) -> Result<Vec<MonthlySnapshot>, sqlx::Error> {
    sqlx::query_as::<_, MonthlySnapshot>("SELECT * FROM monthly_snapshots ORDER BY month DESC LIMIT $1").bind(limit).fetch_all(pool).await
}
