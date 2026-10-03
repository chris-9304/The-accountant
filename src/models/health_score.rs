use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HealthScore {
    pub id: Uuid,
    pub overall_score: i32,
    pub savings_rate_score: i32,
    pub debt_ratio_score: i32,
    pub budget_adherence_score: i32,
    pub emergency_fund_score: i32,
    pub debt_collection_score: i32,
    pub details: serde_json::Value,
    pub calculated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthScoreInput {
    pub monthly_income: Decimal,
    pub monthly_expenses: Decimal,
    pub total_savings: Decimal,
    pub total_debt_owed: Decimal,
    pub total_debt_receivable: Decimal,
    pub collected_this_month: Decimal,
    pub budget_limit: Decimal,
    pub actual_spending: Decimal,
}
