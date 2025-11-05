//! Torrent download tracking
//!
//! This module handles:
//! - Generate download URL with passkey
//! - Track downloads per user
//! - Enforce download permissions
//! - Freeleech handling
//! - Download refunds

use anyhow::{anyhow, Result};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Download permission check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadPermission {
    /// User is allowed to download
    pub allowed: bool,

    /// Reason if not allowed
    pub reason: Option<String>,

    /// User has hit ratio watch
    pub ratio_watch: bool,

    /// Minimum ratio required
    pub min_ratio: Option<f64>,

    /// Current user ratio
    pub current_ratio: Option<f64>,
}

/// Download event for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadEvent {
    /// Event ID
    pub id: Uuid,

    /// Torrent ID
    pub torrent_id: Uuid,

    /// User ID
    pub user_id: Uuid,

    /// User's passkey (for tracker announce)
    pub passkey: String,

    /// Download timestamp
    pub downloaded_at: chrono::DateTime<chrono::Utc>,

    /// IP address
    pub ip_address: Option<String>,

    /// User agent
    pub user_agent: Option<String>,

    /// Is freeleech (no download counted)
    pub is_freeleech: bool,

    /// Refunded (removed from download count)
    pub refunded: bool,
}

/// Freeleech type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "freeleech_type")]
pub enum FreeleechType {
    /// No freeleech
    None,

    /// Partial freeleech (e.g., 50% off)
    Partial,

    /// Full freeleech (100% free)
    Full,

    /// Neutral leech (download doesn't count, upload does)
    Neutral,
}

/// Freeleech token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeleechToken {
    /// Token ID
    pub id: Uuid,

    /// User ID
    pub user_id: Uuid,

    /// Torrent ID (if specific to a torrent)
    pub torrent_id: Option<Uuid>,

    /// Freeleech type
    pub freeleech_type: FreeleechType,

    /// Discount percentage (0-100)
    pub discount_percent: i32,

    /// Expiry time
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Used flag
    pub used: bool,
}

/// Download statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadStats {
    /// Total downloads
    pub total_downloads: i64,

    /// Downloads today
    pub downloads_today: i64,

    /// Downloads this week
    pub downloads_this_week: i64,

    /// Downloads this month
    pub downloads_this_month: i64,

    /// Unique downloaders
    pub unique_downloaders: i64,
}

/// Download service
pub struct DownloadService {
    pool: PgPool,
    min_ratio: f64,
    ratio_watch_threshold: f64,
}

impl DownloadService {
    /// Create new download service
    pub fn new(pool: PgPool, min_ratio: f64, ratio_watch_threshold: f64) -> Self {
        Self {
            pool,
            min_ratio,
            ratio_watch_threshold,
        }
    }

