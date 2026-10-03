use axum::{extract::{State, Path}, Json, http::StatusCode};
use uuid::Uuid;
use crate::app_state::AppState;
use crate::errors::AppError;
use crate::api::responses::*;
use crate::models::account::*;
use crate::services::account_service;

pub async fn list_accounts(State(state): State<AppState>) -> Result<Json<ApiResponse<Vec<Account>>>, AppError> {
    let accs = account_service::list_accounts(&state.db).await?;
    Ok(Json(ApiResponse::success(accs)))
}

pub async fn get_account(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<ApiResponse<Account>>, AppError> {
    let acc = account_service::get_account(&state.db, id).await?;
    Ok(Json(ApiResponse::success(acc)))
}

pub async fn create_account(State(state): State<AppState>, Json(req): Json<CreateAccountRequest>) -> Result<(StatusCode, Json<ApiResponse<Account>>), AppError> {
    let acc = account_service::create_account(&state.db, req).await?;
    Ok(created(acc))
}

pub async fn update_account(State(state): State<AppState>, Path(id): Path<Uuid>, Json(req): Json<UpdateAccountRequest>) -> Result<Json<ApiResponse<Account>>, AppError> {
    let acc = account_service::update_account(&state.db, id, req).await?;
    Ok(Json(ApiResponse::success(acc)))
}

pub async fn delete_account(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<StatusCode, AppError> {
    account_service::delete_account(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
