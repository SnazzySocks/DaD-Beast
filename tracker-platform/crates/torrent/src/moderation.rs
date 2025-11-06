//! Torrent moderation system (Unit3d pattern)
//!
//! This module implements a three-stage moderation workflow:
//! - PENDING: Newly uploaded, awaiting moderation
//! - APPROVED: Approved by moderator, visible to all users
//! - REJECTED: Rejected by moderator, hidden from users
//! - POSTPONED: Needs more information or review
//!
//! Includes auto-approval for trusted uploaders and duplicate detection.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

/// Moderation status for torrents
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "moderation_status")]
pub enum ModerationStatus {
    /// Newly uploaded, awaiting moderation
    Pending,

    /// Approved by moderator
    Approved,

    /// Rejected by moderator
    Rejected,

    /// Postponed for more information
    Postponed,
}

impl std::fmt::Display for ModerationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Approved => write!(f, "Approved"),
            Self::Rejected => write!(f, "Rejected"),
            Self::Postponed => write!(f, "Postponed"),
        }
    }
}

/// Moderation action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationAction {
    /// Action ID
    pub id: Uuid,

    /// Torrent ID
    pub torrent_id: Uuid,

    /// Moderator user ID
    pub moderator_id: Uuid,

    /// Previous status
    pub previous_status: ModerationStatus,

    /// New status
    pub new_status: ModerationStatus,

    /// Reason for action
    pub reason: Option<String>,

    /// Rejection category (if rejected)
    pub rejection_category: Option<RejectionCategory>,

    /// Timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Rejection categories for better tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "rejection_category")]
pub enum RejectionCategory {
    /// Duplicate torrent
    Duplicate,

    /// Poor quality
    LowQuality,

    /// Invalid/corrupted torrent file
    InvalidTorrent,

    /// Missing or incomplete files
    IncompleteFiles,

    /// Copyright violation (DMC)
    Copyright,

    /// Against site rules
    RuleViolation,

    /// Fake/misleading content
    Fake,

    /// Spam
    Spam,

    /// Wrong category
    WrongCategory,

    /// Other reason
    Other,
}

/// Auto-approval rules configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoApprovalRules {
    /// Minimum user uploads for auto-approval
    pub min_uploads: i32,

    /// Minimum account age in days
    pub min_account_age_days: i32,

    /// Minimum upload/download ratio
    pub min_ratio: f64,

    /// User must be trusted uploader
    pub require_trusted: bool,

    /// User must have no recent warnings
    pub require_no_warnings: bool,
}

impl Default for AutoApprovalRules {
    fn default() -> Self {
        Self {
            min_uploads: 10,
            min_account_age_days: 30,
            min_ratio: 1.0,
            require_trusted: false,
            require_no_warnings: true,
        }
    }
}

/// Duplicate detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateCheck {
    /// Is duplicate
    pub is_duplicate: bool,

    /// Matching torrent IDs
    pub matching_torrents: Vec<Uuid>,

    /// Similarity score (0.0 to 1.0)
    pub similarity_scores: Vec<f32>,
}

/// Moderation service
pub struct ModerationService {
    pool: PgPool,
}

impl ModerationService {
    /// Create new moderation service
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Check if user qualifies for auto-approval
    pub async fn check_auto_approval(
        &self,
        user_id: Uuid,
        rules: &AutoApprovalRules,
    ) -> Result<bool> {
        let user_stats = sqlx::query!(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE status = 'approved') as "upload_count!",
                EXTRACT(DAYS FROM NOW() - MIN(created_at)) as "account_age_days",
                COALESCE(uploaded_bytes::float8 / NULLIF(downloaded_bytes, 0)::float8, 0.0) as "ratio!",
                is_trusted_uploader,
                COUNT(*) FILTER (WHERE warning_expires_at > NOW()) as "active_warnings!"
            FROM users u
            LEFT JOIN torrents t ON t.uploader_id = u.id
            LEFT JOIN user_stats us ON us.user_id = u.id
            LEFT JOIN user_warnings uw ON uw.user_id = u.id
            WHERE u.id = $1
            GROUP BY u.id, us.uploaded_bytes, us.downloaded_bytes, u.is_trusted_uploader
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        match user_stats {
            Some(stats) => {
                // Check upload count
                if stats.upload_count < rules.min_uploads {
                    return Ok(false);
                }

                // Check account age
                let account_age = stats.account_age_days.unwrap_or(0.0) as i32;
                if account_age < rules.min_account_age_days {
                    return Ok(false);
                }

                // Check ratio
                if stats.ratio < rules.min_ratio {
                    return Ok(false);
                }

                // Check trusted status
                if rules.require_trusted && !stats.is_trusted_uploader.unwrap_or(false) {
                    return Ok(false);
                }

                // Check warnings
                if rules.require_no_warnings && stats.active_warnings > 0 {
                    return Ok(false);
                }

                Ok(true)
            }
            None => Ok(false),
        }
    }

