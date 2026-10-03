use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CashFlowForecast {
    pub id: Uuid,
    pub forecast_date: NaiveDate,
    pub projected_income: Decimal,
    pub projected_expenses: Decimal,
    pub projected_debt_payments: Decimal,
    pub projected_debt_collections: Decimal,
    pub projected_balance: Decimal,
    pub is_danger_zone: bool,
    pub danger_threshold: Option<Decimal>,
    pub confidence: Decimal,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashFlowPoint {
    pub date: NaiveDate,
    pub projected_balance: Decimal,
    pub is_danger_zone: bool,
}
