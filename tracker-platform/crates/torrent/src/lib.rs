//! Torrent Management Service
//!
//! This crate provides comprehensive torrent management functionality for the unified tracker platform.
//!
//! # Features
//!
//! - **Torrent Upload**: Parse and validate .torrent files with bencode support
//! - **Metadata Management**: Rich metadata including quality indicators, external IDs, and tags
//! - **Moderation System**: Three-stage workflow (PENDING → APPROVED/REJECTED/POSTPONED)
//! - **File Management**: File validation, sanitization, and media type detection
//! - **Download Tracking**: Passkey-based downloads with freeleech support
//! - **Search Integration**: Meilisearch indexing for fast torrent discovery
//! - **Request/Bounty System**: User requests with pooled bounties
//!
//! # Architecture
//!
//! The crate is organized into modules:
//!
//! - `bencode`: BitTorrent bencode parsing and info_hash calculation
//! - `files`: File list management and validation
//! - `metadata`: Torrent metadata and quality information
//! - `moderation`: Three-stage moderation workflow
//! - `upload`: Torrent upload handler
//! - `download`: Download tracking and permission checks
//! - `search`: Meilisearch integration
//! - `requests`: Request/bounty system
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use torrent::{TorrentService, TorrentConfig};
//! use sqlx::PgPool;
//!
//! # async fn example(pool: PgPool) -> anyhow::Result<()> {
//! // Create service with configuration
//! let config = TorrentConfig::default();
//! let service = TorrentService::new(pool, config).await?;
//!
//! // Upload a torrent
//! let torrent_data = std::fs::read("example.torrent")?;
//! let upload_request = torrent::upload::UploadRequest {
//!     name: Some("Example Torrent".to_string()),
//!     description: Some("A great torrent".to_string()),
//!     category_id: uuid::Uuid::new_v4(),
//!     tags: Some(vec!["example".to_string()]),
//!     nfo_content: None,
//!     tmdb_id: None,
//!     imdb_id: None,
//!     tvdb_id: None,
//!     igdb_id: None,
//!     poster_url: None,
//!     screenshots: None,
//!     year: Some(2024),
//!     anonymous: None,
//! };
//!
//! let user_id = uuid::Uuid::new_v4();
//! let response = service.upload_torrent(user_id, torrent_data, upload_request).await?;
//! println!("Uploaded torrent: {}", response.torrent_id);
//! # Ok(())
//! # }
//! ```

pub mod bencode;
pub mod download;
pub mod files;
pub mod metadata;
pub mod moderation;
pub mod requests;
pub mod search;
pub mod upload;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

// Re-export commonly used types
pub use bencode::{Torrent, TorrentInfo};
pub use download::{DownloadService, FreeleechType};
pub use files::{FileType, MediaType as FileMediaType, TorrentFileInfo};
pub use metadata::{MediaType, QualityInfo, TorrentMetadata};
pub use moderation::{ModerationService, ModerationStatus};
pub use requests::{RequestService, RequestStatus, TorrentRequest};
pub use search::SearchService;
pub use upload::{UploadRequest, UploadResponse, UploadService};

/// Torrent service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentConfig {
    /// Minimum ratio required for downloads
    pub min_ratio: f64,

    /// Ratio watch threshold
    pub ratio_watch_threshold: f64,

    /// Auto-approval rules
    pub auto_approval: moderation::AutoApprovalRules,

    /// Request/bounty settings
    pub request_min_bounty: i64,
    pub request_max_bounty_per_user: i64,
    pub request_expiry_days: i64,

    /// Search settings
    pub meilisearch_url: String,
    pub meilisearch_api_key: String,
    pub meilisearch_index: String,
}

impl Default for TorrentConfig {
    fn default() -> Self {
        Self {
            min_ratio: 0.5,
            ratio_watch_threshold: 0.75,
            auto_approval: moderation::AutoApprovalRules::default(),
            request_min_bounty: 100,
            request_max_bounty_per_user: 100000,
            request_expiry_days: 90,
            meilisearch_url: "http://localhost:7700".to_string(),
            meilisearch_api_key: "masterKey".to_string(),
            meilisearch_index: "torrents".to_string(),
        }
    }
}

