//! Torrent upload handler
//!
//! This module handles the complete torrent upload workflow:
//! 1. Parse .torrent file (bencode)
//! 2. Extract info hash (SHA1 of info dict)
//! 3. Validate torrent structure
//! 4. Extract file list and sizes
//! 5. Calculate total size
//! 6. Support multi-file and single-file torrents
//! 7. Validate announce URL
//! 8. Store in database with PENDING moderation status
//! 9. Queue for search indexing

use anyhow::{anyhow, Context, Result};
use axum::{
    extract::{DefaultBodyLimit, Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    bencode::{Torrent, TorrentInfo},
    files::{parse_file_list, validate_file_list, TorrentFileInfo},
    metadata::{determine_media_type, parse_quality_from_name, QualityInfo},
    moderation::{AutoApprovalRules, ModerationService, ModerationStatus},
};

/// Maximum torrent file size (1MB)
const MAX_TORRENT_FILE_SIZE: usize = 1024 * 1024;

/// Upload request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UploadRequest {
    /// Torrent display name (optional, uses .torrent name if not provided)
    #[validate(length(min = 3, max = 255))]
    pub name: Option<String>,

    /// Description (Markdown supported)
    #[validate(length(max = 50000))]
    pub description: Option<String>,

    /// Category ID
    pub category_id: Uuid,

    /// Tags (comma-separated or array)
    pub tags: Option<Vec<String>>,

    /// NFO file content (optional)
    pub nfo_content: Option<String>,

    /// External IDs
    pub tmdb_id: Option<i64>,
    pub imdb_id: Option<String>,
    pub tvdb_id: Option<i64>,
    pub igdb_id: Option<i64>,

    /// Media URLs
    pub poster_url: Option<String>,
    pub screenshots: Option<Vec<String>>,

    /// Year
    pub year: Option<i32>,

    /// Anonymous upload (hide uploader)
    pub anonymous: Option<bool>,
}

/// Upload response
#[derive(Debug, Clone, Serialize)]
pub struct UploadResponse {
    /// Torrent ID
    pub torrent_id: Uuid,

    /// Info hash
    pub info_hash: String,

    /// Moderation status
    pub moderation_status: ModerationStatus,

    /// Auto-approved
    pub auto_approved: bool,

    /// Message
    pub message: String,
}

/// Upload error
#[derive(Debug, Serialize)]
pub struct UploadError {
    pub error: String,
    pub details: Option<Vec<String>>,
}

impl IntoResponse for UploadError {
    fn into_response(self) -> axum::response::Response {
        let status = StatusCode::BAD_REQUEST;
        (status, Json(self)).into_response()
    }
}

/// Torrent upload service
pub struct UploadService {
    pool: PgPool,
    moderation: ModerationService,
    auto_approval_rules: AutoApprovalRules,
}

impl UploadService {
    /// Create new upload service
    pub fn new(pool: PgPool, auto_approval_rules: AutoApprovalRules) -> Self {
        let moderation = ModerationService::new(pool.clone());
        Self {
            pool,
            moderation,
            auto_approval_rules,
        }
    }

