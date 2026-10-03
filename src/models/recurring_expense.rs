use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RecurringExpense {
    pub id: Uuid,
    pub merchant_name: String,
    pub category_id: Option<Uuid>,
    pub average_amount: Decimal,
    pub min_amount: Option<Decimal>,
    pub max_amount: Option<Decimal>,
    pub frequency: String,
    pub last_seen: Option<NaiveDate>,
    pub next_expected: Option<NaiveDate>,
    pub account_id: Option<Uuid>,
    pub occurrence_count: i32,
    pub is_active: bool,
    pub confidence: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRecurringRequest {
    pub category_id: Option<Uuid>,
    pub is_active: Option<bool>,
    pub frequency: Option<String>,
}
