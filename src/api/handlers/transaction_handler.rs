use axum::{extract::{State, Path, Query}, Json, http::StatusCode};
use uuid::Uuid;
use crate::app_state::AppState;
use crate::errors::AppError;
use crate::api::responses::*;
use crate::models::transaction::*;
use crate::services::transaction_service;

pub async fn list_transactions(State(state): State<AppState>, Query(filters): Query<TransactionFilters>) -> Result<Json<ApiResponse<Vec<Transaction>>>, AppError> {
    let page = filters.page.unwrap_or(1);
    let per_page = filters.per_page.unwrap_or(50);
    let (txns, total) = transaction_service::list_transactions(&state.db, filters).await?;
    Ok(Json(ApiResponse::success_with_meta(txns, PaginationMeta { page, per_page, total })))
}

pub async fn get_transaction(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<ApiResponse<Transaction>>, AppError> {
    Ok(Json(ApiResponse::success(transaction_service::get_transaction(&state.db, id).await?)))
}

pub async fn create_transaction(State(state): State<AppState>, Json(req): Json<CreateTransactionRequest>) -> Result<(StatusCode, Json<ApiResponse<Transaction>>), AppError> {
    Ok(created(transaction_service::create_transaction(&state.db, req).await?))
}

pub async fn update_transaction(State(state): State<AppState>, Path(id): Path<Uuid>, Json(req): Json<UpdateTransactionRequest>) -> Result<Json<ApiResponse<Transaction>>, AppError> {
    Ok(Json(ApiResponse::success(transaction_service::update_transaction(&state.db, id, req).await?)))
}

pub async fn delete_transaction(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<StatusCode, AppError> {
    transaction_service::delete_transaction(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn add_tag(State(state): State<AppState>, Path((txn_id, tag_id)): Path<(Uuid, Uuid)>) -> Result<Json<ApiResponse<String>>, AppError> {
    transaction_service::add_tag(&state.db, txn_id, tag_id).await?;
    Ok(Json(ApiResponse::success("Tag added".into())))
}

pub async fn remove_tag(State(state): State<AppState>, Path((txn_id, tag_id)): Path<(Uuid, Uuid)>) -> Result<StatusCode, AppError> {
    transaction_service::remove_tag(&state.db, txn_id, tag_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