    /// Process torrent upload
    pub async fn upload_torrent(
        &self,
        user_id: Uuid,
        torrent_data: Vec<u8>,
        request: UploadRequest,
    ) -> Result<UploadResponse> {
        // Validate request
        request.validate().context("Invalid upload request")?;

        // Validate torrent file size
        if torrent_data.len() > MAX_TORRENT_FILE_SIZE {
            return Err(anyhow!(
                "Torrent file too large (max {}KB)",
                MAX_TORRENT_FILE_SIZE / 1024
            ));
        }

        // Parse torrent file
        let torrent_info = Torrent::parse(&torrent_data)
            .context("Failed to parse torrent file")?;

        // Validate torrent structure
        torrent_info.torrent.info.validate()
            .context("Invalid torrent structure")?;

        // Check for duplicates
        let duplicate_check = self.moderation.check_duplicates(&torrent_info.info_hash).await?;
        if duplicate_check.is_duplicate {
            return Err(anyhow!(
                "Duplicate torrent detected (info_hash: {})",
                torrent_info.info_hash
            ));
        }

        // Validate announce URL
        if let Some(ref announce) = torrent_info.torrent.announce {
            crate::bencode::validate_announce_url(announce)
                .context("Invalid announce URL")?;
        }

        // Parse file list
        let file_list = parse_file_list(
            torrent_info.file_list.clone(),
            &torrent_info.torrent.info.name,
        )?;

        // Validate file list
        let file_validation = validate_file_list(&file_list);
        if !file_validation.is_valid {
            return Err(anyhow!(
                "File list validation failed: {}",
                file_validation.errors.join(", ")
            ));
        }

        // Get category information
        let category = self.get_category(request.category_id).await?;

        // Determine media type
        let file_stats = crate::files::calculate_statistics(&file_list);
        let media_type = determine_media_type(&category.name, &file_stats);

        // Parse quality information from name
        let torrent_name = request.name.as_ref()
            .unwrap_or(&torrent_info.torrent.info.name);
        let quality = parse_quality_from_name(torrent_name);

        // Check auto-approval eligibility
        let auto_approved = self
            .moderation
            .check_auto_approval(user_id, &self.auto_approval_rules)
            .await?;

        let moderation_status = if auto_approved {
            ModerationStatus::Approved
        } else {
            ModerationStatus::Pending
        };

        // Create torrent ID
        let torrent_id = Uuid::new_v4();

        // Insert into database
        self.insert_torrent(
            torrent_id,
            user_id,
            &torrent_info,
            &request,
            &file_list,
            &quality,
            media_type,
            moderation_status,
        )
        .await?;

        // Store .torrent file
        self.store_torrent_file(torrent_id, &torrent_data).await?;

        // Queue for search indexing
        if moderation_status == ModerationStatus::Approved {
            self.queue_for_indexing(torrent_id).await?;
        }

        let message = if auto_approved {
            "Torrent uploaded and auto-approved successfully".to_string()
        } else {
            "Torrent uploaded successfully and is pending moderation".to_string()
        };

        Ok(UploadResponse {
            torrent_id,
            info_hash: torrent_info.info_hash,
            moderation_status,
            auto_approved,
            message,
        })
    }

