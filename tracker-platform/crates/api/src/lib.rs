//! # API Crate
//!
//! Unified API layer providing both GraphQL and REST endpoints for the tracker platform.
//!
//! ## Features
//!
//! - **GraphQL API**: Full-featured GraphQL API with queries, mutations, and subscriptions
//! - **REST API**: RESTful API with versioning and OpenAPI documentation
//! - **Rate Limiting**: Per-user and per-endpoint rate limiting with token bucket algorithm
//! - **Webhooks**: Event-driven webhook system with retry logic
//! - **Authentication**: JWT-based authentication with context propagation
//! - **DataLoaders**: Efficient data loading to prevent N+1 queries
//! - **Real-time Updates**: WebSocket-based subscriptions for live data
//! - **OpenAPI Documentation**: Auto-generated API documentation with Swagger UI
//!
//! ## Architecture
//!
//! The API layer acts as the entry point for all client requests, coordinating between
//! various domain crates (torrent, user, community, etc.) and providing a unified interface.
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                        API Layer                            │
//! ├─────────────────────┬───────────────────────────────────────┤
//! │   GraphQL           │           REST                        │
//! │  - Queries          │  - GET /api/v1/torrents               │
//! │  - Mutations        │  - POST /api/v1/torrents/upload       │
//! │  - Subscriptions    │  - GET /api/v1/users/:id              │
//! └─────────────────────┴───────────────────────────────────────┘
//!              │                        │
//!              ├────────────────────────┤
//!              ▼                        ▼
//! ┌────────────────────────────────────────────────────────────┐
//! │     Domain Services (torrent, user, community, etc.)       │
//! └────────────────────────────────────────────────────────────┘
//! ```

pub mod graphql;
pub mod openapi;
pub mod rate_limit;
pub mod rest;
pub mod webhooks;

use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::{info, instrument};

/// API service configuration
#[derive(Debug, Clone)]
pub struct ApiConfig {
    /// Host to bind to
    pub host: String,
    /// Port to bind to
    pub port: u16,
    /// Enable GraphQL playground
    pub enable_graphql_playground: bool,
    /// Enable Swagger UI
    pub enable_swagger_ui: bool,
    /// Enable CORS
    pub enable_cors: bool,
    /// Rate limit: requests per minute
    pub rate_limit_per_minute: u32,
    /// JWT secret for authentication
    pub jwt_secret: String,
    /// Database connection pool
    pub database_url: String,
    /// Redis connection URL
    pub redis_url: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            enable_graphql_playground: true,
            enable_swagger_ui: true,
            enable_cors: true,
            rate_limit_per_minute: 60,
            jwt_secret: "change-me-in-production".to_string(),
            database_url: "postgres://localhost/tracker".to_string(),
            redis_url: "redis://localhost/".to_string(),
        }
    }
}

/// API service state shared across all handlers
#[derive(Clone)]
pub struct ApiState {
    /// Configuration
    pub config: ApiConfig,
    /// Database connection pool
    pub db_pool: sqlx::PgPool,
    /// Redis connection
    pub redis_client: redis::Client,
    /// GraphQL schema
    pub graphql_schema: graphql::TrackerSchema,
    /// Rate limiter
    pub rate_limiter: Arc<rate_limit::RateLimiter>,
    /// Webhook manager
    pub webhook_manager: Arc<webhooks::WebhookManager>,
}

impl ApiState {
    /// Create a new API state
    #[instrument(skip(config))]
    pub async fn new(config: ApiConfig) -> Result<Self> {
        info!("Initializing API state");

        // Create database pool
        let db_pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(50)
            .connect(&config.database_url)
            .await?;

        // Create Redis client
        let redis_client = redis::Client::open(config.redis_url.as_str())?;

        // Create GraphQL schema with dataloaders
        let graphql_schema = graphql::create_schema(db_pool.clone(), redis_client.clone()).await?;

        // Create rate limiter
        let rate_limiter = Arc::new(rate_limit::RateLimiter::new(
            redis_client.clone(),
            config.rate_limit_per_minute,
        ));

        // Create webhook manager
        let webhook_manager = Arc::new(webhooks::WebhookManager::new(
            db_pool.clone(),
            redis_client.clone(),
        ));

        Ok(Self {
            config,
            db_pool,
            redis_client,
            graphql_schema,
            rate_limiter,
            webhook_manager,
        })
    }
}

