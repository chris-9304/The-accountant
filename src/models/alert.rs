use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Alert {
    pub id: Uuid,
    pub alert_type: String,
    pub title: String,
    pub message: String,
    pub severity: String,
    pub is_read: bool,
    pub is_dismissed: bool,
    pub reference_type: Option<String>,
    pub reference_id: Option<Uuid>,
    pub triggered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlertFilters {
    pub severity: Option<String>,
    pub is_read: Option<bool>,
    pub alert_type: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}
