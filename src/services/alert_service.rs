use sqlx::PgPool;
use uuid::Uuid;
use rust_decimal::Decimal;
use chrono::{Utc, NaiveDate, Datelike};
use crate::errors::AppError;
use crate::models::alert::*;
use crate::repositories::{alert_repo, debt_repo, budget_repo, account_repo, transaction_repo};

pub async fn check_and_generate_alerts(pool: &PgPool) -> Result<Vec<Alert>, AppError> {
    let mut alerts = Vec::new();
    let today = Utc::now().date_naive();
    for d in debt_repo::find_upcoming_deadlines(pool, 7).await? {
        if let Some(dl) = d.deadline {
            let days = (dl - today).num_days();
            let sev = if days <= 2 { "critical" } else { "warning" };
            alerts.push(alert_repo::create(pool, "debt_deadline", &format!("Debt due in {} days", days), &format!("{} {} - {}", d.remaining_amount, d.counterparty, dl), sev, Some("debt"), Some(d.id)).await?);
        }
    }
    for d in debt_repo::find_overdue(pool).await? {
        alerts.push(alert_repo::create(pool, "debt_overdue", "Debt overdue!", &format!("{} {} overdue", d.remaining_amount, d.counterparty), "critical", Some("debt"), Some(d.id)).await?);
    }
    let ms = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap_or(today);
    let me = if ms.month()==12 { NaiveDate::from_ymd_opt(ms.year()+1,1,1) } else { NaiveDate::from_ymd_opt(ms.year(), ms.month()+1,1) }.unwrap_or(today);
    if let Some(b) = budget_repo::find_by_month(pool, ms).await? {
        let spent = transaction_repo::get_total_by_type(pool, None, ms, me, "debit").await?;
        let pct = if b.total_spending_limit > Decimal::ZERO { spent / b.total_spending_limit * Decimal::new(100,0) } else { Decimal::ZERO };
        if pct > Decimal::new(100,0) { alerts.push(alert_repo::create(pool, "budget_overrun", "Budget exceeded!", &format!("Spent {} of {} ({}%)", spent, b.total_spending_limit, pct), "critical", Some("budget"), Some(b.id)).await?); }
        else if pct > Decimal::new(80,0) { alerts.push(alert_repo::create(pool, "budget_warning", "Budget warning", &format!("Spent {} of {} ({}%)", spent, b.total_spending_limit, pct), "warning", Some("budget"), Some(b.id)).await?); }
    }
    let thr = Decimal::new(5000,0);
    for a in account_repo::find_all(pool).await? { if a.is_active && a.balance < thr { alerts.push(alert_repo::create(pool, "low_balance", &format!("Low balance: {}", a.name), &format!("{} balance: {}", a.name, a.balance), "warning", Some("account"), Some(a.id)).await?); } }
    if let Some(l) = debt_repo::get_current_limit(pool).await? {
        let cur = debt_repo::get_total_by_type(pool, "i_owe").await?;
        let p = if l.max_total_debt > Decimal::ZERO { cur / l.max_total_debt * Decimal::new(100,0) } else { Decimal::ZERO };
        if p > Decimal::new(80,0) { alerts.push(alert_repo::create(pool, "debt_limit_warning", "Near debt limit", &format!("Debt {} of {} ({}%)", cur, l.max_total_debt, p), "warning", None, None).await?); }
    }
    Ok(alerts)
}

pub async fn list_alerts(pool: &PgPool, f: AlertFilters) -> Result<(Vec<Alert>, i64), AppError> { Ok(alert_repo::find_all(pool, &f).await?) }
pub async fn mark_read(pool: &PgPool, id: Uuid) -> Result<(), AppError> { Ok(alert_repo::mark_read(pool, id).await?) }
pub async fn mark_dismissed(pool: &PgPool, id: Uuid) -> Result<(), AppError> { Ok(alert_repo::mark_dismissed(pool, id).await?) }
pub async fn mark_all_read(pool: &PgPool) -> Result<(), AppError> { Ok(alert_repo::mark_all_read(pool).await?) }
pub async fn count_unread(pool: &PgPool) -> Result<i64, AppError> { Ok(alert_repo::count_unread(pool).await?) }
