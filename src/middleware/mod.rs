use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use std::time::Instant;
use tracing::{error, event, info, Level};

use axum::body::to_bytes;

pub async fn logging_middleware(
    request: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let version = request.version();
    let headers = request.headers().clone();

    let request_id = uuid::Uuid::new_v4();
    println!("logging_middleware");

    // Log request
    info!(
        request_id = %request_id,
        method = %method,
        uri = %uri,
        version = ?version,
        "Incoming request"
    );

    event!(Level::INFO, "something happened");

    // Log headers (be careful with sensitive headers)
    for (name, value) in headers.iter() {
        if let Ok(value_str) = value.to_str() {
            // Skip logging sensitive headers
            if !is_sensitive_header(name) {
                info!(
                    request_id = %request_id,
                    header_name = %name,
                    header_value = value_str,
                    "Request header"
                );
            }
        }
    }

    let start = Instant::now();

    // Clone request body for logging (optional - can be heavy)
    let (parts, body) = request.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.map_err(|e| {
        error!("Failed to read request body: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to read request body".to_string(),
        )
    })?;

    let body_str = String::from_utf8_lossy(&bytes);
    if !body_str.trim().is_empty() {
        info!(
            request_id = %request_id,
            body = %body_str,
            "Request body"
        );
    }

    let request = Request::from_parts(parts, Body::from(bytes));
    let response = next.run(request).await;

    let latency = start.elapsed();
    let status = response.status();

    // Log response
    info!(
        request_id = %request_id,
        status = %status,
        latency = ?latency,
        "Response generated"
    );

    // Log response headers
    for (name, value) in response.headers().iter() {
        if let Ok(value_str) = value.to_str() {
            if !is_sensitive_header(name) {
                info!(
                    request_id = %request_id,
                    header_name = %name,
                    header_value = value_str,
                    "Response header"
                );
            }
        }
    }

    Ok(response)
}

fn is_sensitive_header(name: &axum::http::HeaderName) -> bool {
    let sensitive_headers = [
        "authorization",
        "cookie",
        "proxy-authorization",
        "x-api-key",
    ];

    sensitive_headers.contains(&name.as_str().to_lowercase().as_str())
}