/// Main API service
pub struct ApiService {
    state: Arc<ApiState>,
    router: Router,
}

impl ApiService {
    /// Create a new API service
    #[instrument(skip(config))]
    pub async fn new(config: ApiConfig) -> Result<Self> {
        info!("Creating API service");

        let state = Arc::new(ApiState::new(config.clone()).await?);
        let router = Self::create_router(state.clone()).await?;

        Ok(Self { state, router })
    }

    /// Create the main router with all endpoints
    #[instrument(skip(state))]
    async fn create_router(state: Arc<ApiState>) -> Result<Router> {
        info!("Creating API router");

        let mut app = Router::new()
            // Health check endpoint
            .route("/health", get(health_check))
            .route("/ready", get(readiness_check));

        // Add GraphQL endpoints
        app = graphql::configure_routes(app, state.clone());

        // Add REST endpoints
        app = rest::configure_routes(app, state.clone());

        // Add OpenAPI documentation
        if state.config.enable_swagger_ui {
            app = openapi::configure_routes(app);
        }

        // Add middleware
        app = app
            .layer(tower_http::trace::TraceLayer::new_for_http())
            .layer(tower_http::compression::CompressionLayer::new());

        // Add CORS if enabled
        if state.config.enable_cors {
            app = app.layer(CorsLayer::permissive());
        }

        // Add state
        app = app.with_state(state);

        Ok(app)
    }

    /// Get the router
    pub fn router(&self) -> Router {
        self.router.clone()
    }

    /// Run the API service
    #[instrument(skip(self))]
    pub async fn run(self) -> Result<()> {
        let addr = format!("{}:{}", self.state.config.host, self.state.config.port);
        info!("Starting API service on {}", addr);

        let listener = tokio::net::TcpListener::bind(&addr).await?;

        info!("API service listening on {}", addr);
        info!("GraphQL endpoint: http://{}/graphql", addr);

        if self.state.config.enable_graphql_playground {
            info!("GraphQL Playground: http://{}/graphql/playground", addr);
        }

        if self.state.config.enable_swagger_ui {
            info!("Swagger UI: http://{}/swagger-ui", addr);
        }

        axum::serve(listener, self.router).await?;

        Ok(())
    }
}

/// Health check endpoint
#[instrument]
async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// Readiness check endpoint - verifies database and Redis connections
#[instrument(skip(state))]
async fn readiness_check(State(state): State<Arc<ApiState>>) -> Response {
    // Check database
    if let Err(e) = sqlx::query("SELECT 1").execute(&state.db_pool).await {
        tracing::error!("Database health check failed: {}", e);
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            format!("Database unavailable: {}", e),
        )
            .into_response();
    }

    // Check Redis
    let mut conn = match state.redis_client.get_multiplexed_async_connection().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Redis health check failed: {}", e);
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                format!("Redis unavailable: {}", e),
            )
                .into_response();
        }
    };

    if let Err(e) = redis::cmd("PING").query_async::<_, String>(&mut conn).await {
        tracing::error!("Redis ping failed: {}", e);
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            format!("Redis unavailable: {}", e),
        )
            .into_response();
    }

    (StatusCode::OK, "Ready").into_response()
}

/// API error type
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Authorization failed: {0}")]
    AuthorizationError(String),

    #[error("Validation failed: {0}")]
    ValidationError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Internal server error: {0}")]
    InternalError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::AuthenticationError(msg) => (StatusCode::UNAUTHORIZED, msg),
            ApiError::AuthorizationError(msg) => (StatusCode::FORBIDDEN, msg),
            ApiError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::RateLimitExceeded => {
                (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded".to_string())
            }
            ApiError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::DatabaseError(e) => {
                tracing::error!("Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error".to_string(),
                )
            }
            ApiError::RedisError(e) => {
                tracing::error!("Redis error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Cache error".to_string(),
                )
            }
            ApiError::Other(e) => {
                tracing::error!("Internal error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
        };

        let body = serde_json::json!({
            "error": message,
        });

        (status, axum::Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_config_default() {
        let config = ApiConfig::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
        assert!(config.enable_graphql_playground);
        assert!(config.enable_swagger_ui);
    }

    #[tokio::test]
    async fn test_health_check() {
        let response = health_check().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
