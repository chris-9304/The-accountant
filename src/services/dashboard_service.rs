use sqlx::PgPool;
use rust_decimal::Decimal;
use chrono::{Utc, NaiveDate, Datelike};
use crate::errors::AppError;
use crate::models::dashboard::*;
use crate::models::account::AccountSummary;
use crate::models::transaction::TransactionFilters;
use crate::repositories::{account_repo, debt_repo, budget_repo, transaction_repo, alert_repo, health_score_repo};

pub async fn get_dashboard(pool: &PgPool) -> Result<DashboardResponse, AppError> {
    let today = Utc::now().date_naive();
    let ms = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap_or(today);
    let me = if ms.month()==12 { NaiveDate::from_ymd_opt(ms.year()+1,1,1) } else { NaiveDate::from_ymd_opt(ms.year(), ms.month()+1,1) }.unwrap_or(today);
    let all = account_repo::find_all(pool).await?;
    let bal = account_repo::get_total_balance(pool).await?;
    let accounts: Vec<AccountSummary> = all.iter().filter(|a|a.is_active).map(|a| AccountSummary{id:a.id,name:a.name.clone(),account_type:a.account_type.clone(),balance:a.balance}).collect();
    let do_ = debt_repo::get_total_by_type(pool, "i_owe").await?;
    let dr = debt_repo::get_total_by_type(pool, "owed_to_me").await?;
    let net = dr - do_; let proj = bal + net;
    let b = budget_repo::find_by_month(pool, ms).await?;
    let spent = transaction_repo::get_total_by_type(pool, None, ms, me, "debit").await?;
    let income = transaction_repo::get_total_by_type(pool, None, ms, me, "credit").await?;
    let blim = b.as_ref().map(|x|x.total_spending_limit).unwrap_or(Decimal::ZERO);
    let st = b.as_ref().map(|x|x.savings_target).unwrap_or(Decimal::ZERO);
    let lms = if ms.month()==1 { NaiveDate::from_ymd_opt(ms.year()-1,12,1) } else { NaiveDate::from_ymd_opt(ms.year(), ms.month()-1,1) }.unwrap_or(today);
    let last_spent = transaction_repo::get_total_by_type(pool, None, lms, ms, "debit").await?;
    let chg = if last_spent > Decimal::ZERO { (spent - last_spent) / last_spent * Decimal::new(100,0) } else { Decimal::ZERO };
    let cs = transaction_repo::get_spending_by_category(pool, ms, me).await?;
    let tsc: Vec<CategorySpending> = cs.iter().take(5).map(|(n,a)| { let p = if spent > Decimal::ZERO { *a / spent * Decimal::new(100,0) } else { Decimal::ZERO }; CategorySpending{category_name:n.clone(),amount:*a,percentage:p} }).collect();
    let f = TransactionFilters { per_page: Some(10), sort_by: Some("transaction_date".into()), sort_order: Some("desc".into()), ..Default::default() };
    let (recent, _) = transaction_repo::find_all(pool, &f).await?;
    let hs = health_score_repo::find_latest(pool).await?.map(|h|h.overall_score);
    let ua = alert_repo::count_unread(pool).await?;
    let lim = debt_repo::get_current_limit(pool).await?;
    Ok(DashboardResponse { total_balance:bal, accounts, total_debt_owed:do_, total_debt_receivable:dr, net_debt_position:net, projected_balance_after_debts:proj, budget_spent:spent, budget_limit:blim, budget_remaining:blim-spent, savings_target:st, savings_actual:income-spent, health_score:hs, spending_this_month:spent, spending_last_month:last_spent, spending_change_pct:chg, top_spending_categories:tsc, recent_transactions:recent, unread_alert_count:ua, debt_limit:lim.as_ref().map(|l|l.max_total_debt), current_total_debt:do_, available_debt_capacity:lim.map(|l|(l.max_total_debt-do_).max(Decimal::ZERO)) })
}
