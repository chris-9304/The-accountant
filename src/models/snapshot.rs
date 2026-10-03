use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MonthlySnapshot {
    pub id: Uuid,
    pub month: NaiveDate,
    pub total_income: Decimal,
    pub total_expenses: Decimal,
    pub total_savings: Decimal,
    pub net_worth: Decimal,
    pub total_debt_owed: Decimal,
    pub total_debt_receivable: Decimal,
    pub account_balances: serde_json::Value,
    pub category_breakdown: serde_json::Value,
    pub top_expenses: serde_json::Value,
    pub health_score: Option<i32>,
    pub budget_adherence_pct: Option<Decimal>,
    pub savings_rate_pct: Option<Decimal>,
    pub generated_at: DateTime<Utc>,
}
