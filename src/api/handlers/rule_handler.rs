use axum::{extract::{State, Path}, Json, http::StatusCode};
use uuid::Uuid;
use crate::app_state::AppState;
use crate::errors::AppError;
use crate::api::responses::*;
use crate::models::categorization_rule::*;
use crate::repositories::rule_repo;

pub async fn list_rules(State(state): State<AppState>) -> Result<Json<ApiResponse<Vec<CategorizationRule>>>, AppError> {
    Ok(Json(ApiResponse::success(rule_repo::find_all(&state.db).await?)))
}
pub async fn create_rule(State(state): State<AppState>, Json(req): Json<CreateRuleRequest>) -> Result<(StatusCode, Json<ApiResponse<CategorizationRule>>), AppError> {
    Ok(created(rule_repo::create(&state.db, &req).await?))
}
pub async fn update_rule(State(state): State<AppState>, Path(id): Path<Uuid>, Json(req): Json<UpdateRuleRequest>) -> Result<Json<ApiResponse<CategorizationRule>>, AppError> {
    let r = rule_repo::update(&state.db, id, &req).await?.ok_or_else(|| AppError::NotFound("Rule not found".into()))?;
    Ok(Json(ApiResponse::success(r)))
}
pub async fn delete_rule(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<StatusCode, AppError> {
    if !rule_repo::delete(&state.db, id).await? { return Err(AppError::NotFound("Rule not found".into())); }
    Ok(StatusCode::NO_CONTENT)
}
