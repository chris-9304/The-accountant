use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;

pub async fn request_logger(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let start = Instant::now();
    let response = next.run(req).await;
    tracing::info!(method = %method, uri = %uri, status = %response.status(), duration_ms = %start.elapsed().as_millis(), "request completed");
    response
}