    /// Detect duplicate torrents by info_hash
    pub async fn check_duplicates(&self, info_hash: &str) -> Result<DuplicateCheck> {
        let matches = sqlx::query!(
            r#"
            SELECT id, info_hash
            FROM torrents
            WHERE info_hash = $1
            AND status != 'rejected'
            "#,
            info_hash
        )
        .fetch_all(&self.pool)
        .await?;

        let is_duplicate = !matches.is_empty();
        let matching_torrents = matches.iter().map(|m| m.id).collect();
        let similarity_scores = vec![1.0; matches.len()]; // Exact match

        Ok(DuplicateCheck {
            is_duplicate,
            matching_torrents,
            similarity_scores,
        })
    }

    /// Detect similar torrents by name and size
    pub async fn check_similar_torrents(
        &self,
        name: &str,
        total_size: i64,
        size_tolerance: f64,
    ) -> Result<DuplicateCheck> {
        // Calculate size range (within tolerance percentage)
        let min_size = (total_size as f64 * (1.0 - size_tolerance)) as i64;
        let max_size = (total_size as f64 * (1.0 + size_tolerance)) as i64;

        let matches = sqlx::query!(
            r#"
            SELECT id, name, total_size,
                   similarity(name, $1) as "similarity!"
            FROM torrents
            WHERE total_size BETWEEN $2 AND $3
            AND status != 'rejected'
            AND similarity(name, $1) > 0.5
            ORDER BY similarity(name, $1) DESC
            LIMIT 10
            "#,
            name,
            min_size,
            max_size
        )
        .fetch_all(&self.pool)
        .await?;

        let is_duplicate = matches.iter().any(|m| m.similarity > 0.8);
        let matching_torrents = matches.iter().map(|m| m.id).collect();
        let similarity_scores = matches.iter().map(|m| m.similarity as f32).collect();

        Ok(DuplicateCheck {
            is_duplicate,
            matching_torrents,
            similarity_scores,
        })
    }

