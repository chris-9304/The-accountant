use sqlx::PgPool;
use uuid::Uuid;
use rust_decimal::Decimal;
use chrono::Utc;
use crate::errors::AppError;
use crate::models::planned_purchase::*;
use crate::repositories::{purchase_repo, account_repo, transaction_repo, debt_repo};

pub async fn list_purchases(pool: &PgPool, status: Option<String>) -> Result<Vec<PlannedPurchase>, AppError> {
    Ok(purchase_repo::find_all(pool, status.as_deref()).await?)
}
pub async fn get_purchase(pool: &PgPool, id: Uuid) -> Result<PlannedPurchase, AppError> {
    purchase_repo::find_by_id(pool, id).await?.ok_or_else(|| AppError::NotFound(format!("Purchase {} not found", id)))
}
pub async fn create_purchase(pool: &PgPool, req: CreatePurchaseRequest) -> Result<PlannedPurchase, AppError> {
    if req.estimated_cost <= Decimal::ZERO { return Err(AppError::Validation("Cost must be positive".into())); }
    Ok(purchase_repo::create(pool, &req).await?)
}
pub async fn update_purchase(pool: &PgPool, id: Uuid, req: UpdatePurchaseRequest) -> Result<PlannedPurchase, AppError> {
    purchase_repo::update(pool, id, &req).await?.ok_or_else(|| AppError::NotFound(format!("Purchase {} not found", id)))
}
pub async fn delete_purchase(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    if !purchase_repo::delete(pool, id).await? { return Err(AppError::NotFound(format!("Purchase {} not found", id))); }
    Ok(())
}
pub async fn get_advice(pool: &PgPool, purchase_id: Uuid) -> Result<PurchaseAdvice, AppError> {
    let p = get_purchase(pool, purchase_id).await?;
    let cost = p.estimated_cost;
    let today = Utc::now().date_naive();
    let balance = account_repo::get_total_balance(pool).await?;
    let ago90 = today - chrono::Duration::days(90);
    let inc3 = transaction_repo::get_total_by_type(pool, None, ago90, today, "credit").await?;
    let exp3 = transaction_repo::get_total_by_type(pool, None, ago90, today, "debit").await?;
    let m_inc = inc3 / Decimal::new(3,0);
    let m_exp = exp3 / Decimal::new(3,0);
    let surplus = m_inc - m_exp;
    let emergency = m_exp * Decimal::new(3,0);
    let avail = (balance - emergency).max(Decimal::ZERO);
    let limit = debt_repo::get_current_limit(pool).await?;
    let cur_debt = debt_repo::get_total_by_type(pool, "i_owe").await?;
    let debt_cap = limit.as_ref().map(|l| (l.max_total_debt - cur_debt).max(Decimal::ZERO)).unwrap_or(Decimal::ZERO);
    let mut sources = Vec::new();
    let (strategy, rationale, rec_date);
    if avail >= cost {
        strategy = "savings";
        sources.push(FundingSource{source:"savings".into(),amount:cost,details:format!("Available: {}",avail)});
        rationale = format!("Affordable from savings. Available: {}, Cost: {}", avail, cost);
        rec_date = Some(today);
    } else if surplus > Decimal::ZERO {
        let needed = cost - avail.max(Decimal::ZERO);
        let months = ((needed / surplus).to_string().parse::<f64>().unwrap_or(12.0).ceil()) as i64;
        strategy = "income";
        sources.push(FundingSource{source:"monthly_savings".into(),amount:cost,details:format!("Save {}/mo for {} months",surplus,months)});
        rationale = format!("Save {}/month from surplus. Ready in ~{} months.",surplus,months);
        rec_date = Some(today + chrono::Duration::days(months*30));
    } else if debt_cap >= cost {
        strategy = "debt";
        sources.push(FundingSource{source:"debt".into(),amount:cost,details:format!("Debt capacity: {}",debt_cap)});
        rationale = "No surplus. Using debt capacity.".into();
        rec_date = Some(today);
    } else {
        strategy = "unaffordable";
        rationale = format!("Cannot afford. No surplus, debt capacity {} < cost {}.", debt_cap, cost);
        rec_date = None;
    };
    let dj = serde_json::to_value(&sources).unwrap_or_default();
    purchase_repo::update_advice(pool, purchase_id, rec_date, strategy, dj).await?;
    Ok(PurchaseAdvice{purchase_id, item_name:p.item_name, estimated_cost:cost, recommended_date:rec_date, funding_strategy:strategy.into(), funding_breakdown:sources, rationale})
}