/// Comprehensive torrent management service
///
/// This is the main entry point for torrent operations, providing a unified
/// interface to all torrent-related functionality.
pub struct TorrentService {
    pool: PgPool,
    config: TorrentConfig,
    upload: UploadService,
    download: DownloadService,
    moderation: ModerationService,
    search: SearchService,
    requests: RequestService,
}

impl TorrentService {
    /// Create a new torrent service
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection pool
    /// * `config` - Service configuration
    pub async fn new(pool: PgPool, config: TorrentConfig) -> Result<Self> {
        let upload = UploadService::new(pool.clone(), config.auto_approval.clone());
        let download = DownloadService::new(
            pool.clone(),
            config.min_ratio,
            config.ratio_watch_threshold,
        );
        let moderation = ModerationService::new(pool.clone());
        let search = SearchService::new(
            pool.clone(),
            &config.meilisearch_url,
            &config.meilisearch_api_key,
            config.meilisearch_index.clone(),
        );
        let requests = RequestService::new(
            pool.clone(),
            config.request_min_bounty,
            config.request_max_bounty_per_user,
            config.request_expiry_days,
        );

        Ok(Self {
            pool,
            config,
            upload,
            download,
            moderation,
            search,
            requests,
        })
    }

    /// Get upload service
    pub fn upload(&self) -> &UploadService {
        &self.upload
    }

    /// Get download service
    pub fn download(&self) -> &DownloadService {
        &self.download
    }

    /// Get moderation service
    pub fn moderation(&self) -> &ModerationService {
        &self.moderation
    }

    /// Get search service
    pub fn search(&self) -> &SearchService {
        &self.search
    }

    /// Get request service
    pub fn requests(&self) -> &RequestService {
        &self.requests
    }

    /// Get database pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Get configuration
    pub fn config(&self) -> &TorrentConfig {
        &self.config
    }

    /// Upload a torrent (convenience method)
    pub async fn upload_torrent(
        &self,
        user_id: uuid::Uuid,
        torrent_data: Vec<u8>,
        request: UploadRequest,
    ) -> Result<UploadResponse> {
        self.upload.upload_torrent(user_id, torrent_data, request).await
    }

    /// Check download permission (convenience method)
    pub async fn check_download_permission(
        &self,
        user_id: uuid::Uuid,
        torrent_id: uuid::Uuid,
    ) -> Result<download::DownloadPermission> {
        self.download.check_permission(user_id, torrent_id).await
    }

    /// Approve torrent (convenience method)
    pub async fn approve_torrent(
        &self,
        torrent_id: uuid::Uuid,
        moderator_id: uuid::Uuid,
        reason: Option<String>,
    ) -> Result<()> {
        self.moderation.approve_torrent(torrent_id, moderator_id, reason).await
    }

    /// Initialize search index (convenience method)
    pub async fn initialize_search(&self) -> Result<()> {
        self.search.initialize_index().await
    }

    /// Health check
    ///
    /// Verifies that all services are operational
    pub async fn health_check(&self) -> Result<HealthStatus> {
        // Check database
        let db_ok = sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .is_ok();

        // Check search (basic ping)
        let search_ok = self.search.get_index().await.is_ok();

        Ok(HealthStatus {
            database: db_ok,
            search: search_ok,
            overall: db_ok && search_ok,
        })
    }
}

/// Health status for service monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Database connection status
    pub database: bool,

    /// Search service status
    pub search: bool,

    /// Overall health
    pub overall: bool,
}

/// Error types for the torrent service
#[derive(Debug, thiserror::Error)]
pub enum TorrentError {
    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Invalid torrent file
    #[error("Invalid torrent file: {0}")]
    InvalidTorrent(String),

    /// Duplicate torrent
    #[error("Duplicate torrent: {0}")]
    Duplicate(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Search error
    #[error("Search error: {0}")]
    Search(String),

    /// General error
    #[error("Error: {0}")]
    General(String),
}

impl From<anyhow::Error> for TorrentError {
    fn from(err: anyhow::Error) -> Self {
        TorrentError::General(err.to_string())
    }
}

/// Result type for torrent operations
pub type TorrentResult<T> = std::result::Result<T, TorrentError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = TorrentConfig::default();
        assert_eq!(config.min_ratio, 0.5);
        assert_eq!(config.ratio_watch_threshold, 0.75);
    }

    #[test]
    fn test_health_status() {
        let health = HealthStatus {
            database: true,
            search: true,
            overall: true,
        };
        assert!(health.overall);
    }
}
