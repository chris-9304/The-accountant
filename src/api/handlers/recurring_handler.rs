use axum::{extract::{State, Path}, Json};
use uuid::Uuid;
use crate::app_state::AppState;
use crate::errors::AppError;
use crate::api::responses::*;
use crate::models::recurring_expense::*;
use crate::repositories::recurring_repo;
use crate::services::recurring_detector;

pub async fn list_recurring(State(state): State<AppState>) -> Result<Json<ApiResponse<Vec<RecurringExpense>>>, AppError> {
    Ok(Json(ApiResponse::success(recurring_repo::find_all(&state.db, false).await?)))
}
pub async fn update_recurring(State(state): State<AppState>, Path(id): Path<Uuid>, Json(req): Json<UpdateRecurringRequest>) -> Result<Json<ApiResponse<RecurringExpense>>, AppError> {
    let r = recurring_repo::update(&state.db, id, &req).await?.ok_or_else(|| AppError::NotFound("Not found".into()))?;
    Ok(Json(ApiResponse::success(r)))
}
pub async fn trigger_detection(State(state): State<AppState>) -> Result<Json<ApiResponse<Vec<RecurringExpense>>>, AppError> {
    Ok(Json(ApiResponse::success(recurring_detector::detect_recurring(&state.db).await?)))
}
