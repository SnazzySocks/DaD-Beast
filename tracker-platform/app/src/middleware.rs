use axum::body::Body;
use axum::extract::{MatchedPath, Request};
use axum::http::header::{HeaderName, HeaderValue};
use axum::http::{Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use std::time::Instant;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;
use uuid::Uuid;

/// Request ID middleware - adds a unique ID to each request
pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    let request_id = Uuid::new_v4().to_string();

    // Add request ID to request extensions
    request.extensions_mut().insert(RequestId(request_id.clone()));

    // Add request ID to tracing span
    tracing::Span::current().record("request_id", &request_id);

    let mut response = next.run(request).await;

    // Add request ID to response headers
    response.headers_mut().insert(
        HeaderName::from_static("x-request-id"),
        HeaderValue::from_str(&request_id).unwrap(),
    );

    response
}

/// Request ID extractor
#[derive(Clone, Debug)]
pub struct RequestId(pub String);

/// Metrics middleware - tracks request metrics
pub async fn metrics_middleware(request: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let path = request
        .extensions()
        .get::<MatchedPath>()
        .map(|p| p.as_str().to_string())
        .unwrap_or_else(|| request.uri().path().to_string());

    let response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status();

    // Record metrics
    crate::telemetry::metrics::HTTP_REQUESTS_TOTAL
        .with_label_values(&[method.as_str(), &path, status.as_str()])
        .inc();

    crate::telemetry::metrics::HTTP_REQUEST_DURATION_SECONDS
        .with_label_values(&[method.as_str(), &path, status.as_str()])
        .observe(duration.as_secs_f64());

    response
}

/// Security headers middleware
pub async fn security_headers_middleware(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;

    let headers = response.headers_mut();

    // Prevent clickjacking
    headers.insert(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );

    // XSS protection
    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );

    // Disable browser features
    headers.insert(
        HeaderName::from_static("x-permitted-cross-domain-policies"),
        HeaderValue::from_static("none"),
    );

    // Referrer policy
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Content Security Policy (adjust based on your needs)
    headers.insert(
        HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static("default-src 'self'"),
    );

    response
}

/// Request logging middleware
pub async fn request_logging_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let version = request.version();

    tracing::info!(
        method = %method,
        uri = %uri,
        version = ?version,
        "Incoming request"
    );

    let response = next.run(request).await;

    tracing::info!(
        method = %method,
        uri = %uri,
        status = %response.status(),
        "Request completed"
    );

    response
}

/// Error handling middleware
pub async fn error_handling_middleware(request: Request, next: Next) -> Response {
    let response = next.run(request).await;

    if response.status().is_server_error() {
        tracing::error!(
            status = %response.status(),
            "Server error occurred"
        );
    } else if response.status().is_client_error() {
        tracing::warn!(
            status = %response.status(),
            "Client error occurred"
        );
    }

    response
}

/// Create CORS layer from configuration
pub fn create_cors_layer(config: &crate::config::CorsConfig) -> CorsLayer {
    let mut cors = CorsLayer::new();

    // Configure allowed origins
    if config.allowed_origins.contains(&"*".to_string()) {
        cors = cors.allow_origin(Any);
    } else {
        for origin in &config.allowed_origins {
            if let Ok(origin_value) = origin.parse::<HeaderValue>() {
                cors = cors.allow_origin(origin_value);
            }
        }
    }

    // Configure allowed methods
    let methods: Vec<Method> = config
        .allowed_methods
        .iter()
        .filter_map(|m| m.parse().ok())
        .collect();
    cors = cors.allow_methods(methods);

    // Configure allowed headers
    let headers: Vec<HeaderName> = config
        .allowed_headers
        .iter()
        .filter_map(|h| h.parse().ok())
        .collect();
    cors = cors.allow_headers(headers);

    // Configure max age
    cors = cors.max_age(std::time::Duration::from_secs(config.max_age_secs));

    cors
}

/// Create compression layer
pub fn create_compression_layer() -> CompressionLayer {
    CompressionLayer::new()
}

/// Create tracing layer for request/response logging
pub fn create_trace_layer() -> TraceLayer<
    tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
> {
    TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO))
}

/// Error response structure
#[derive(serde::Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub request_id: Option<String>,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        let status = StatusCode::INTERNAL_SERVER_ERROR;
        let body = serde_json::to_string(&self).unwrap_or_else(|_| {
            r#"{"error":"internal_server_error","message":"An error occurred"}"#.to_string()
        });

        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::CorsConfig;

    #[test]
    fn test_request_id() {
        let id = Uuid::new_v4().to_string();
        let request_id = RequestId(id.clone());
        assert_eq!(request_id.0, id);
    }

    #[test]
    fn test_cors_layer_creation() {
        let config = CorsConfig {
            allowed_origins: vec!["http://localhost:3000".to_string()],
            allowed_methods: vec!["GET".to_string(), "POST".to_string()],
            allowed_headers: vec!["Content-Type".to_string()],
            max_age_secs: 3600,
        };

        let _cors_layer = create_cors_layer(&config);
    }

    #[test]
    fn test_error_response_serialization() {
        let error = ErrorResponse {
            error: "test_error".to_string(),
            message: "Test error message".to_string(),
            request_id: Some("test-id".to_string()),
        };

        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("test_error"));
        assert!(json.contains("Test error message"));
        assert!(json.contains("test-id"));
    }
}
