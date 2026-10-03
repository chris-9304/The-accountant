use axum::{extract::{State, Path, Query}, Json};
use uuid::Uuid;
use crate::app_state::AppState;
use crate::errors::AppError;
use crate::api::responses::*;
use crate::models::alert::*;
use crate::services::alert_service;

pub async fn list_alerts(State(state): State<AppState>, Query(f): Query<AlertFilters>) -> Result<Json<ApiResponse<Vec<Alert>>>, AppError> {
    let page = f.page.unwrap_or(1);
    let per_page = f.per_page.unwrap_or(50);
    let (alerts, total) = alert_service::list_alerts(&state.db, f).await?;
    Ok(Json(ApiResponse::success_with_meta(alerts, PaginationMeta { page, per_page, total })))
}
pub async fn mark_read(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<ApiResponse<String>>, AppError> {
    alert_service::mark_read(&state.db, id).await?; Ok(Json(ApiResponse::success("Marked read".into())))
}
pub async fn mark_dismissed(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<ApiResponse<String>>, AppError> {
    alert_service::mark_dismissed(&state.db, id).await?; Ok(Json(ApiResponse::success("Dismissed".into())))
}
pub async fn mark_all_read(State(state): State<AppState>) -> Result<Json<ApiResponse<String>>, AppError> {
    alert_service::mark_all_read(&state.db).await?; Ok(Json(ApiResponse::success("All marked read".into())))
}
