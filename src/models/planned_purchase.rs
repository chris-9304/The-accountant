use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PlannedPurchase {
    pub id: Uuid,
    pub item_name: String,
    pub estimated_cost: Decimal,
    pub urgency: String,
    pub target_date: Option<NaiveDate>,
    pub recommended_purchase_date: Option<NaiveDate>,
    pub funding_strategy: Option<String>,
    pub funding_details: Option<serde_json::Value>,
    pub category_id: Option<Uuid>,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePurchaseRequest {
    pub item_name: String,
    pub estimated_cost: Decimal,
    pub urgency: Option<String>,
    pub target_date: Option<NaiveDate>,
    pub category_id: Option<Uuid>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePurchaseRequest {
    pub item_name: Option<String>,
    pub estimated_cost: Option<Decimal>,
    pub urgency: Option<String>,
    pub target_date: Option<NaiveDate>,
    pub status: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseAdvice {
    pub purchase_id: Uuid,
    pub item_name: String,
    pub estimated_cost: Decimal,
    pub recommended_date: Option<NaiveDate>,
    pub funding_strategy: String,
    pub funding_breakdown: Vec<FundingSource>,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundingSource {
    pub source: String,
    pub amount: Decimal,
    pub details: String,
}
