use axum::{extract::{State, Path, Query}, Json, http::StatusCode};
use uuid::Uuid;
use serde::Deserialize;
use crate::app_state::AppState;
use crate::errors::AppError;
use crate::api::responses::*;
use crate::models::debt::*;
use crate::services::debt_service;

#[derive(Debug, Deserialize)]
pub struct DebtQuery { pub debt_type: Option<String>, pub status: Option<String> }

pub async fn list_debts(State(state): State<AppState>, Query(q): Query<DebtQuery>) -> Result<Json<ApiResponse<Vec<Debt>>>, AppError> {
    Ok(Json(ApiResponse::success(debt_service::list_debts(&state.db, q.debt_type, q.status).await?)))
}
pub async fn get_debt(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<ApiResponse<Debt>>, AppError> {
    Ok(Json(ApiResponse::success(debt_service::get_debt(&state.db, id).await?)))
}
pub async fn create_debt(State(state): State<AppState>, Json(req): Json<CreateDebtRequest>) -> Result<(StatusCode, Json<ApiResponse<Debt>>), AppError> {
    Ok(created(debt_service::create_debt(&state.db, req).await?))
}
pub async fn update_debt(State(state): State<AppState>, Path(id): Path<Uuid>, Json(req): Json<UpdateDebtRequest>) -> Result<Json<ApiResponse<Debt>>, AppError> {
    Ok(Json(ApiResponse::success(debt_service::update_debt(&state.db, id, req).await?)))
}
pub async fn delete_debt(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<StatusCode, AppError> {
    debt_service::delete_debt(&state.db, id).await?; Ok(StatusCode::NO_CONTENT)
}
pub async fn record_payment(State(state): State<AppState>, Path(id): Path<Uuid>, Json(req): Json<CreateDebtPaymentRequest>) -> Result<(StatusCode, Json<ApiResponse<DebtPayment>>), AppError> {
    Ok(created(debt_service::record_payment(&state.db, id, req).await?))
}
pub async fn get_summary(State(state): State<AppState>) -> Result<Json<ApiResponse<DebtSummary>>, AppError> {
    Ok(Json(ApiResponse::success(debt_service::get_summary(&state.db).await?)))
}
pub async fn get_limit(State(state): State<AppState>) -> Result<Json<ApiResponse<Option<DebtLimit>>>, AppError> {
    Ok(Json(ApiResponse::success(debt_service::get_limit(&state.db).await?)))
}
pub async fn set_limit(State(state): State<AppState>, Json(req): Json<CreateDebtLimitRequest>) -> Result<(StatusCode, Json<ApiResponse<DebtLimit>>), AppError> {
    Ok(created(debt_service::set_limit(&state.db, req).await?))
}
