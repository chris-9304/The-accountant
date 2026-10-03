use axum::{extract::State, Json};
use crate::app_state::AppState;
use crate::errors::AppError;
use crate::api::responses::*;
use crate::models::dashboard::DashboardResponse;
use crate::services::dashboard_service;

pub async fn get_dashboard(State(state): State<AppState>) -> Result<Json<ApiResponse<DashboardResponse>>, AppError> {
    Ok(Json(ApiResponse::success(dashboard_service::get_dashboard(&state.db).await?)))
}
