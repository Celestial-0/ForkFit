use axum::{
    body::Body,
    http::{HeaderValue, Request},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

pub const REQUEST_ID_HEADER: &str = "x-request-id";

pub async fn request_id_middleware(mut request: Request<Body>, next: Next) -> Response {
    let request_id = request
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or_else(Uuid::new_v4);

    let request_id_str = request_id.to_string();
    
    // Inject request ID into request extensions
    request.extensions_mut().insert(request_id);

    // Run next middleware/handler
    let mut response = next.run(request).await;

    // Add to response headers
    if let Ok(header_val) = HeaderValue::from_str(&request_id_str) {
        response.headers_mut().insert(REQUEST_ID_HEADER, header_val);
    }

    response
}
