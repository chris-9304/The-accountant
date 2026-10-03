use axum::{extract::{State, Path, Query}, Json, http::StatusCode};
use uuid::Uuid;
use serde::Deserialize;
use crate::app_state::AppState;
use crate::errors::AppError;
use crate::api::responses::*;
use crate::models::planned_purchase::*;
use crate::services::purchase_advisor;

#[derive(Debug, Deserialize)]
pub struct PurchaseQuery { pub status: Option<String> }

pub async fn list_purchases(State(state): State<AppState>, Query(q): Query<PurchaseQuery>) -> Result<Json<ApiResponse<Vec<PlannedPurchase>>>, AppError> {
    Ok(Json(ApiResponse::success(purchase_advisor::list_purchases(&state.db, q.status).await?)))
}
pub async fn get_purchase(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<ApiResponse<PlannedPurchase>>, AppError> {
    Ok(Json(ApiResponse::success(purchase_advisor::get_purchase(&state.db, id).await?)))
}
pub async fn create_purchase(State(state): State<AppState>, Json(req): Json<CreatePurchaseRequest>) -> Result<(StatusCode, Json<ApiResponse<PlannedPurchase>>), AppError> {
    Ok(created(purchase_advisor::create_purchase(&state.db, req).await?))
}
pub async fn update_purchase(State(state): State<AppState>, Path(id): Path<Uuid>, Json(req): Json<UpdatePurchaseRequest>) -> Result<Json<ApiResponse<PlannedPurchase>>, AppError> {
    Ok(Json(ApiResponse::success(purchase_advisor::update_purchase(&state.db, id, req).await?)))
}
pub async fn delete_purchase(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<StatusCode, AppError> {
    purchase_advisor::delete_purchase(&state.db, id).await?; Ok(StatusCode::NO_CONTENT)
}
pub async fn get_advice(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<ApiResponse<PurchaseAdvice>>, AppError> {
    Ok(Json(ApiResponse::success(purchase_advisor::get_advice(&state.db, id).await?)))
}
