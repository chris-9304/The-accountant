use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Transaction {
    pub id: Uuid,
    pub account_id: Uuid,
    pub category_id: Option<Uuid>,
    pub statement_id: Option<Uuid>,
    pub amount: Decimal,
    pub transaction_type: String,
    pub description: Option<String>,
    pub merchant_name: Option<String>,
    pub reference_number: Option<String>,
    pub transaction_date: NaiveDate,
    pub source: String,
    pub balance_after: Option<Decimal>,
    pub notes: Option<String>,
    pub is_recurring: bool,
    pub recurring_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTransactionRequest {
    pub account_id: Uuid,
    pub category_id: Option<Uuid>,
    pub amount: Decimal,
    pub transaction_type: String,
    pub description: Option<String>,
    pub merchant_name: Option<String>,
    pub reference_number: Option<String>,
    pub transaction_date: NaiveDate,
    pub notes: Option<String>,
    pub tag_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTransactionRequest {
    pub category_id: Option<Uuid>,
    pub description: Option<String>,
    pub merchant_name: Option<String>,
    pub notes: Option<String>,
    pub tag_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TransactionFilters {
    pub account_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
    pub amount_min: Option<Decimal>,
    pub amount_max: Option<Decimal>,
    pub transaction_type: Option<String>,
    pub source: Option<String>,
    pub tag: Option<String>,
    pub search: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}
