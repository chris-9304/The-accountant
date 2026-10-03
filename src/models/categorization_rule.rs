use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CategorizationRule {
    pub id: Uuid,
    pub pattern: String,
    pub category_id: Uuid,
    pub priority: i32,
    pub match_type: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRuleRequest {
    pub pattern: String,
    pub category_id: Uuid,
    pub priority: Option<i32>,
    pub match_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRuleRequest {
    pub pattern: Option<String>,
    pub category_id: Option<Uuid>,
    pub priority: Option<i32>,
    pub match_type: Option<String>,
    pub is_active: Option<bool>,
}
