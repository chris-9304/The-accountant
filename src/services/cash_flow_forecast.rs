use sqlx::PgPool;
use rust_decimal::Decimal;
use chrono::Utc;
use crate::errors::AppError;
use crate::models::cash_flow::CashFlowForecast;
use crate::repositories::{account_repo, debt_repo, transaction_repo, forecast_repo};

pub async fn generate_forecast(pool: &PgPool, days: u32) -> Result<Vec<CashFlowForecast>, AppError> {
    let today = Utc::now().date_naive();
    let end = today + chrono::Duration::days(days as i64);
    let mut bal = account_repo::get_total_balance(pool).await?;
    let ago90 = today - chrono::Duration::days(90);
    let exp90 = transaction_repo::get_total_by_type(pool, None, ago90, today, "debit").await?;
    let inc90 = transaction_repo::get_total_by_type(pool, None, ago90, today, "credit").await?;
    let daily_exp = exp90 / Decimal::new(90,0);
    let daily_inc = inc90 / Decimal::new(90,0);
    let debts_owe = debt_repo::find_all(pool, Some("i_owe"), Some("active")).await?;
    let debts_recv = debt_repo::find_all(pool, Some("owed_to_me"), Some("active")).await?;
    let threshold = Decimal::new(5000,0);
    forecast_repo::delete_future(pool, today).await?;
    let mut forecasts = Vec::new();
    let mut cur = today + chrono::Duration::days(1);
    while cur <= end {
        let mut dp = Decimal::ZERO; let mut dc = Decimal::ZERO;
        for d in &debts_owe { if d.deadline == Some(cur) { dp += d.remaining_amount; } }
        for d in &debts_recv { if d.deadline == Some(cur) { dc += d.remaining_amount; } }
        bal = bal + daily_inc + dc - daily_exp - dp;
        let days_out = (cur - today).num_days() as f64;
        let conf = Decimal::new(((1.0 - days_out / (days as f64 * 1.5)).max(0.1) * 100.0) as i64, 2);
        forecasts.push(CashFlowForecast { id: uuid::Uuid::new_v4(), forecast_date: cur, projected_income: daily_inc, projected_expenses: daily_exp, projected_debt_payments: dp, projected_debt_collections: dc, projected_balance: bal, is_danger_zone: bal < threshold, danger_threshold: Some(threshold), confidence: conf, generated_at: Utc::now() });
        cur += chrono::Duration::days(1);
    }
    forecast_repo::create_batch(pool, &forecasts).await?;
    Ok(forecasts)
}
