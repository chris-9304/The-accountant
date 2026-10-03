use axum::{extract::{State, Path}, Json, http::StatusCode};
use uuid::Uuid;
use crate::app_state::AppState;
use crate::errors::AppError;
use crate::api::responses::*;
use crate::models::budget::*;
use crate::services::budget_service;

pub async fn list_budgets(State(state): State<AppState>) -> Result<Json<ApiResponse<Vec<Budget>>>, AppError> {
    Ok(Json(ApiResponse::success(budget_service::list_budgets(&state.db).await?)))
}
pub async fn get_budget(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<ApiResponse<Budget>>, AppError> {
    Ok(Json(ApiResponse::success(budget_service::get_budget(&state.db, id).await?)))
}
pub async fn get_current_budget(State(state): State<AppState>) -> Result<Json<ApiResponse<Option<BudgetProgress>>>, AppError> {
    Ok(Json(ApiResponse::success(budget_service::get_current_budget(&state.db).await?)))
}
pub async fn create_budget(State(state): State<AppState>, Json(req): Json<CreateBudgetRequest>) -> Result<(StatusCode, Json<ApiResponse<Budget>>), AppError> {
    Ok(created(budget_service::create_budget(&state.db, req).await?))
}
pub async fn update_budget(State(state): State<AppState>, Path(id): Path<Uuid>, Json(req): Json<UpdateBudgetRequest>) -> Result<Json<ApiResponse<Budget>>, AppError> {
    Ok(Json(ApiResponse::success(budget_service::update_budget(&state.db, id, req).await?)))
}
pub async fn delete_budget(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<StatusCode, AppError> {
    budget_service::delete_budget(&state.db, id).await?; Ok(StatusCode::NO_CONTENT)
}
