use axum::{extract::{State, Path}, Json, http::StatusCode};
use uuid::Uuid;
use crate::app_state::AppState;
use crate::errors::AppError;
use crate::api::responses::*;
use crate::models::tag::*;
use crate::repositories::tag_repo;

pub async fn list_tags(State(state): State<AppState>) -> Result<Json<ApiResponse<Vec<Tag>>>, AppError> {
    Ok(Json(ApiResponse::success(tag_repo::find_all(&state.db).await?)))
}

pub async fn create_tag(State(state): State<AppState>, Json(req): Json<CreateTagRequest>) -> Result<(StatusCode, Json<ApiResponse<Tag>>), AppError> {
    Ok(created(tag_repo::create(&state.db, &req).await?))
}

pub async fn update_tag(State(state): State<AppState>, Path(id): Path<Uuid>, Json(req): Json<UpdateTagRequest>) -> Result<Json<ApiResponse<Tag>>, AppError> {
    let t = tag_repo::update(&state.db, id, &req).await?.ok_or_else(|| AppError::NotFound("Tag not found".into()))?;
    Ok(Json(ApiResponse::success(t)))
}

pub async fn delete_tag(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<StatusCode, AppError> {
    if !tag_repo::delete(&state.db, id).await? { return Err(AppError::NotFound("Tag not found".into())); }
    Ok(StatusCode::NO_CONTENT)
}
