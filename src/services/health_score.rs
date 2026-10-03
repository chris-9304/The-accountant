use sqlx::PgPool;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use chrono::{Utc, NaiveDate, Datelike};
use crate::errors::AppError;
use crate::models::health_score::HealthScore;
use crate::repositories::{account_repo, debt_repo, transaction_repo, budget_repo, health_score_repo};

fn d2i(d: Decimal) -> i32 { d.to_string().parse::<f64>().unwrap_or(0.0) as i32 }

pub async fn calculate_health_score(pool: &PgPool) -> Result<HealthScore, AppError> {
    let today = Utc::now().date_naive();
    let ms = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).ok_or_else(|| AppError::Internal(anyhow::anyhow!("date")))?;
    let me = if ms.month()==12 { NaiveDate::from_ymd_opt(ms.year()+1,1,1) } else { NaiveDate::from_ymd_opt(ms.year(), ms.month()+1,1) }.ok_or_else(|| AppError::Internal(anyhow::anyhow!("date")))?;
    let income = transaction_repo::get_total_by_type(pool, None, ms, me, "credit").await?;
    let expenses = transaction_repo::get_total_by_type(pool, None, ms, me, "debit").await?;
    let savings = account_repo::get_total_balance(pool).await?;
    let debt_owed = debt_repo::get_total_by_type(pool, "i_owe").await?;
    let debt_recv = debt_repo::get_total_by_type(pool, "owed_to_me").await?;
    let budget = budget_repo::find_by_month(pool, ms).await?;
    let blimit = budget.map(|b| b.total_spending_limit).unwrap_or(Decimal::ZERO);

    let sr = if income > Decimal::ZERO { ((income - expenses) / income * dec!(100)).min(dec!(100)).max(dec!(0)) } else { Decimal::ZERO };
    let srs = (sr / dec!(20) * dec!(100)).min(dec!(100)).max(dec!(0));
    let drs = if income > Decimal::ZERO { ((dec!(1) - debt_owed / income) * dec!(100)).max(dec!(0)).min(dec!(100)) } else if debt_owed == Decimal::ZERO { dec!(100) } else { dec!(0) };
    let bas = if blimit > Decimal::ZERO { let r = expenses / blimit; if r <= dec!(1) { dec!(100) } else { ((dec!(1.5) - r) / dec!(0.5) * dec!(100)).max(dec!(0)) } } else { dec!(50) };
    let mc = if expenses > Decimal::ZERO { savings / expenses } else { dec!(6) };
    let efs = (mc / dec!(3) * dec!(100)).min(dec!(100)).max(dec!(0));
    let dcs = if debt_recv > Decimal::ZERO { dec!(50) } else { dec!(100) };
    let overall = (srs * dec!(25) + drs * dec!(25) + bas * dec!(20) + efs * dec!(15) + dcs * dec!(15)) / dec!(100);
    let s = HealthScore { id: uuid::Uuid::new_v4(), overall_score: d2i(overall), savings_rate_score: d2i(srs), debt_ratio_score: d2i(drs), budget_adherence_score: d2i(bas), emergency_fund_score: d2i(efs), debt_collection_score: d2i(dcs),
        details: serde_json::json!({"income":income.to_string(),"expenses":expenses.to_string(),"savings":savings.to_string()}), calculated_at: Utc::now() };
    let saved = health_score_repo::create(pool, &s).await?;
    Ok(saved)
}
