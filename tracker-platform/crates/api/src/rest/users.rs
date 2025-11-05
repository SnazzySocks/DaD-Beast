//! # User REST Endpoints
//!
//! RESTful API endpoints for user operations.

use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::IntoResponse,
    routing::{get, patch},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::instrument;

use crate::{ApiError, ApiState};
use super::{ErrorResponse, require_auth};

/// User response DTO
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UserResponse {
    pub id: uuid::Uuid,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub user_class: String,
    pub uploaded: i64,
    pub downloaded: i64,
    pub is_active: bool,
    pub is_verified: bool,
}

/// User statistics response
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UserStatisticsResponse {
    pub user_id: uuid::Uuid,
    pub torrents_uploaded: i64,
    pub torrents_seeding: i64,
    pub torrents_leeching: i64,
    pub forum_posts: i64,
    pub bonus_points: i64,
    pub invites_available: i32,
    pub ratio: f64,
}

/// User update request
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    /// Avatar URL
    pub avatar_url: Option<String>,
    /// User bio
    pub bio: Option<String>,
}

/// Configure user routes
pub fn routes() -> Router<Arc<ApiState>> {
    Router::new()
        .route("/:id", get(get_user).patch(update_user))
        .route("/:id/stats", get(get_user_stats))
        .route("/:id/torrents", get(get_user_torrents))
}

/// Get user by ID
#[utoipa::path(
    get,
    path = "/api/v1/users/{id}",
    tag = "users",
    params(
        ("id" = uuid::Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User details", body = UserResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
#[instrument(skip(state))]
async fn get_user(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<UserResponse>, ApiError> {
    let user = sqlx::query_as::<_, UserResponse>(
        "SELECT id, username, email, created_at, user_class, uploaded, downloaded,
         is_active, is_verified
         FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    Ok(Json(user))
}

/// Update user profile
#[utoipa::path(
    patch,
    path = "/api/v1/users/{id}",
    tag = "users",
    params(
        ("id" = uuid::Uuid, Path, description = "User ID")
    ),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated successfully", body = UserResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(state, headers))]
async fn update_user(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Path(id): Path<uuid::Uuid>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, ApiError> {
    let user_id = require_auth(&headers).await?;

    // Users can only update their own profile
    if user_id != id {
        return Err(ApiError::AuthorizationError(
            "Not authorized to update this user".to_string(),
        ));
    }

    // Get current user
    let current = sqlx::query_as::<_, UserResponse>(
        "SELECT id, username, email, created_at, user_class, uploaded, downloaded,
         is_active, is_verified
         FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    // Update email if provided and validate
    let email = if let Some(new_email) = request.email {
        // TODO: Validate email format
        // TODO: Check if email is already in use
        new_email
    } else {
        current.email
    };

    // Update user
    let user = sqlx::query_as::<_, UserResponse>(
        "UPDATE users SET email = $1 WHERE id = $2
         RETURNING id, username, email, created_at, user_class, uploaded, downloaded,
                   is_active, is_verified",
    )
    .bind(&email)
    .bind(id)
    .fetch_one(&state.db_pool)
    .await?;

    // TODO: Update profile fields (avatar_url, bio) in separate user_profiles table

    tracing::info!("User updated: {}", id);

    Ok(Json(user))
}

/// Get user statistics
#[utoipa::path(
    get,
    path = "/api/v1/users/{id}/stats",
    tag = "users",
    params(
        ("id" = uuid::Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User statistics", body = UserStatisticsResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
#[instrument(skip(state))]
async fn get_user_stats(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<UserStatisticsResponse>, ApiError> {
    // Check if user exists
    let user_exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(&state.db_pool)
        .await?;

    if user_exists == 0 {
        return Err(ApiError::NotFound("User not found".to_string()));
    }

    // Get user upload/download for ratio calculation
    let (uploaded, downloaded): (i64, i64) = sqlx::query_as(
        "SELECT uploaded, downloaded FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&state.db_pool)
    .await?;

    let ratio = if downloaded == 0 {
        f64::INFINITY
    } else {
        uploaded as f64 / downloaded as f64
    };

    // Get statistics
    let stats = sqlx::query_as::<_, UserStatisticsResponse>(
        "SELECT user_id, torrents_uploaded, torrents_seeding, torrents_leeching,
         forum_posts, bonus_points, invites_available, 0.0 as ratio
         FROM user_statistics WHERE user_id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await?
    .map(|mut s| {
        s.ratio = ratio;
        s
    })
    .unwrap_or_else(|| UserStatisticsResponse {
        user_id: id,
        torrents_uploaded: 0,
        torrents_seeding: 0,
        torrents_leeching: 0,
        forum_posts: 0,
        bonus_points: 0,
        invites_available: 0,
        ratio,
    });

    Ok(Json(stats))
}

/// Get user's uploaded torrents
#[utoipa::path(
    get,
    path = "/api/v1/users/{id}/torrents",
    tag = "users",
    params(
        ("id" = uuid::Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User's torrents", body = Vec<super::torrents::TorrentResponse>),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
#[instrument(skip(state))]
async fn get_user_torrents(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<Vec<super::torrents::TorrentResponse>>, ApiError> {
    // Check if user exists
    let user_exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(&state.db_pool)
        .await?;

    if user_exists == 0 {
        return Err(ApiError::NotFound("User not found".to_string()));
    }

    // Get user's torrents
    let torrents = sqlx::query_as::<_, super::torrents::TorrentResponse>(
        "SELECT * FROM torrents WHERE uploader_id = $1 ORDER BY created_at DESC LIMIT 100",
    )
    .bind(id)
    .fetch_all(&state.db_pool)
    .await?;

    Ok(Json(torrents))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_response_serialization() {
        let response = UserResponse {
            id: uuid::Uuid::new_v4(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            created_at: Utc::now(),
            user_class: "user".to_string(),
            uploaded: 1000000,
            downloaded: 500000,
            is_active: true,
            is_verified: true,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("testuser"));
    }

    #[test]
    fn test_ratio_calculation() {
        let uploaded = 1000;
        let downloaded = 500;
        let ratio = uploaded as f64 / downloaded as f64;
        assert_eq!(ratio, 2.0);
    }
}
