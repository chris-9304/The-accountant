use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub meta: Option<PaginationMeta>,
}

#[derive(Debug, Serialize, Clone)]
pub struct PaginationMeta {
    pub page: u32,
    pub per_page: u32,
    pub total: i64,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self { success: true, data: Some(data), error: None, meta: None }
    }
    pub fn success_with_meta(data: T, meta: PaginationMeta) -> Self {
        Self { success: true, data: Some(data), error: None, meta: Some(meta) }
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> axum::response::Response {
        let status = if self.success { StatusCode::OK } else { StatusCode::BAD_REQUEST };
        (status, Json(self)).into_response()
    }
}

pub fn created<T: Serialize>(data: T) -> (StatusCode, Json<ApiResponse<T>>) {
    (StatusCode::CREATED, Json(ApiResponse::success(data)))
}