    /// Insert torrent into database
    async fn insert_torrent(
        &self,
        torrent_id: Uuid,
        user_id: Uuid,
        torrent_info: &TorrentInfo,
        request: &UploadRequest,
        file_list: &[TorrentFileInfo],
        quality: &QualityInfo,
        media_type: crate::metadata::MediaType,
        moderation_status: ModerationStatus,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        let torrent_name = request.name.as_ref()
            .unwrap_or(&torrent_info.torrent.info.name)
            .clone();

        // Insert main torrent record
        sqlx::query!(
            r#"
            INSERT INTO torrents (
                id, info_hash, name, total_size, piece_length,
                piece_count, is_private, uploader_id, category_id,
                moderation_status, created_at, is_multi_file,
                announce_url, anonymous
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW(), $11, $12, $13
            )
            "#,
            torrent_id,
            torrent_info.info_hash,
            torrent_name,
            torrent_info.total_size,
            torrent_info.torrent.info.piece_length,
            torrent_info.piece_count as i32,
            torrent_info.is_private,
            user_id,
            request.category_id,
            moderation_status as ModerationStatus,
            torrent_info.is_multi_file,
            torrent_info.torrent.announce,
            request.anonymous.unwrap_or(false),
        )
        .execute(&mut *tx)
        .await?;

        // Insert file list
        for (idx, file) in file_list.iter().enumerate() {
            sqlx::query!(
                r#"
                INSERT INTO torrent_files (
                    id, torrent_id, path, size, file_index,
                    extension, file_type, media_type, is_sample
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9
                )
                "#,
                Uuid::new_v4(),
                torrent_id,
                file.path,
                file.size,
                idx as i32,
                file.extension,
                serde_json::to_value(&file.file_type)?,
                serde_json::to_value(&file.media_type)?,
                file.is_sample,
            )
            .execute(&mut *tx)
            .await?;
        }

        // Insert metadata
        let external_ids = crate::metadata::ExternalIds {
            tmdb_id: request.tmdb_id,
            imdb_id: request.imdb_id.clone(),
            tvdb_id: request.tvdb_id,
            igdb_id: request.igdb_id,
            ..Default::default()
        };

        let media_urls = crate::metadata::MediaUrls {
            poster_url: request.poster_url.clone(),
            screenshots: request.screenshots.clone().unwrap_or_default(),
            ..Default::default()
        };

        sqlx::query!(
            r#"
            INSERT INTO torrent_metadata (
                id, name, description, category_id, media_type,
                tags, quality, external_ids, nfo_content, media_urls,
                year, is_featured, is_sticky
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, false, false
            )
            "#,
            torrent_id,
            torrent_name,
            request.description,
            request.category_id,
            media_type as crate::metadata::MediaType,
            request.tags.as_ref().map(|t| t.as_slice()).unwrap_or(&[]),
            serde_json::to_value(quality)?,
            serde_json::to_value(&external_ids)?,
            request.nfo_content,
            serde_json::to_value(&media_urls)?,
            request.year,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    /// Store .torrent file (could be filesystem, S3, etc.)
    async fn store_torrent_file(&self, torrent_id: Uuid, data: &[u8]) -> Result<()> {
        // In a real implementation, this would store to S3/filesystem
        // For now, we'll store in database
        sqlx::query!(
            r#"
            UPDATE torrents
            SET torrent_file_data = $2
            WHERE id = $1
            "#,
            torrent_id,
            data
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Queue torrent for search indexing
    async fn queue_for_indexing(&self, torrent_id: Uuid) -> Result<()> {
        // This would typically publish to Kafka or similar
        // For now, we'll insert into a queue table
        sqlx::query!(
            r#"
            INSERT INTO search_index_queue (id, torrent_id, status)
            VALUES ($1, $2, 'pending')
            ON CONFLICT (torrent_id) DO NOTHING
            "#,
            Uuid::new_v4(),
            torrent_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get category by ID
    async fn get_category(&self, category_id: Uuid) -> Result<crate::metadata::Category> {
        let record = sqlx::query!(
            r#"
            SELECT id, name, slug, icon, sort_order, parent_id
            FROM categories
            WHERE id = $1
            "#,
            category_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow!("Category not found"))?;

        Ok(crate::metadata::Category {
            id: record.id,
            name: record.name,
            slug: record.slug,
            icon: record.icon,
            sort_order: record.sort_order,
            parent_id: record.parent_id,
        })
    }
}

/// Axum handler for torrent upload
///
/// Expects multipart form data with:
/// - `torrent` field: .torrent file
/// - `data` field: JSON with upload metadata
pub async fn upload_handler(
    State(service): State<UploadService>,
    user_id: Uuid, // Would come from authentication middleware
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, UploadError> {
    let mut torrent_data: Option<Vec<u8>> = None;
    let mut request: Option<UploadRequest> = None;

    // Parse multipart form
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| UploadError {
            error: format!("Failed to read multipart field: {}", e),
            details: None,
        })?
    {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "torrent" => {
                let data = field.bytes().await.map_err(|e| UploadError {
                    error: format!("Failed to read torrent file: {}", e),
                    details: None,
                })?;

                torrent_data = Some(data.to_vec());
            }
            "data" => {
                let data = field.text().await.map_err(|e| UploadError {
                    error: format!("Failed to read request data: {}", e),
                    details: None,
                })?;

                request = Some(serde_json::from_str(&data).map_err(|e| UploadError {
                    error: format!("Invalid request data: {}", e),
                    details: None,
                })?);
            }
            _ => {}
        }
    }

    // Validate we have required data
    let torrent_data = torrent_data.ok_or_else(|| UploadError {
        error: "Missing torrent file".to_string(),
        details: None,
    })?;

    let request = request.ok_or_else(|| UploadError {
        error: "Missing request data".to_string(),
        details: None,
    })?;

    // Process upload
    let response = service
        .upload_torrent(user_id, torrent_data, request)
        .await
        .map_err(|e| UploadError {
            error: e.to_string(),
            details: Some(vec![format!("{:?}", e)]),
        })?;

    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upload_request_validation() {
        let mut req = UploadRequest {
            name: Some("Valid Name".to_string()),
            description: None,
            category_id: Uuid::new_v4(),
            tags: None,
            nfo_content: None,
            tmdb_id: None,
            imdb_id: None,
            tvdb_id: None,
            igdb_id: None,
            poster_url: None,
            screenshots: None,
            year: None,
            anonymous: None,
        };

        assert!(req.validate().is_ok());

        // Test invalid name (too short)
        req.name = Some("ab".to_string());
        assert!(req.validate().is_err());

        // Test invalid name (too long)
        req.name = Some("a".repeat(300));
        assert!(req.validate().is_err());
    }
}
