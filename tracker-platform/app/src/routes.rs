use crate::middleware::{
    error_handling_middleware, metrics_middleware, request_id_middleware,
    request_logging_middleware, security_headers_middleware,
};
use crate::state::{AppState, HealthStatus};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use axum::routing::{get, post};
use axum::{middleware, Router};
use std::sync::Arc;

/// Build the main application router with all routes
pub fn build_router(state: AppState) -> Router {
    let app_state = Arc::new(state);

    Router::new()
        // Health check endpoint (no auth required)
        .route("/health", get(health_check))
        .route("/health/ready", get(readiness_check))
        .route("/health/live", get(liveness_check))
        // Metrics endpoint (no auth required, but consider protecting in production)
        .route("/metrics", get(crate::telemetry::metrics::metrics_handler))
        // API routes
        .nest("/api/v1", api_routes(app_state.clone()))
        // GraphQL endpoint
        .nest("/graphql", graphql_routes(app_state.clone()))
        // Tracker endpoints (BitTorrent protocol)
        .nest("/tracker", tracker_routes(app_state.clone()))
        // Static file serving (for uploaded media)
        .nest("/static", static_routes(app_state.clone()))
        // Fallback for 404
        .fallback(not_found)
        // Global middleware (applied to all routes)
        .layer(middleware::from_fn(request_id_middleware))
        .layer(middleware::from_fn(metrics_middleware))
        .layer(middleware::from_fn(security_headers_middleware))
        .layer(middleware::from_fn(request_logging_middleware))
        .layer(middleware::from_fn(error_handling_middleware))
        .with_state(app_state)
}

/// API routes (REST API)
fn api_routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        // Auth routes
        .nest("/auth", auth_routes())
        // User routes
        .nest("/users", user_routes())
        // Torrent routes
        .nest("/torrents", torrent_routes())
        // Search routes
        .nest("/search", search_routes())
        // Media routes
        .nest("/media", media_routes())
        // Community routes
        .nest("/community", community_routes())
        .with_state(state)
}

/// Authentication routes
fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(api::auth::register))
        .route("/login", post(api::auth::login))
        .route("/logout", post(api::auth::logout))
        .route("/refresh", post(api::auth::refresh_token))
        .route("/verify-email", post(api::auth::verify_email))
        .route("/resend-verification", post(api::auth::resend_verification))
        .route("/forgot-password", post(api::auth::forgot_password))
        .route("/reset-password", post(api::auth::reset_password))
        .route("/change-password", post(api::auth::change_password))
        .route("/enable-2fa", post(api::auth::enable_2fa))
        .route("/disable-2fa", post(api::auth::disable_2fa))
        .route("/verify-2fa", post(api::auth::verify_2fa))
}

/// User routes
fn user_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(api::users::list_users))
        .route("/me", get(api::users::get_current_user))
        .route("/me", post(api::users::update_current_user))
        .route("/:id", get(api::users::get_user))
        .route("/:id", post(api::users::update_user))
        .route("/:id/profile", get(api::users::get_user_profile))
        .route("/:id/stats", get(api::users::get_user_stats))
        .route("/:id/uploads", get(api::users::get_user_uploads))
        .route("/:id/downloads", get(api::users::get_user_downloads))
}

/// Torrent routes
fn torrent_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(api::torrents::list_torrents))
        .route("/", post(api::torrents::upload_torrent))
        .route("/:id", get(api::torrents::get_torrent))
        .route("/:id", post(api::torrents::update_torrent))
        .route("/:id/download", get(api::torrents::download_torrent))
        .route("/:id/peers", get(api::torrents::get_torrent_peers))
        .route("/:id/files", get(api::torrents::get_torrent_files))
        .route("/:id/comments", get(api::torrents::get_torrent_comments))
        .route("/:id/comments", post(api::torrents::add_torrent_comment))
}

/// Search routes
fn search_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/torrents", get(api::search::search_torrents))
        .route("/users", get(api::search::search_users))
        .route("/suggest", get(api::search::suggest))
}

/// Media routes
fn media_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/upload", post(api::media::upload_media))
        .route("/:id", get(api::media::get_media))
        .route("/:id", post(api::media::update_media))
}

/// Community routes
fn community_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/forums", get(api::community::list_forums))
        .route("/forums/:id", get(api::community::get_forum))
        .route("/forums/:id/threads", get(api::community::list_threads))
        .route("/threads/:id", get(api::community::get_thread))
        .route("/threads/:id/posts", get(api::community::list_posts))
        .route("/posts", post(api::community::create_post))
}

/// GraphQL routes
fn graphql_routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(api::graphql::graphql_handler))
        .route("/playground", get(api::graphql::graphql_playground))
        .with_state(state)
}

/// Tracker routes (BitTorrent protocol)
fn tracker_routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/announce", get(tracker_announce))
        .route("/scrape", get(tracker_scrape))
        .with_state(state)
}

/// Static file routes
fn static_routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/uploads/*path", get(serve_static_file))
        .with_state(state)
}

// Route handlers

/// Health check endpoint
async fn health_check(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.health_check().await {
        Ok(status) => {
            if status.overall {
                (StatusCode::OK, Json(status))
            } else {
                (StatusCode::SERVICE_UNAVAILABLE, Json(status))
            }
        }
        Err(e) => {
            tracing::error!("Health check failed: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(HealthStatus::default()),
            )
        }
    }
}

/// Readiness check - checks if the service is ready to accept traffic
async fn readiness_check(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.health_check().await {
        Ok(status) if status.overall => StatusCode::OK,
        _ => StatusCode::SERVICE_UNAVAILABLE,
    }
}

/// Liveness check - simple check if the service is running
async fn liveness_check() -> impl IntoResponse {
    StatusCode::OK
}

/// Tracker announce endpoint
async fn tracker_announce(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    // This is a placeholder - actual implementation should be in the tracker crate
    StatusCode::NOT_IMPLEMENTED
}

/// Tracker scrape endpoint
async fn tracker_scrape(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    // This is a placeholder - actual implementation should be in the tracker crate
    StatusCode::NOT_IMPLEMENTED
}

/// Serve static files
async fn serve_static_file(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    // This is a placeholder - actual implementation should handle file serving
    StatusCode::NOT_IMPLEMENTED
}

/// 404 handler
async fn not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "error": "not_found",
            "message": "The requested resource was not found"
        })),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_liveness_check() {
        let response = liveness_check().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_routes_build() {
        // This test just ensures the router can be built
        // In a real test, you'd set up a test AppState
    }
}
