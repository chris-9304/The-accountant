use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Budget {
    pub id: Uuid,
    pub month: NaiveDate,
    pub total_spending_limit: Decimal,
    pub savings_target: Decimal,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBudgetRequest {
    pub month: NaiveDate,
    pub total_spending_limit: Decimal,
    pub savings_target: Decimal,
    pub notes: Option<String>,
    pub categories: Option<Vec<BudgetCategoryAllocation>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateBudgetRequest {
    pub total_spending_limit: Option<Decimal>,
    pub savings_target: Option<Decimal>,
    pub notes: Option<String>,
    pub categories: Option<Vec<BudgetCategoryAllocation>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BudgetCategory {
    pub id: Uuid,
    pub budget_id: Uuid,
    pub category_id: Uuid,
    pub allocated_amount: Decimal,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetCategoryAllocation {
    pub category_id: Uuid,
    pub allocated_amount: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetProgress {
    pub budget: Budget,
    pub total_spent: Decimal,
    pub remaining: Decimal,
    pub percentage_used: Decimal,
    pub category_progress: Vec<CategoryBudgetProgress>,
    pub savings_actual: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryBudgetProgress {
    pub category_id: Uuid,
    pub category_name: String,
    pub allocated: Decimal,
    pub spent: Decimal,
    pub remaining: Decimal,
    pub percentage_used: Decimal,
}