    /// Check if user can download torrent
    pub async fn check_permission(
        &self,
        user_id: Uuid,
        torrent_id: Uuid,
    ) -> Result<DownloadPermission> {
        // Get user stats
        let user_stats = sqlx::query!(
            r#"
            SELECT
                downloaded_bytes,
                uploaded_bytes,
                CASE
                    WHEN downloaded_bytes = 0 THEN 999.99
                    ELSE uploaded_bytes::float8 / downloaded_bytes::float8
                END as ratio,
                is_banned,
                is_donor,
                can_download
            FROM user_stats
            JOIN users ON users.id = user_stats.user_id
            WHERE user_stats.user_id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        let user_stats = match user_stats {
            Some(stats) => stats,
            None => {
                return Ok(DownloadPermission {
                    allowed: false,
                    reason: Some("User not found".to_string()),
                    ratio_watch: false,
                    min_ratio: None,
                    current_ratio: None,
                });
            }
        };

        // Check if user is banned
        if user_stats.is_banned.unwrap_or(false) {
            return Ok(DownloadPermission {
                allowed: false,
                reason: Some("User is banned".to_string()),
                ratio_watch: false,
                min_ratio: None,
                current_ratio: None,
            });
        }

        // Check if user has download permission disabled
        if !user_stats.can_download.unwrap_or(true) {
            return Ok(DownloadPermission {
                allowed: false,
                reason: Some("Download permission disabled".to_string()),
                ratio_watch: false,
                min_ratio: None,
                current_ratio: None,
            });
        }

        // Get torrent info
        let torrent = sqlx::query!(
            r#"
            SELECT
                moderation_status as "status: crate::moderation::ModerationStatus",
                is_deleted
            FROM torrents
            WHERE id = $1
            "#,
            torrent_id
        )
        .fetch_optional(&self.pool)
        .await?;

        let torrent = match torrent {
            Some(t) => t,
            None => {
                return Ok(DownloadPermission {
                    allowed: false,
                    reason: Some("Torrent not found".to_string()),
                    ratio_watch: false,
                    min_ratio: None,
                    current_ratio: None,
                });
            }
        };

        // Check if torrent is approved
        if torrent.status != crate::moderation::ModerationStatus::Approved {
            return Ok(DownloadPermission {
                allowed: false,
                reason: Some("Torrent is not approved".to_string()),
                ratio_watch: false,
                min_ratio: None,
                current_ratio: None,
            });
        }

        // Check if torrent is deleted
        if torrent.is_deleted.unwrap_or(false) {
            return Ok(DownloadPermission {
                allowed: false,
                reason: Some("Torrent is deleted".to_string()),
                ratio_watch: false,
                min_ratio: None,
                current_ratio: None,
            });
        }

        let current_ratio = user_stats.ratio.unwrap_or(0.0);
        let is_donor = user_stats.is_donor.unwrap_or(false);

        // Donors bypass ratio requirements
        if is_donor {
            return Ok(DownloadPermission {
                allowed: true,
                reason: None,
                ratio_watch: false,
                min_ratio: Some(self.min_ratio),
                current_ratio: Some(current_ratio),
            });
        }

        // Check ratio requirements
        let ratio_watch = current_ratio < self.ratio_watch_threshold;

        if current_ratio < self.min_ratio {
            return Ok(DownloadPermission {
                allowed: false,
                reason: Some(format!(
                    "Insufficient ratio: {:.2} (minimum: {:.2})",
                    current_ratio, self.min_ratio
                )),
                ratio_watch,
                min_ratio: Some(self.min_ratio),
                current_ratio: Some(current_ratio),
            });
        }

        Ok(DownloadPermission {
            allowed: true,
            reason: None,
            ratio_watch,
            min_ratio: Some(self.min_ratio),
            current_ratio: Some(current_ratio),
        })
    }

    /// Generate download URL with passkey
    pub async fn generate_download_url(
        &self,
        user_id: Uuid,
        torrent_id: Uuid,
        base_url: &str,
    ) -> Result<String> {
        // Get user's passkey
        let passkey = sqlx::query!(
            r#"
            SELECT passkey
            FROM users
            WHERE id = $1
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?
        .passkey;

        // Generate URL: /download/{passkey}/{torrent_id}.torrent
        let url = format!("{}/download/{}/{}.torrent", base_url, passkey, torrent_id);

        Ok(url)
    }

    /// Record download event
    pub async fn record_download(
        &self,
        user_id: Uuid,
        torrent_id: Uuid,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<DownloadEvent> {
        // Check for active freeleech
        let freeleech = self.check_freeleech(user_id, torrent_id).await?;

        // Get user passkey
        let passkey = sqlx::query!(
            r#"
            SELECT passkey
            FROM users
            WHERE id = $1
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?
        .passkey;

        let event_id = Uuid::new_v4();

        // Insert download event
        sqlx::query!(
            r#"
            INSERT INTO torrent_downloads (
                id, torrent_id, user_id, ip_address, user_agent,
                is_freeleech, downloaded_at
            ) VALUES ($1, $2, $3, $4, $5, $6, NOW())
            "#,
            event_id,
            torrent_id,
            user_id,
            ip_address,
            user_agent,
            freeleech.freeleech_type != FreeleechType::None,
        )
        .execute(&self.pool)
        .await?;

        // Update torrent download count
        sqlx::query!(
            r#"
            UPDATE torrents
            SET times_completed = times_completed + 1
            WHERE id = $1
            "#,
            torrent_id
        )
        .execute(&self.pool)
        .await?;

        Ok(DownloadEvent {
            id: event_id,
            torrent_id,
            user_id,
            passkey,
            downloaded_at: chrono::Utc::now(),
            ip_address,
            user_agent,
            is_freeleech: freeleech.freeleech_type != FreeleechType::None,
            refunded: false,
        })
    }

    /// Check freeleech status for download
    pub async fn check_freeleech(
        &self,
        user_id: Uuid,
        torrent_id: Uuid,
    ) -> Result<FreeleechToken> {
        // Check for site-wide freeleech
        let site_freeleech = sqlx::query!(
            r#"
            SELECT value
            FROM site_settings
            WHERE key = 'global_freeleech'
            "#
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(setting) = site_freeleech {
            if setting.value == "true" {
                return Ok(FreeleechToken {
                    id: Uuid::new_v4(),
                    user_id,
                    torrent_id: None,
                    freeleech_type: FreeleechType::Full,
                    discount_percent: 100,
                    expires_at: None,
                    created_at: chrono::Utc::now(),
                    used: false,
                });
            }
        }

        // Check for torrent-specific freeleech
        let torrent_freeleech = sqlx::query!(
            r#"
            SELECT freeleech_type as "freeleech_type: FreeleechType",
                   freeleech_percent
            FROM torrents
            WHERE id = $1
            "#,
            torrent_id
        )
        .fetch_one(&self.pool)
        .await?;

        if let Some(fl_type) = torrent_freeleech.freeleech_type {
            if fl_type != FreeleechType::None {
                return Ok(FreeleechToken {
                    id: Uuid::new_v4(),
                    user_id,
                    torrent_id: Some(torrent_id),
                    freeleech_type: fl_type,
                    discount_percent: torrent_freeleech.freeleech_percent.unwrap_or(100),
                    expires_at: None,
                    created_at: chrono::Utc::now(),
                    used: false,
                });
            }
        }

        // Check for user's freeleech tokens
        let user_token = sqlx::query!(
            r#"
            SELECT
                id, torrent_id,
                freeleech_type as "freeleech_type: FreeleechType",
                discount_percent, expires_at, created_at, used
            FROM freeleech_tokens
            WHERE user_id = $1
            AND (torrent_id = $2 OR torrent_id IS NULL)
            AND (expires_at IS NULL OR expires_at > NOW())
            AND used = false
            ORDER BY discount_percent DESC
            LIMIT 1
            "#,
            user_id,
            torrent_id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(token) = user_token {
            return Ok(FreeleechToken {
                id: token.id,
                user_id,
                torrent_id: token.torrent_id,
                freeleech_type: token.freeleech_type,
                discount_percent: token.discount_percent,
                expires_at: token.expires_at,
                created_at: token.created_at,
                used: token.used,
            });
        }

        // No freeleech
        Ok(FreeleechToken {
            id: Uuid::new_v4(),
            user_id,
            torrent_id: None,
            freeleech_type: FreeleechType::None,
            discount_percent: 0,
            expires_at: None,
            created_at: chrono::Utc::now(),
            used: false,
        })
    }

    /// Refund download (remove from download count)
    pub async fn refund_download(&self, download_id: Uuid, admin_id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE torrent_downloads
            SET refunded = true, refunded_by = $2, refunded_at = NOW()
            WHERE id = $1
            "#,
            download_id,
            admin_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get download statistics for a torrent
    pub async fn get_download_stats(&self, torrent_id: Uuid) -> Result<DownloadStats> {
        let stats = sqlx::query!(
            r#"
            SELECT
                COUNT(*) as "total_downloads!",
                COUNT(*) FILTER (WHERE downloaded_at >= NOW() - INTERVAL '1 day') as "downloads_today!",
                COUNT(*) FILTER (WHERE downloaded_at >= NOW() - INTERVAL '7 days') as "downloads_this_week!",
                COUNT(*) FILTER (WHERE downloaded_at >= NOW() - INTERVAL '30 days') as "downloads_this_month!",
                COUNT(DISTINCT user_id) as "unique_downloaders!"
            FROM torrent_downloads
            WHERE torrent_id = $1
            AND refunded = false
            "#,
            torrent_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(DownloadStats {
            total_downloads: stats.total_downloads,
            downloads_today: stats.downloads_today,
            downloads_this_week: stats.downloads_this_week,
            downloads_this_month: stats.downloads_this_month,
            unique_downloaders: stats.unique_downloaders,
        })
    }

    /// Get user's download history
    pub async fn get_user_downloads(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<DownloadEvent>> {
        let records = sqlx::query!(
            r#"
            SELECT
                td.id, td.torrent_id, td.user_id,
                u.passkey,
                td.downloaded_at, td.ip_address, td.user_agent,
                td.is_freeleech, td.refunded
            FROM torrent_downloads td
            JOIN users u ON u.id = td.user_id
            WHERE td.user_id = $1
            ORDER BY td.downloaded_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records
            .into_iter()
            .map(|r| DownloadEvent {
                id: r.id,
                torrent_id: r.torrent_id,
                user_id: r.user_id,
                passkey: r.passkey,
                downloaded_at: r.downloaded_at,
                ip_address: r.ip_address,
                user_agent: r.user_agent,
                is_freeleech: r.is_freeleech,
                refunded: r.refunded,
            })
            .collect())
    }
}

/// Axum handler for download permission check
pub async fn check_permission_handler(
    State(service): State<DownloadService>,
    user_id: Uuid,
    Path(torrent_id): Path<Uuid>,
) -> Result<Json<DownloadPermission>, StatusCode> {
    service
        .check_permission(user_id, torrent_id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Axum handler for download URL generation
pub async fn download_url_handler(
    State(service): State<DownloadService>,
    user_id: Uuid,
    Path(torrent_id): Path<Uuid>,
    Query(params): Query<DownloadUrlQuery>,
) -> Result<Json<DownloadUrlResponse>, StatusCode> {
    let url = service
        .generate_download_url(user_id, torrent_id, &params.base_url)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(DownloadUrlResponse { url }))
}

#[derive(Debug, Deserialize)]
pub struct DownloadUrlQuery {
    pub base_url: String,
}

#[derive(Debug, Serialize)]
pub struct DownloadUrlResponse {
    pub url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_freeleech_type() {
        assert_eq!(FreeleechType::None as i32, 0);
        assert!(FreeleechType::Full != FreeleechType::None);
    }
}
