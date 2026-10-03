use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::models::account::AccountSummary;
use crate::models::transaction::Transaction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardResponse {
    pub total_balance: Decimal,
    pub accounts: Vec<AccountSummary>,
    pub total_debt_owed: Decimal,
    pub total_debt_receivable: Decimal,
    pub net_debt_position: Decimal,
    pub projected_balance_after_debts: Decimal,
    pub budget_spent: Decimal,
    pub budget_limit: Decimal,
    pub budget_remaining: Decimal,
    pub savings_target: Decimal,
    pub savings_actual: Decimal,
    pub health_score: Option<i32>,
    pub spending_this_month: Decimal,
    pub spending_last_month: Decimal,
    pub spending_change_pct: Decimal,
    pub top_spending_categories: Vec<CategorySpending>,
    pub recent_transactions: Vec<Transaction>,
    pub unread_alert_count: i64,
    pub debt_limit: Option<Decimal>,
    pub current_total_debt: Decimal,
    pub available_debt_capacity: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySpending {
    pub category_name: String,
    pub amount: Decimal,
    pub percentage: Decimal,
}
