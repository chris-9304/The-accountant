use sqlx::PgPool;
use rust_decimal::Decimal;
use chrono::{Utc, NaiveDate, Datelike};
use crate::errors::AppError;
use crate::models::snapshot::MonthlySnapshot;
use crate::repositories::{snapshot_repo, account_repo, transaction_repo, debt_repo};
use crate::services::health_score;

pub async fn generate_snapshot(pool: &PgPool, month: NaiveDate) -> Result<MonthlySnapshot, AppError> {
    let ms = NaiveDate::from_ymd_opt(month.year(), month.month(), 1).ok_or_else(|| AppError::Internal(anyhow::anyhow!("date")))?;
    let me = if ms.month()==12 { NaiveDate::from_ymd_opt(ms.year()+1,1,1) } else { NaiveDate::from_ymd_opt(ms.year(), ms.month()+1,1) }.ok_or_else(|| AppError::Internal(anyhow::anyhow!("date")))?;
    let income = transaction_repo::get_total_by_type(pool, None, ms, me, "credit").await?;
    let expenses = transaction_repo::get_total_by_type(pool, None, ms, me, "debit").await?;
    let accs = account_repo::find_all(pool).await?;
    let mut ab = serde_json::Map::new(); for a in &accs { ab.insert(a.name.clone(), serde_json::Value::String(a.balance.to_string())); }
    let cs = transaction_repo::get_spending_by_category(pool, ms, me).await?;
    let mut cb = serde_json::Map::new(); for (c,a) in &cs { cb.insert(c.clone(), serde_json::Value::String(a.to_string())); }
    let bal = account_repo::get_total_balance(pool).await?;
    let do_ = debt_repo::get_total_by_type(pool, "i_owe").await?;
    let dr = debt_repo::get_total_by_type(pool, "owed_to_me").await?;
    let h = health_score::calculate_health_score(pool).await.ok();
    let sr = if income > Decimal::ZERO { Some((income - expenses) / income * Decimal::new(100,0)) } else { None };
    let s = MonthlySnapshot { id: uuid::Uuid::new_v4(), month: ms, total_income: income, total_expenses: expenses, total_savings: income - expenses, net_worth: bal - do_ + dr, total_debt_owed: do_, total_debt_receivable: dr, account_balances: serde_json::Value::Object(ab), category_breakdown: serde_json::Value::Object(cb), top_expenses: serde_json::Value::Array(vec![]), health_score: h.map(|x|x.overall_score), budget_adherence_pct: None, savings_rate_pct: sr, generated_at: Utc::now() };
    Ok(snapshot_repo::create(pool, &s).await?)
}
