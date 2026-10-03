use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Debt {
    pub id: Uuid,
    pub counterparty: String,
    pub amount: Decimal,
    pub remaining_amount: Decimal,
    pub debt_type: String,
    pub reason: Option<String>,
    pub deadline: Option<NaiveDate>,
    pub interest_rate: Decimal,
    pub status: String,
    pub priority: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDebtRequest {
    pub counterparty: String,
    pub amount: Decimal,
    pub debt_type: String,
    pub reason: Option<String>,
    pub deadline: Option<NaiveDate>,
    pub interest_rate: Option<Decimal>,
    pub priority: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDebtRequest {
    pub counterparty: Option<String>,
    pub reason: Option<String>,
    pub deadline: Option<NaiveDate>,
    pub priority: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DebtPayment {
    pub id: Uuid,
    pub debt_id: Uuid,
    pub amount: Decimal,
    pub payment_date: NaiveDate,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDebtPaymentRequest {
    pub amount: Decimal,
    pub payment_date: NaiveDate,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DebtLimit {
    pub id: Uuid,
    pub max_total_debt: Decimal,
    pub max_single_debt: Option<Decimal>,
    pub effective_from: NaiveDate,
    pub effective_until: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDebtLimitRequest {
    pub max_total_debt: Decimal,
    pub max_single_debt: Option<Decimal>,
    pub effective_from: NaiveDate,
    pub effective_until: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtSummary {
    pub total_owed: Decimal,
    pub total_receivable: Decimal,
    pub net_position: Decimal,
    pub current_balance: Decimal,
    pub projected_balance_after_settlements: Decimal,
    pub debt_limit: Option<Decimal>,
    pub available_debt_capacity: Option<Decimal>,
    pub upcoming_deadlines: Vec<DebtDeadline>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtDeadline {
    pub debt_id: Uuid,
    pub counterparty: String,
    pub amount: Decimal,
    pub remaining: Decimal,
    pub debt_type: String,
    pub deadline: NaiveDate,
    pub days_remaining: i64,
}
