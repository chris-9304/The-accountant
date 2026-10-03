use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Statement {
    pub id: Uuid,
    pub account_id: Uuid,
    pub file_name: String,
    pub file_path: String,
    pub file_format: String,
    pub file_hash: String,
    pub bank_format: Option<String>,
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
    pub total_transactions: Option<i32>,
    pub total_credits: Option<Decimal>,
    pub total_debits: Option<Decimal>,
    pub status: String,
    pub error_message: Option<String>,
    pub parsed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatementUploadResponse {
    pub statement_id: Uuid,
    pub status: String,
    pub total_transactions: i32,
    pub total_credits: Decimal,
    pub total_debits: Decimal,
    pub message: String,
}