    /// Approve torrent
    pub async fn approve_torrent(
        &self,
        torrent_id: Uuid,
        moderator_id: Uuid,
        reason: Option<String>,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Get current status
        let current = sqlx::query!(
            r#"
            SELECT moderation_status as "status: ModerationStatus"
            FROM torrents
            WHERE id = $1
            "#,
            torrent_id
        )
        .fetch_one(&mut *tx)
        .await?;

        // Update torrent status
        sqlx::query!(
            r#"
            UPDATE torrents
            SET moderation_status = 'approved',
                approved_at = NOW(),
                approved_by = $2
            WHERE id = $1
            "#,
            torrent_id,
            moderator_id
        )
        .execute(&mut *tx)
        .await?;

        // Log moderation action
        sqlx::query!(
            r#"
            INSERT INTO moderation_actions (
                id, torrent_id, moderator_id, previous_status, new_status, reason
            ) VALUES ($1, $2, $3, $4, 'approved', $5)
            "#,
            Uuid::new_v4(),
            torrent_id,
            moderator_id,
            current.status as ModerationStatus,
            reason
        )
        .execute(&mut *tx)
        .await?;

        // Award upload credit to uploader
        sqlx::query!(
            r#"
            UPDATE user_stats
            SET uploads_approved = uploads_approved + 1
            WHERE user_id = (SELECT uploader_id FROM torrents WHERE id = $1)
            "#,
            torrent_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    /// Reject torrent
    pub async fn reject_torrent(
        &self,
        torrent_id: Uuid,
        moderator_id: Uuid,
        reason: String,
        category: RejectionCategory,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Get current status
        let current = sqlx::query!(
            r#"
            SELECT moderation_status as "status: ModerationStatus"
            FROM torrents
            WHERE id = $1
            "#,
            torrent_id
        )
        .fetch_one(&mut *tx)
        .await?;

        // Update torrent status
        sqlx::query!(
            r#"
            UPDATE torrents
            SET moderation_status = 'rejected',
                rejected_at = NOW(),
                rejected_by = $2,
                rejection_reason = $3
            WHERE id = $1
            "#,
            torrent_id,
            moderator_id,
            reason
        )
        .execute(&mut *tx)
        .await?;

        // Log moderation action
        sqlx::query!(
            r#"
            INSERT INTO moderation_actions (
                id, torrent_id, moderator_id, previous_status, new_status,
                reason, rejection_category
            ) VALUES ($1, $2, $3, $4, 'rejected', $5, $6)
            "#,
            Uuid::new_v4(),
            torrent_id,
            moderator_id,
            current.status as ModerationStatus,
            reason,
            category as RejectionCategory
        )
        .execute(&mut *tx)
        .await?;

        // Update uploader stats
        sqlx::query!(
            r#"
            UPDATE user_stats
            SET uploads_rejected = uploads_rejected + 1
            WHERE user_id = (SELECT uploader_id FROM torrents WHERE id = $1)
            "#,
            torrent_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    /// Postpone torrent (needs more info)
    pub async fn postpone_torrent(
        &self,
        torrent_id: Uuid,
        moderator_id: Uuid,
        reason: String,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Get current status
        let current = sqlx::query!(
            r#"
            SELECT moderation_status as "status: ModerationStatus"
            FROM torrents
            WHERE id = $1
            "#,
            torrent_id
        )
        .fetch_one(&mut *tx)
        .await?;

        // Update torrent status
        sqlx::query!(
            r#"
            UPDATE torrents
            SET moderation_status = 'postponed',
                postponed_at = NOW(),
                postponed_by = $2,
                postpone_reason = $3
            WHERE id = $1
            "#,
            torrent_id,
            moderator_id,
            reason
        )
        .execute(&mut *tx)
        .await?;

        // Log moderation action
        sqlx::query!(
            r#"
            INSERT INTO moderation_actions (
                id, torrent_id, moderator_id, previous_status, new_status, reason
            ) VALUES ($1, $2, $3, $4, 'postponed', $5)
            "#,
            Uuid::new_v4(),
            torrent_id,
            moderator_id,
            current.status as ModerationStatus,
            reason
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    /// Get moderation history for a torrent
    pub async fn get_moderation_history(&self, torrent_id: Uuid) -> Result<Vec<ModerationAction>> {
        let records = sqlx::query!(
            r#"
            SELECT
                id, torrent_id, moderator_id,
                previous_status as "previous_status: ModerationStatus",
                new_status as "new_status: ModerationStatus",
                reason,
                rejection_category as "rejection_category: RejectionCategory",
                created_at
            FROM moderation_actions
            WHERE torrent_id = $1
            ORDER BY created_at DESC
            "#,
            torrent_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records
            .into_iter()
            .map(|r| ModerationAction {
                id: r.id,
                torrent_id: r.torrent_id,
                moderator_id: r.moderator_id,
                previous_status: r.previous_status,
                new_status: r.new_status,
                reason: r.reason,
                rejection_category: r.rejection_category,
                created_at: r.created_at,
            })
            .collect())
    }

    /// Get pending torrents for moderation
    pub async fn get_pending_torrents(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PendingTorrent>> {
        let records = sqlx::query!(
            r#"
            SELECT
                t.id, t.name, t.info_hash, t.total_size,
                t.uploader_id, u.username as uploader_name,
                t.created_at, t.category_id, c.name as category_name
            FROM torrents t
            JOIN users u ON u.id = t.uploader_id
            JOIN categories c ON c.id = t.category_id
            WHERE t.moderation_status = 'pending'
            ORDER BY t.created_at ASC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records
            .into_iter()
            .map(|r| PendingTorrent {
                id: r.id,
                name: r.name,
                info_hash: r.info_hash,
                total_size: r.total_size,
                uploader_id: r.uploader_id,
                uploader_name: r.uploader_name,
                category_id: r.category_id,
                category_name: r.category_name,
                created_at: r.created_at,
            })
            .collect())
    }

    /// Get moderation statistics
    pub async fn get_moderation_stats(&self) -> Result<ModerationStats> {
        let stats = sqlx::query!(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE moderation_status = 'pending') as "pending!",
                COUNT(*) FILTER (WHERE moderation_status = 'approved') as "approved!",
                COUNT(*) FILTER (WHERE moderation_status = 'rejected') as "rejected!",
                COUNT(*) FILTER (WHERE moderation_status = 'postponed') as "postponed!",
                COUNT(*) FILTER (
                    WHERE moderation_status = 'pending'
                    AND created_at < NOW() - INTERVAL '24 hours'
                ) as "pending_over_24h!",
                AVG(
                    EXTRACT(EPOCH FROM (approved_at - created_at))
                ) FILTER (WHERE approved_at IS NOT NULL) as approval_time_avg
            FROM torrents
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(ModerationStats {
            pending: stats.pending,
            approved: stats.approved,
            rejected: stats.rejected,
            postponed: stats.postponed,
            pending_over_24h: stats.pending_over_24h,
            avg_approval_time_seconds: stats.approval_time_avg.map(|t| t as i64),
        })
    }
}

/// Pending torrent information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingTorrent {
    pub id: Uuid,
    pub name: String,
    pub info_hash: String,
    pub total_size: i64,
    pub uploader_id: Uuid,
    pub uploader_name: String,
    pub category_id: Uuid,
    pub category_name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Moderation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationStats {
    pub pending: i64,
    pub approved: i64,
    pub rejected: i64,
    pub postponed: i64,
    pub pending_over_24h: i64,
    pub avg_approval_time_seconds: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moderation_status_display() {
        assert_eq!(ModerationStatus::Pending.to_string(), "Pending");
        assert_eq!(ModerationStatus::Approved.to_string(), "Approved");
        assert_eq!(ModerationStatus::Rejected.to_string(), "Rejected");
        assert_eq!(ModerationStatus::Postponed.to_string(), "Postponed");
    }

    #[test]
    fn test_auto_approval_rules_default() {
        let rules = AutoApprovalRules::default();
        assert_eq!(rules.min_uploads, 10);
        assert_eq!(rules.min_account_age_days, 30);
        assert_eq!(rules.min_ratio, 1.0);
        assert!(!rules.require_trusted);
        assert!(rules.require_no_warnings);
    }
}
