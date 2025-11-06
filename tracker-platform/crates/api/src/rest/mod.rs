//! # REST API Module
//!
//! RESTful API endpoints with OpenAPI documentation and versioning.
//!
//! ## Features
//!
//! - **Versioning**: All endpoints are versioned (e.g., /api/v1/)
//! - **OpenAPI**: Auto-generated documentation
//! - **Rate Limiting**: Per-endpoint and per-user limits
//! - **Authentication**: JWT-based authentication
//! - **Pagination**: Cursor-based and offset-based pagination
//! - **Filtering**: Query parameters for filtering and sorting

pub mod torrents;
pub mod users;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::instrument;

use crate::{ApiError, ApiState};

/// Configure REST API routes
pub fn configure_routes(app: Router<Arc<ApiState>>, state: Arc<ApiState>) -> Router<Arc<ApiState>> {
    app
        // API version info
        .route("/api/v1", get(api_version))
        // Torrent endpoints
        .nest("/api/v1/torrents", torrents::routes())
        // User endpoints
        .nest("/api/v1/users", users::routes())
}

/// API version information
#[derive(Debug, Serialize)]
pub struct ApiVersion {
    pub version: String,
    pub api_version: String,
    pub endpoints: Vec<String>,
}

/// Get API version information
#[utoipa::path(
    get,
    path = "/api/v1",
    tag = "meta",
    responses(
        (status = 200, description = "API version information", body = ApiVersion)
    )
)]
#[instrument]
async fn api_version() -> Json<ApiVersion> {
    Json(ApiVersion {
        version: env!("CARGO_PKG_VERSION").to_string(),
        api_version: "v1".to_string(),
        endpoints: vec![
            "/api/v1/torrents".to_string(),
            "/api/v1/users".to_string(),
        ],
    })
}

/// Standard pagination parameters
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct PaginationParams {
    /// Page number (1-indexed)
    #[serde(default = "default_page")]
    pub page: u32,
    /// Items per page (max 100)
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    20
}

impl PaginationParams {
    /// Get the offset for the current page
    pub fn offset(&self) -> i64 {
        ((self.page.saturating_sub(1)) * self.per_page) as i64
    }

    /// Get the limit (capped at 100)
    pub fn limit(&self) -> i64 {
        self.per_page.min(100) as i64
    }
}

/// Standard paginated response
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

/// Pagination metadata
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PaginationMeta {
    pub page: u32,
    pub per_page: u32,
    pub total: i64,
    pub total_pages: u32,
}

impl PaginationMeta {
    pub fn new(page: u32, per_page: u32, total: i64) -> Self {
        let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;
        Self {
            page,
            per_page,
            total,
            total_pages,
        }
    }
}

/// Standard error response
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ErrorResponse {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            details: None,
        }
    }

    pub fn with_details(error: impl Into<String>, details: serde_json::Value) -> Self {
        Self {
            error: error.into(),
            details: Some(details),
        }
    }
}

/// Extract user ID from JWT token in Authorization header
///
/// Returns None if not authenticated
pub async fn extract_user_id(
    headers: &axum::http::HeaderMap,
) -> Option<uuid::Uuid> {
    // TODO: Implement JWT token validation
    // For now, return None (unauthenticated)
    None
}

/// Extract user ID from JWT token, or return error if not authenticated
pub async fn require_auth(
    headers: &axum::http::HeaderMap,
) -> Result<uuid::Uuid, ApiError> {
    extract_user_id(headers)
        .await
        .ok_or_else(|| ApiError::AuthenticationError("Authentication required".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_params() {
        let params = PaginationParams {
            page: 2,
            per_page: 20,
        };
        assert_eq!(params.offset(), 20);
        assert_eq!(params.limit(), 20);
    }

    #[test]
    fn test_pagination_params_max_limit() {
        let params = PaginationParams {
            page: 1,
            per_page: 200, // Over max
        };
        assert_eq!(params.limit(), 100); // Should be capped
    }

    #[test]
    fn test_pagination_meta() {
        let meta = PaginationMeta::new(1, 20, 45);
        assert_eq!(meta.total_pages, 3);
    }
}
