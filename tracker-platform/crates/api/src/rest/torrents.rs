//! # Torrent REST Endpoints
//!
//! RESTful API endpoints for torrent operations.

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, patch, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::instrument;

use crate::{ApiError, ApiState};
use super::{ErrorResponse, PaginatedResponse, PaginationMeta, PaginationParams, require_auth};

/// Torrent response DTO
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TorrentResponse {
    pub id: uuid::Uuid,
    pub info_hash: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub size: i64,
    pub file_count: i32,
    pub uploader_id: uuid::Uuid,
    pub created_at: DateTime<Utc>,
    pub seeders: i32,
    pub leechers: i32,
    pub times_completed: i32,
    pub is_freeleech: bool,
    pub is_featured: bool,
}

/// Torrent search/filter parameters
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct TorrentSearchParams {
    /// Search query
    pub q: Option<String>,
    /// Category filter
    pub category: Option<String>,
    /// Sort field
    #[serde(default)]
    pub sort: TorrentSortField,
    /// Sort order
    #[serde(default)]
    pub order: SortOrder,
    /// Only show freeleech torrents
    #[serde(default)]
    pub freeleech: bool,
}

#[derive(Debug, Deserialize, Default, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum TorrentSortField {
    #[default]
    #[serde(rename = "created_at")]
    CreatedAt,
    #[serde(rename = "name")]
    Name,
    #[serde(rename = "size")]
    Size,
    #[serde(rename = "seeders")]
    Seeders,
    #[serde(rename = "leechers")]
    Leechers,
    #[serde(rename = "times_completed")]
    TimesCompleted,
}

#[derive(Debug, Deserialize, Default, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    #[default]
    #[serde(rename = "desc")]
    Desc,
    #[serde(rename = "asc")]
    Asc,
}

/// Torrent upload request
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UploadTorrentRequest {
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub info_hash: String,
    /// Base64-encoded torrent file
    pub torrent_file: String,
}

/// Torrent update request
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateTorrentRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
}

/// Configure torrent routes
pub fn routes() -> Router<Arc<ApiState>> {
    Router::new()
        .route("/", get(list_torrents).post(upload_torrent))
        .route("/:id", get(get_torrent).patch(update_torrent))
        .route("/:id/download", get(download_torrent))
}

