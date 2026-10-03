use sqlx::PgPool;
use uuid::Uuid;
use chrono::NaiveDate;
use crate::errors::AppError;
use crate::models::transaction::TransactionFilters;
use crate::repositories::{transaction_repo, category_repo, account_repo, snapshot_repo, debt_repo};

pub async fn export_csv(pool: &PgPool, from: NaiveDate, to: NaiveDate, account_id: Option<Uuid>) -> Result<String, AppError> {
    let f = TransactionFilters { account_id, date_from: Some(from), date_to: Some(to), per_page: Some(100000), ..Default::default() };
    let (txns, _) = transaction_repo::find_all(pool, &f).await?;
    let cats = category_repo::find_all(pool).await?;
    let accs = account_repo::find_all(pool).await?;
    let mut csv = String::from("Date,Description,Category,Amount,Type,Account,Merchant,Reference\n");
    for t in &txns {
        let cn = t.category_id.and_then(|cid| cats.iter().find(|c| c.id == cid)).map(|c| c.name.as_str()).unwrap_or("Uncategorized");
        let an = accs.iter().find(|a| a.id == t.account_id).map(|a| a.name.as_str()).unwrap_or("Unknown");
        csv.push_str(&format!("{},\"{}\",\"{}\",{},{},\"{}\",\"{}\",{}\n", t.transaction_date, t.description.as_deref().unwrap_or(""), cn, t.amount, t.transaction_type, an, t.merchant_name.as_deref().unwrap_or(""), t.reference_number.as_deref().unwrap_or("")));
    }
    Ok(csv)
}

pub async fn export_monthly_report(pool: &PgPool, month: NaiveDate) -> Result<String, AppError> {
    let snap = snapshot_repo::find_by_month(pool, month).await?;
    let accs = account_repo::find_all(pool).await?;
    let do_ = debt_repo::get_total_by_type(pool, "i_owe").await?;
    let dr = debt_repo::get_total_by_type(pool, "owed_to_me").await?;
    let mut r = format!("=== THE ACCOUNTANT - {} ===\n\n", month.format("%B %Y"));
    if let Some(s) = &snap { r.push_str(&format!("Income: {}\nExpenses: {}\nSavings: {}\nNet Worth: {}\nHealth: {}/100\n\n", s.total_income, s.total_expenses, s.total_savings, s.net_worth, s.health_score.unwrap_or(0))); }
    r.push_str("-- Accounts --\n"); for a in &accs { if a.is_active { r.push_str(&format!("  {}: {}\n", a.name, a.balance)); } }
    r.push_str(&format!("\n-- Debt --\n  I Owe: {}\n  Owed to Me: {}\n  Net: {}\n", do_, dr, dr - do_));
    Ok(r)
}
