use axum::{extract::{State, Path, Query}, Json, http::StatusCode, response::IntoResponse};
use chrono::NaiveDate;
use uuid::Uuid;
use serde::Deserialize;
use crate::app_state::AppState;
use crate::errors::AppError;
use crate::api::responses::*;
use crate::models::snapshot::MonthlySnapshot;
use crate::models::health_score::HealthScore;
use crate::models::cash_flow::CashFlowForecast;
use crate::services::{snapshot_service, health_score as hs_service, cash_flow_forecast, export_service};
use crate::repositories::{snapshot_repo, health_score_repo, transaction_repo};

#[derive(Debug, Deserialize)]
pub struct DateRangeQuery { pub date_from: Option<NaiveDate>, pub date_to: Option<NaiveDate>, pub account_id: Option<Uuid>, pub days: Option<u32> }

pub async fn get_monthly_report(State(state): State<AppState>, Path(month_str): Path<String>) -> Result<Json<ApiResponse<MonthlySnapshot>>, AppError> {
    let month = NaiveDate::parse_from_str(&format!("{}-01", month_str), "%Y-%m-%d").map_err(|_| AppError::Validation("Format: YYYY-MM".into()))?;
    if let Some(s) = snapshot_repo::find_by_month(&state.db, month).await? { return Ok(Json(ApiResponse::success(s))); }
    let s = snapshot_service::generate_snapshot(&state.db, month).await?;
    Ok(Json(ApiResponse::success(s)))
}

pub async fn get_health_score(State(state): State<AppState>) -> Result<Json<ApiResponse<HealthScore>>, AppError> {
    let s = hs_service::calculate_health_score(&state.db).await?;
    Ok(Json(ApiResponse::success(s)))
}

pub async fn get_health_history(State(state): State<AppState>) -> Result<Json<ApiResponse<Vec<HealthScore>>>, AppError> {
    Ok(Json(ApiResponse::success(health_score_repo::find_history(&state.db, 12).await?)))
}

pub async fn get_cash_flow(State(state): State<AppState>, Query(q): Query<DateRangeQuery>) -> Result<Json<ApiResponse<Vec<CashFlowForecast>>>, AppError> {
    let days = q.days.unwrap_or(90);
    let forecasts = cash_flow_forecast::generate_forecast(&state.db, days).await?;
    Ok(Json(ApiResponse::success(forecasts)))
}

pub async fn get_trends(State(state): State<AppState>, Query(q): Query<DateRangeQuery>) -> Result<Json<ApiResponse<Vec<(String, rust_decimal::Decimal)>>>, AppError> {
    let from = q.date_from.unwrap_or_else(|| chrono::Utc::now().date_naive() - chrono::Duration::days(90));
    let to = q.date_to.unwrap_or_else(|| chrono::Utc::now().date_naive());
    let data = transaction_repo::get_spending_by_category(&state.db, from, to).await?;
    Ok(Json(ApiResponse::success(data)))
}

pub async fn get_income_vs_expenses(State(state): State<AppState>, Query(q): Query<DateRangeQuery>) -> Result<impl IntoResponse, AppError> {
    let from = q.date_from.unwrap_or_else(|| chrono::Utc::now().date_naive() - chrono::Duration::days(30));
    let to = q.date_to.unwrap_or_else(|| chrono::Utc::now().date_naive());
    let income = transaction_repo::get_total_by_type(&state.db, None, from, to, "credit").await?;
    let expenses = transaction_repo::get_total_by_type(&state.db, None, from, to, "debit").await?;
    Ok(Json(ApiResponse::success(serde_json::json!({"income": income.to_string(), "expenses": expenses.to_string(), "net": (income - expenses).to_string()}))))
}

pub async fn export_csv(State(state): State<AppState>, Query(q): Query<DateRangeQuery>) -> Result<impl IntoResponse, AppError> {
    let from = q.date_from.unwrap_or_else(|| chrono::Utc::now().date_naive() - chrono::Duration::days(30));
    let to = q.date_to.unwrap_or_else(|| chrono::Utc::now().date_naive());
    let csv = export_service::export_csv(&state.db, from, to, q.account_id).await?;
    Ok((StatusCode::OK, [("content-type", "text/csv"), ("content-disposition", "attachment; filename=transactions.csv")], csv))
}

pub async fn export_report(State(state): State<AppState>, Path(month_str): Path<String>) -> Result<impl IntoResponse, AppError> {
    let month = NaiveDate::parse_from_str(&format!("{}-01", month_str), "%Y-%m-%d").map_err(|_| AppError::Validation("Format: YYYY-MM".into()))?;
    let report = export_service::export_monthly_report(&state.db, month).await?;
    Ok((StatusCode::OK, [("content-type", "text/plain")], report))
}
