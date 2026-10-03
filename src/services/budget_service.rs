use sqlx::PgPool;
use uuid::Uuid;
use rust_decimal::Decimal;
use chrono::{Utc, NaiveDate, Datelike};
use crate::errors::AppError;
use crate::models::budget::*;
use crate::repositories::{budget_repo, transaction_repo, category_repo};

pub async fn list_budgets(pool: &PgPool) -> Result<Vec<Budget>, AppError> { Ok(budget_repo::find_all(pool).await?) }
pub async fn get_budget(pool: &PgPool, id: Uuid) -> Result<Budget, AppError> {
    budget_repo::find_by_id(pool, id).await?.ok_or_else(|| AppError::NotFound(format!("Budget {} not found", id)))
}

pub async fn get_current_budget(pool: &PgPool) -> Result<Option<BudgetProgress>, AppError> {
    let today = Utc::now().date_naive();
    let ms = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).ok_or_else(|| AppError::Internal(anyhow::anyhow!("date")))?;
    if let Some(b) = budget_repo::find_by_month(pool, ms).await? { Ok(Some(calc_progress(pool, &b).await?)) } else { Ok(None) }
}

pub async fn create_budget(pool: &PgPool, req: CreateBudgetRequest) -> Result<Budget, AppError> {
    if req.total_spending_limit <= Decimal::ZERO { return Err(AppError::Validation("Limit must be positive".into())); }
    if budget_repo::find_by_month(pool, req.month).await?.is_some() { return Err(AppError::Duplicate(format!("Budget for {} exists", req.month))); }
    let b = budget_repo::create(pool, &req).await?;
    if let Some(cats) = &req.categories { budget_repo::set_categories(pool, b.id, cats).await?; }
    Ok(b)
}

pub async fn update_budget(pool: &PgPool, id: Uuid, req: UpdateBudgetRequest) -> Result<Budget, AppError> {
    let cats = req.categories.clone();
    let b = budget_repo::update(pool, id, &req).await?.ok_or_else(|| AppError::NotFound(format!("Budget {} not found", id)))?;
    if let Some(c) = cats { budget_repo::set_categories(pool, id, &c).await?; }
    Ok(b)
}

pub async fn delete_budget(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    if !budget_repo::delete(pool, id).await? { return Err(AppError::NotFound(format!("Budget {} not found", id))); }
    Ok(())
}

pub async fn get_progress(pool: &PgPool, budget_id: Uuid) -> Result<BudgetProgress, AppError> {
    let b = get_budget(pool, budget_id).await?;
    calc_progress(pool, &b).await
}

async fn calc_progress(pool: &PgPool, budget: &Budget) -> Result<BudgetProgress, AppError> {
    let ms = budget.month;
    let me = if ms.month() == 12 { NaiveDate::from_ymd_opt(ms.year()+1,1,1) } else { NaiveDate::from_ymd_opt(ms.year(), ms.month()+1,1) }
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("date")))?;
    let spent = transaction_repo::get_total_by_type(pool, None, ms, me, "debit").await?;
    let remaining = budget.total_spending_limit - spent;
    let pct = if budget.total_spending_limit > Decimal::ZERO { (spent / budget.total_spending_limit) * Decimal::new(100,0) } else { Decimal::ZERO };
    let bcs = budget_repo::find_categories(pool, budget.id).await?;
    let cat_spending = transaction_repo::get_spending_by_category(pool, ms, me).await?;
    let all_cats = category_repo::find_all(pool).await?;
    let cp: Vec<CategoryBudgetProgress> = bcs.iter().map(|bc| {
        let name = all_cats.iter().find(|c| c.id == bc.category_id).map(|c| c.name.clone()).unwrap_or_else(|| "Unknown".into());
        let s = cat_spending.iter().find(|(n,_)| n == &name).map(|(_,a)| *a).unwrap_or(Decimal::ZERO);
        let r = bc.allocated_amount - s;
        let p = if bc.allocated_amount > Decimal::ZERO { (s / bc.allocated_amount) * Decimal::new(100,0) } else { Decimal::ZERO };
        CategoryBudgetProgress { category_id: bc.category_id, category_name: name, allocated: bc.allocated_amount, spent: s, remaining: r, percentage_used: p }
    }).collect();
    let income = transaction_repo::get_total_by_type(pool, None, ms, me, "credit").await?;
    Ok(BudgetProgress { budget: budget.clone(), total_spent: spent, remaining, percentage_used: pct, category_progress: cp, savings_actual: income - spent })
}
