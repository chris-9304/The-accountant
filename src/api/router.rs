use axum::{
    Router,
    routing::{get, post, put},
    middleware,
};
use tower_http::cors::CorsLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::services::ServeDir;
use crate::app_state::AppState;
use crate::api::middleware::request_logger;
use crate::api::handlers::*;

pub fn create_router(state: AppState) -> Router {
    let api = Router::new()
        // Dashboard
        .route("/dashboard", get(dashboard_handler::get_dashboard))
        // Accounts
        .route("/accounts", get(account_handler::list_accounts).post(account_handler::create_account))
        .route("/accounts/{id}", get(account_handler::get_account).put(account_handler::update_account).delete(account_handler::delete_account))
        // Categories
        .route("/categories", get(category_handler::list_categories).post(category_handler::create_category))
        .route("/categories/{id}", put(category_handler::update_category).delete(category_handler::delete_category))
        // Tags
        .route("/tags", get(tag_handler::list_tags).post(tag_handler::create_tag))
        .route("/tags/{id}", put(tag_handler::update_tag).delete(tag_handler::delete_tag))
        // Transactions
        .route("/transactions", get(transaction_handler::list_transactions).post(transaction_handler::create_transaction))
        .route("/transactions/{id}", get(transaction_handler::get_transaction).put(transaction_handler::update_transaction).delete(transaction_handler::delete_transaction))
        .route("/transactions/{txn_id}/tags/{tag_id}", post(transaction_handler::add_tag).delete(transaction_handler::remove_tag))
        // Statements
        .route("/statements/upload", post(statement_handler::upload_statement))
        .route("/statements", get(statement_handler::list_statements))
        .route("/statements/{id}", get(statement_handler::get_statement).delete(statement_handler::delete_statement))
        // Debts - specific routes first
        .route("/debts/summary", get(debt_handler::get_summary))
        .route("/debts/limits", get(debt_handler::get_limit).post(debt_handler::set_limit))
        .route("/debts", get(debt_handler::list_debts).post(debt_handler::create_debt))
        .route("/debts/{id}", get(debt_handler::get_debt).put(debt_handler::update_debt).delete(debt_handler::delete_debt))
        .route("/debts/{id}/payments", post(debt_handler::record_payment))
        // Budgets - specific routes first
        .route("/budgets/current", get(budget_handler::get_current_budget))
        .route("/budgets", get(budget_handler::list_budgets).post(budget_handler::create_budget))
        .route("/budgets/{id}", get(budget_handler::get_budget).put(budget_handler::update_budget).delete(budget_handler::delete_budget))
        // Purchases
        .route("/purchases", get(purchase_handler::list_purchases).post(purchase_handler::create_purchase))
        .route("/purchases/{id}", get(purchase_handler::get_purchase).put(purchase_handler::update_purchase).delete(purchase_handler::delete_purchase))
        .route("/purchases/{id}/advice", get(purchase_handler::get_advice))
        // Recurring
        .route("/recurring/detect", post(recurring_handler::trigger_detection))
        .route("/recurring", get(recurring_handler::list_recurring))
        .route("/recurring/{id}", put(recurring_handler::update_recurring))
        // Alerts
        .route("/alerts/read-all", post(alert_handler::mark_all_read))
        .route("/alerts", get(alert_handler::list_alerts))
        .route("/alerts/{id}/read", put(alert_handler::mark_read))
        .route("/alerts/{id}/dismiss", put(alert_handler::mark_dismissed))
        // Rules
        .route("/rules", get(rule_handler::list_rules).post(rule_handler::create_rule))
        .route("/rules/{id}", put(rule_handler::update_rule).delete(rule_handler::delete_rule))
        // Reports
        .route("/reports/monthly/{month}", get(report_handler::get_monthly_report))
        .route("/reports/health-score", get(report_handler::get_health_score))
        .route("/reports/health-score/history", get(report_handler::get_health_history))
        .route("/reports/cash-flow", get(report_handler::get_cash_flow))
        .route("/reports/trends", get(report_handler::get_trends))
        .route("/reports/income-vs-expenses", get(report_handler::get_income_vs_expenses))
        .route("/reports/export/csv", get(report_handler::export_csv))
        .route("/reports/export/report/{month}", get(report_handler::export_report));

    Router::new()
        .nest("/api", api)
        .fallback_service(ServeDir::new("static"))
        .layer(middleware::from_fn(request_logger))
        .layer(CorsLayer::permissive())
        .layer(RequestBodyLimitLayer::new(50 * 1024 * 1024)) // 50MB
        .with_state(state)
}
