//! # OpenAPI Documentation
//!
//! Auto-generated OpenAPI specification and Swagger UI integration.
//!
//! ## Features
//!
//! - **OpenAPI 3.0**: Industry-standard API documentation
//! - **Swagger UI**: Interactive API explorer
//! - **RapiDoc**: Alternative documentation UI
//! - **ReDoc**: Beautiful API documentation
//! - **API Versioning**: Support for multiple API versions

use axum::Router;
use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

use crate::rest::{
    torrents::{TorrentResponse, TorrentSearchParams, UploadTorrentRequest, UpdateTorrentRequest},
    users::{UserResponse, UserStatisticsResponse, UpdateUserRequest},
    ErrorResponse, PaginatedResponse, PaginationMeta, PaginationParams,
};

/// OpenAPI documentation structure
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Unified Tracker Platform API",
        version = "1.0.0",
        description = "Comprehensive API for torrent tracking, user management, and community features",
        contact(
            name = "API Support",
            email = "api@tracker.example.com"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "http://localhost:8080", description = "Local development server"),
        (url = "https://api.tracker.example.com", description = "Production server")
    ),
    paths(
        crate::rest::api_version,
        crate::rest::torrents::list_torrents,
        crate::rest::torrents::get_torrent,
        crate::rest::torrents::upload_torrent,
        crate::rest::torrents::update_torrent,
        crate::rest::torrents::download_torrent,
        crate::rest::users::get_user,
        crate::rest::users::update_user,
        crate::rest::users::get_user_stats,
        crate::rest::users::get_user_torrents,
    ),
    components(
        schemas(
            TorrentResponse,
            TorrentSearchParams,
            UploadTorrentRequest,
            UpdateTorrentRequest,
            UserResponse,
            UserStatisticsResponse,
            UpdateUserRequest,
            ErrorResponse,
            PaginationParams,
            PaginationMeta,
            crate::rest::ApiVersion,
        )
    ),
    tags(
        (name = "meta", description = "API metadata and version information"),
        (name = "torrents", description = "Torrent operations"),
        (name = "users", description = "User management"),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

/// Security scheme configuration
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .description(Some("JWT token for authentication"))
                        .build(),
                ),
            );
        }
    }
}

/// Configure OpenAPI documentation routes
pub fn configure_routes<S>(app: Router<S>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let openapi = ApiDoc::openapi();

    app
        // Swagger UI
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi.clone()))
        // RapiDoc
        .merge(
            utoipa_rapidoc::RapiDoc::new("/api-docs/openapi.json")
                .path("/rapidoc")
                .custom_html(include_str!("../static/rapidoc.html").unwrap_or(RAPIDOC_HTML)),
        )
        // ReDoc
        .merge(
            utoipa_redoc::Redoc::with_url("/redoc", openapi.clone())
        )
}

/// Default RapiDoc HTML template
const RAPIDOC_HTML: &str = r#"<!doctype html>
<html>
<head>
    <meta charset="utf-8">
    <script type="module" src="https://unpkg.com/rapidoc/dist/rapidoc-min.js"></script>
</head>
<body>
    <rapi-doc
        spec-url="{spec_url}"
        theme="dark"
        bg-color="#1a1a2e"
        text-color="#eaeaea"
        header-color="#16213e"
        primary-color="#0f3460"
        nav-bg-color="#16213e"
        nav-text-color="#eaeaea"
        nav-hover-bg-color="#0f3460"
        nav-hover-text-color="#ffffff"
        render-style="view"
        show-header="false"
        allow-search="true"
        allow-try="true"
        allow-server-selection="true"
    ></rapi-doc>
</body>
</html>"#;

/// OpenAPI specification metadata
pub struct OpenApiMetadata {
    pub version: String,
    pub title: String,
    pub description: String,
}

impl Default for OpenApiMetadata {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            title: "Unified Tracker Platform API".to_string(),
            description: "Comprehensive API for torrent tracking, user management, and community features".to_string(),
        }
    }
}

/// Get OpenAPI specification as JSON
pub fn get_openapi_spec() -> String {
    serde_json::to_string_pretty(&ApiDoc::openapi()).expect("Failed to serialize OpenAPI spec")
}

/// Get OpenAPI specification as YAML
pub fn get_openapi_spec_yaml() -> String {
    serde_yaml::to_string(&ApiDoc::openapi()).expect("Failed to serialize OpenAPI spec to YAML")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openapi_spec_generation() {
        let spec = ApiDoc::openapi();
        assert_eq!(spec.info.title, "Unified Tracker Platform API");
        assert_eq!(spec.info.version, "1.0.0");
    }

    #[test]
    fn test_openapi_spec_has_paths() {
        let spec = ApiDoc::openapi();
        assert!(spec.paths.paths.len() > 0);
    }

    #[test]
    fn test_openapi_spec_has_components() {
        let spec = ApiDoc::openapi();
        assert!(spec.components.is_some());
        let components = spec.components.unwrap();
        assert!(components.schemas.len() > 0);
    }

    #[test]
    fn test_openapi_spec_json_serialization() {
        let json = get_openapi_spec();
        assert!(json.contains("Unified Tracker Platform API"));
    }

    #[test]
    fn test_metadata_default() {
        let metadata = OpenApiMetadata::default();
        assert_eq!(metadata.title, "Unified Tracker Platform API");
        assert!(!metadata.version.is_empty());
    }
}
