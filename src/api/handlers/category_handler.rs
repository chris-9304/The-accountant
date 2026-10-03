use axum::{extract::{State, Path}, Json, http::StatusCode};
use uuid::Uuid;
use crate::app_state::AppState;
use crate::errors::AppError;
use crate::api::responses::*;
use crate::models::category::*;
use crate::repositories::category_repo;

pub async fn list_categories(State(state): State<AppState>) -> Result<Json<ApiResponse<Vec<Category>>>, AppError> {
    Ok(Json(ApiResponse::success(category_repo::find_all(&state.db).await?)))
}

pub async fn create_category(State(state): State<AppState>, Json(req): Json<CreateCategoryRequest>) -> Result<(StatusCode, Json<ApiResponse<Category>>), AppError> {
    Ok(created(category_repo::create(&state.db, &req).await?))
}

pub async fn update_category(State(state): State<AppState>, Path(id): Path<Uuid>, Json(req): Json<UpdateCategoryRequest>) -> Result<Json<ApiResponse<Category>>, AppError> {
    let c = category_repo::update(&state.db, id, &req).await?.ok_or_else(|| AppError::NotFound("Category not found".into()))?;
    Ok(Json(ApiResponse::success(c)))
}

pub async fn delete_category(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<StatusCode, AppError> {
    if !category_repo::delete(&state.db, id).await? { return Err(AppError::NotFound("Category not found".into())); }
    Ok(StatusCode::NO_CONTENT)
}
