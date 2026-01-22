use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;

/// Middleware to log HTTP access logs (similar to nginx access logs)
/// Logs: timestamp, method, URI, status, duration, user-agent
pub async fn access_logger(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("-")
        .to_string();

    // Try to extract client IP from various sources:
    // 1. X-Forwarded-For header (if behind proxy)
    // 2. X-Real-IP header (if behind proxy)
    // 3. Connection info extension (direct connection)
    let client_ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim())
        .or_else(|| req.headers().get("x-real-ip").and_then(|v| v.to_str().ok()))
        .map(|s| s.to_string())
        .or_else(|| {
            // Try to get from ConnectInfo extension (requires IntoMakeServiceWithConnectInfo)
            req.extensions()
                .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
                .map(|ci| ci.0.ip().to_string())
        })
        .unwrap_or_else(|| "unknown".to_string());

    let start = Instant::now();
    let response = next.run(req).await;
    let duration = start.elapsed();

    let status = response.status();

    // Log in a structured format similar to nginx combined log format
    tracing::info!(
        target: "access_log",
        client_ip = %client_ip,
        method = %method,
        uri = %uri,
        status = %status.as_u16(),
        duration_ms = %duration.as_millis(),
        user_agent = %user_agent,
        "HTTP Request"
    );

    response
}

/// Middleware to log errors from request handlers
pub async fn error_logger(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();

    let response = next.run(req).await;
    let status = response.status();

    // Log errors (4xx and 5xx status codes)
    if status.is_client_error() || status.is_server_error() {
        tracing::error!(
            target: "error_log",
            method = %method,
            uri = %uri,
            status = %status.as_u16(),
            "HTTP Error"
        );
    }

    response
}