/// List torrents with filtering and pagination
#[utoipa::path(
    get,
    path = "/api/v1/torrents",
    tag = "torrents",
    params(
        PaginationParams,
        TorrentSearchParams
    ),
    responses(
        (status = 200, description = "List of torrents", body = PaginatedResponse<TorrentResponse>),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
#[instrument(skip(state))]
async fn list_torrents(
    State(state): State<Arc<ApiState>>,
    Query(pagination): Query<PaginationParams>,
    Query(search): Query<TorrentSearchParams>,
) -> Result<Json<PaginatedResponse<TorrentResponse>>, ApiError> {
    // Build query based on filters
    let mut conditions = vec![];
    let mut query_params: Vec<String> = vec![];

    if let Some(ref q) = search.q {
        conditions.push(format!("name ILIKE '%{}%'", q.replace("'", "''")));
    }

    if let Some(ref cat) = search.category {
        conditions.push(format!("category = '{}'", cat.replace("'", "''")));
    }

    if search.freeleech {
        conditions.push("is_freeleech = true".to_string());
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    // Build order by clause
    let sort_field = match search.sort {
        TorrentSortField::CreatedAt => "created_at",
        TorrentSortField::Name => "name",
        TorrentSortField::Size => "size",
        TorrentSortField::Seeders => "seeders",
        TorrentSortField::Leechers => "leechers",
        TorrentSortField::TimesCompleted => "times_completed",
    };

    let sort_order = match search.order {
        SortOrder::Asc => "ASC",
        SortOrder::Desc => "DESC",
    };

    // Get total count
    let count_sql = format!("SELECT COUNT(*) FROM torrents {}", where_clause);
    let total: i64 = sqlx::query_scalar(&count_sql)
        .fetch_one(&state.db_pool)
        .await?;

    // Get torrents
    let sql = format!(
        "SELECT * FROM torrents {} ORDER BY {} {} LIMIT $1 OFFSET $2",
        where_clause, sort_field, sort_order
    );

    let torrents: Vec<TorrentResponse> = sqlx::query_as(&sql)
        .bind(pagination.limit())
        .bind(pagination.offset())
        .fetch_all(&state.db_pool)
        .await?;

    Ok(Json(PaginatedResponse {
        data: torrents,
        pagination: PaginationMeta::new(pagination.page, pagination.per_page, total),
    }))
}

/// Get a specific torrent by ID
#[utoipa::path(
    get,
    path = "/api/v1/torrents/{id}",
    tag = "torrents",
    params(
        ("id" = uuid::Uuid, Path, description = "Torrent ID")
    ),
    responses(
        (status = 200, description = "Torrent details", body = TorrentResponse),
        (status = 404, description = "Torrent not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
#[instrument(skip(state))]
async fn get_torrent(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<TorrentResponse>, ApiError> {
    let torrent = sqlx::query_as::<_, TorrentResponse>(
        "SELECT * FROM torrents WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| ApiError::NotFound("Torrent not found".to_string()))?;

    Ok(Json(torrent))
}

/// Upload a new torrent
#[utoipa::path(
    post,
    path = "/api/v1/torrents",
    tag = "torrents",
    request_body = UploadTorrentRequest,
    responses(
        (status = 201, description = "Torrent uploaded successfully", body = TorrentResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 409, description = "Torrent already exists", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(state, headers))]
async fn upload_torrent(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(request): Json<UploadTorrentRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let user_id = require_auth(&headers).await?;

    // Validate input
    if request.name.is_empty() {
        return Err(ApiError::ValidationError("Torrent name is required".to_string()));
    }

    if request.info_hash.len() != 40 {
        return Err(ApiError::ValidationError("Invalid info hash".to_string()));
    }

    // Check if torrent already exists
    let existing = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM torrents WHERE info_hash = $1",
    )
    .bind(&request.info_hash)
    .fetch_one(&state.db_pool)
    .await?;

    if existing > 0 {
        return Err(ApiError::ValidationError("Torrent already exists".to_string()));
    }

    // Decode torrent file
    let torrent_data = base64::decode(&request.torrent_file)
        .map_err(|_| ApiError::ValidationError("Invalid base64-encoded torrent file".to_string()))?;

    // TODO: Parse torrent file to extract metadata
    let file_count = 1;
    let size = torrent_data.len() as i64;

    // Insert torrent
    let torrent = sqlx::query_as::<_, TorrentResponse>(
        "INSERT INTO torrents
         (id, info_hash, name, description, category, size, file_count,
          uploader_id, created_at, seeders, leechers, times_completed,
          is_freeleech, is_featured)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 0, 0, 0, false, false)
         RETURNING *",
    )
    .bind(uuid::Uuid::new_v4())
    .bind(&request.info_hash)
    .bind(&request.name)
    .bind(&request.description)
    .bind(&request.category)
    .bind(size)
    .bind(file_count)
    .bind(user_id)
    .bind(Utc::now())
    .fetch_one(&state.db_pool)
    .await?;

    tracing::info!("Torrent uploaded: {} by user {}", torrent.id, user_id);

    Ok((StatusCode::CREATED, Json(torrent)))
}

/// Update a torrent
#[utoipa::path(
    patch,
    path = "/api/v1/torrents/{id}",
    tag = "torrents",
    params(
        ("id" = uuid::Uuid, Path, description = "Torrent ID")
    ),
    request_body = UpdateTorrentRequest,
    responses(
        (status = 200, description = "Torrent updated successfully", body = TorrentResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Torrent not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(state, headers))]
async fn update_torrent(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Path(id): Path<uuid::Uuid>,
    Json(request): Json<UpdateTorrentRequest>,
) -> Result<Json<TorrentResponse>, ApiError> {
    let user_id = require_auth(&headers).await?;

    // Get existing torrent
    let existing = sqlx::query_as::<_, TorrentResponse>(
        "SELECT * FROM torrents WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| ApiError::NotFound("Torrent not found".to_string()))?;

    // Check authorization
    if existing.uploader_id != user_id {
        return Err(ApiError::AuthorizationError(
            "Not authorized to update this torrent".to_string(),
        ));
    }

    // Update fields (keep existing if not provided)
    let name = request.name.unwrap_or(existing.name);
    let description = request.description.or(existing.description);
    let category = request.category.unwrap_or(existing.category);

    let torrent = sqlx::query_as::<_, TorrentResponse>(
        "UPDATE torrents
         SET name = $1, description = $2, category = $3
         WHERE id = $4
         RETURNING *",
    )
    .bind(&name)
    .bind(&description)
    .bind(&category)
    .bind(id)
    .fetch_one(&state.db_pool)
    .await?;

    tracing::info!("Torrent updated: {}", id);

    Ok(Json(torrent))
}

/// Download a torrent file
#[utoipa::path(
    get,
    path = "/api/v1/torrents/{id}/download",
    tag = "torrents",
    params(
        ("id" = uuid::Uuid, Path, description = "Torrent ID")
    ),
    responses(
        (status = 200, description = "Torrent file", content_type = "application/x-bittorrent"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Torrent not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[instrument(skip(state, headers))]
async fn download_torrent(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Path(id): Path<uuid::Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let user_id = require_auth(&headers).await?;

    // Check if torrent exists
    let torrent = sqlx::query_as::<_, TorrentResponse>(
        "SELECT * FROM torrents WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| ApiError::NotFound("Torrent not found".to_string()))?;

    // TODO: Retrieve torrent file from storage
    // TODO: Personalize announce URL with user's passkey
    // TODO: Track download stats

    tracing::info!("Torrent downloaded: {} by user {}", id, user_id);

    // For now, return placeholder response
    Ok((
        StatusCode::OK,
        [("Content-Type", "application/x-bittorrent")],
        vec![],
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_torrent_response_serialization() {
        let response = TorrentResponse {
            id: uuid::Uuid::new_v4(),
            info_hash: "a".repeat(40),
            name: "Test Torrent".to_string(),
            description: Some("Description".to_string()),
            category: "movies".to_string(),
            size: 1024,
            file_count: 1,
            uploader_id: uuid::Uuid::new_v4(),
            created_at: Utc::now(),
            seeders: 10,
            leechers: 5,
            times_completed: 100,
            is_freeleech: false,
            is_featured: false,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("Test Torrent"));
    }
}
