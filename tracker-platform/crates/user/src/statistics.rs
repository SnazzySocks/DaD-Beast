//! User statistics tracking
//!
//! This module provides functionality for tracking user statistics, including:
//! - Upload/download totals
//! - Ratio calculation
//! - Seedbonus points
//! - Active seeding count
//! - Snatched torrents list
//! - Upload/download history charts
//! - Peer time tracking

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

/// Statistics-related errors
#[derive(Debug, Error)]
pub enum StatisticsError {
    #[error("Statistics not found for user {0}")]
    NotFound(Uuid),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Invalid time range: start must be before end")]
    InvalidTimeRange,
}

/// User statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStatistics {
    /// User ID
    pub user_id: Uuid,

    /// Total uploaded bytes
    pub uploaded: i64,

    /// Total downloaded bytes
    pub downloaded: i64,

    /// Upload/download ratio
    pub ratio: f64,

    /// Seedbonus points balance
    pub seedbonus: f64,

    /// Number of active seeding torrents
    pub active_seeding: i32,

    /// Number of active leeching torrents
    pub active_leeching: i32,

    /// Number of snatched (completed) torrents
    pub snatched_count: i32,

    /// Total number of uploads
    pub upload_count: i32,

    /// Average seed time per torrent (in seconds)
    pub avg_seed_time: i64,

    /// Total peer time (seeding + leeching) in seconds
    pub total_peer_time: i64,

    /// Last seen timestamp
    pub last_seen: Option<DateTime<Utc>>,

    /// Statistics created at timestamp
    pub created_at: DateTime<Utc>,

    /// Statistics last updated at timestamp
    pub updated_at: DateTime<Utc>,
}

impl UserStatistics {
    /// Calculate the ratio from uploaded and downloaded bytes
    ///
    /// Returns infinity if downloaded is 0, otherwise uploaded/downloaded
    pub fn calculate_ratio(uploaded: i64, downloaded: i64) -> f64 {
        if downloaded == 0 {
            if uploaded > 0 {
                f64::INFINITY
            } else {
                0.0
            }
        } else {
            uploaded as f64 / downloaded as f64
        }
    }
}

/// Upload/download history entry for charting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadDownloadHistory {
    /// Date of the entry
    pub date: DateTime<Utc>,

    /// Bytes uploaded on this date
    pub uploaded: i64,

    /// Bytes downloaded on this date
    pub downloaded: i64,
}

/// Peer time tracking for a torrent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerTime {
    /// User ID
    pub user_id: Uuid,

    /// Torrent ID
    pub torrent_id: Uuid,

    /// Time spent seeding (in seconds)
    pub seed_time: i64,

    /// Time spent leeching (in seconds)
    pub leech_time: i64,

    /// First seen timestamp
    pub first_seen: DateTime<Utc>,

    /// Last seen timestamp
    pub last_seen: DateTime<Utc>,
}

/// Snatched torrent entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnatchedTorrent {
    /// Torrent ID
    pub torrent_id: Uuid,

    /// Torrent name
    pub torrent_name: String,

    /// Uploaded bytes for this torrent
    pub uploaded: i64,

    /// Downloaded bytes for this torrent
    pub downloaded: i64,

    /// Time left (in seconds) when snatched
    pub time_left: i64,

    /// Seed time (in seconds)
    pub seed_time: i64,

    /// Snatched at timestamp
    pub snatched_at: DateTime<Utc>,
}

/// Statistics service for managing user statistics
pub struct StatisticsService {
    db: PgPool,
}

impl StatisticsService {
    /// Create a new statistics service
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Get user statistics by user ID
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID to fetch statistics for
    ///
    /// # Returns
    ///
    /// Returns the user statistics or an error if not found
    pub async fn get_statistics(&self, user_id: Uuid) -> Result<UserStatistics, StatisticsError> {
        let stats = sqlx::query_as!(
            UserStatistics,
            r#"
            SELECT
                user_id,
                uploaded,
                downloaded,
                ratio,
                seedbonus,
                active_seeding,
                active_leeching,
                snatched_count,
                upload_count,
                avg_seed_time,
                total_peer_time,
                last_seen,
                created_at,
                updated_at
            FROM user_statistics
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(StatisticsError::NotFound(user_id))?;

        Ok(stats)
    }

    /// Update upload amount for a user
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `amount` - The amount to add to uploaded bytes
    pub async fn add_uploaded(
        &self,
        user_id: Uuid,
        amount: i64,
    ) -> Result<UserStatistics, StatisticsError> {
        let stats = sqlx::query_as!(
            UserStatistics,
            r#"
            UPDATE user_statistics
            SET
                uploaded = uploaded + $2,
                ratio = CASE
                    WHEN downloaded = 0 THEN 0
                    ELSE (uploaded + $2)::float8 / downloaded::float8
                END,
                updated_at = NOW()
            WHERE user_id = $1
            RETURNING
                user_id,
                uploaded,
                downloaded,
                ratio,
                seedbonus,
                active_seeding,
                active_leeching,
                snatched_count,
                upload_count,
                avg_seed_time,
                total_peer_time,
                last_seen,
                created_at,
                updated_at
            "#,
            user_id,
            amount
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(StatisticsError::NotFound(user_id))?;

        Ok(stats)
    }

    /// Update download amount for a user
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `amount` - The amount to add to downloaded bytes
    pub async fn add_downloaded(
        &self,
        user_id: Uuid,
        amount: i64,
    ) -> Result<UserStatistics, StatisticsError> {
        let stats = sqlx::query_as!(
            UserStatistics,
            r#"
            UPDATE user_statistics
            SET
                downloaded = downloaded + $2,
                ratio = CASE
                    WHEN (downloaded + $2) = 0 THEN 0
                    ELSE uploaded::float8 / (downloaded + $2)::float8
                END,
                updated_at = NOW()
            WHERE user_id = $1
            RETURNING
                user_id,
                uploaded,
                downloaded,
                ratio,
                seedbonus,
                active_seeding,
                active_leeching,
                snatched_count,
                upload_count,
                avg_seed_time,
                total_peer_time,
                last_seen,
                created_at,
                updated_at
            "#,
            user_id,
            amount
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(StatisticsError::NotFound(user_id))?;

        Ok(stats)
    }

    /// Update seedbonus points for a user
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `amount` - The amount to add (can be negative)
    pub async fn add_seedbonus(
        &self,
        user_id: Uuid,
        amount: f64,
    ) -> Result<UserStatistics, StatisticsError> {
        let stats = sqlx::query_as!(
            UserStatistics,
            r#"
            UPDATE user_statistics
            SET
                seedbonus = seedbonus + $2,
                updated_at = NOW()
            WHERE user_id = $1
            RETURNING
                user_id,
                uploaded,
                downloaded,
                ratio,
                seedbonus,
                active_seeding,
                active_leeching,
                snatched_count,
                upload_count,
                avg_seed_time,
                total_peer_time,
                last_seen,
                created_at,
                updated_at
            "#,
            user_id,
            amount
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(StatisticsError::NotFound(user_id))?;

        Ok(stats)
    }

    /// Update active seeding/leeching counts
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    pub async fn update_active_counts(&self, user_id: Uuid) -> Result<(), StatisticsError> {
        sqlx::query!(
            r#"
            UPDATE user_statistics
            SET
                active_seeding = (
                    SELECT COUNT(*)
                    FROM peers
                    WHERE user_id = $1 AND seeder = true
                ),
                active_leeching = (
                    SELECT COUNT(*)
                    FROM peers
                    WHERE user_id = $1 AND seeder = false
                ),
                updated_at = NOW()
            WHERE user_id = $1
            "#,
            user_id
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Get upload/download history for charting
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `start_date` - Start date for the history
    /// * `end_date` - End date for the history
    ///
    /// # Returns
    ///
    /// Returns a list of history entries
    pub async fn get_upload_download_history(
        &self,
        user_id: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<UploadDownloadHistory>, StatisticsError> {
        if start_date >= end_date {
            return Err(StatisticsError::InvalidTimeRange);
        }

        let history = sqlx::query_as!(
            UploadDownloadHistory,
            r#"
            SELECT
                date,
                uploaded,
                downloaded
            FROM upload_download_history
            WHERE user_id = $1
                AND date >= $2
                AND date <= $3
            ORDER BY date ASC
            "#,
            user_id,
            start_date,
            end_date
        )
        .fetch_all(&self.db)
        .await?;

        Ok(history)
    }

    /// Get snatched torrents list
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `limit` - Maximum number of results
    /// * `offset` - Offset for pagination
    ///
    /// # Returns
    ///
    /// Returns a list of snatched torrents
    pub async fn get_snatched_torrents(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<SnatchedTorrent>, StatisticsError> {
        let snatched = sqlx::query_as!(
            SnatchedTorrent,
            r#"
            SELECT
                s.torrent_id,
                t.name as torrent_name,
                s.uploaded,
                s.downloaded,
                s.time_left,
                s.seed_time,
                s.snatched_at
            FROM snatched s
            JOIN torrents t ON s.torrent_id = t.id
            WHERE s.user_id = $1
            ORDER BY s.snatched_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            limit,
            offset
        )
        .fetch_all(&self.db)
        .await?;

        Ok(snatched)
    }

    /// Get peer time for a specific torrent
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `torrent_id` - The torrent ID
    ///
    /// # Returns
    ///
    /// Returns the peer time or None if not found
    pub async fn get_peer_time(
        &self,
        user_id: Uuid,
        torrent_id: Uuid,
    ) -> Result<Option<PeerTime>, StatisticsError> {
        let peer_time = sqlx::query_as!(
            PeerTime,
            r#"
            SELECT
                user_id,
                torrent_id,
                seed_time,
                leech_time,
                first_seen,
                last_seen
            FROM peer_times
            WHERE user_id = $1 AND torrent_id = $2
            "#,
            user_id,
            torrent_id
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(peer_time)
    }

    /// Update peer time tracking
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `torrent_id` - The torrent ID
    /// * `is_seeding` - Whether the user is currently seeding
    /// * `duration` - Duration in seconds to add
    pub async fn update_peer_time(
        &self,
        user_id: Uuid,
        torrent_id: Uuid,
        is_seeding: bool,
        duration: i64,
    ) -> Result<(), StatisticsError> {
        if is_seeding {
            sqlx::query!(
                r#"
                INSERT INTO peer_times (user_id, torrent_id, seed_time, leech_time, first_seen, last_seen)
                VALUES ($1, $2, $3, 0, NOW(), NOW())
                ON CONFLICT (user_id, torrent_id)
                DO UPDATE SET
                    seed_time = peer_times.seed_time + $3,
                    last_seen = NOW()
                "#,
                user_id,
                torrent_id,
                duration
            )
            .execute(&self.db)
            .await?;
        } else {
            sqlx::query!(
                r#"
                INSERT INTO peer_times (user_id, torrent_id, seed_time, leech_time, first_seen, last_seen)
                VALUES ($1, $2, 0, $3, NOW(), NOW())
                ON CONFLICT (user_id, torrent_id)
                DO UPDATE SET
                    leech_time = peer_times.leech_time + $3,
                    last_seen = NOW()
                "#,
                user_id,
                torrent_id,
                duration
            )
            .execute(&self.db)
            .await?;
        }

        Ok(())
    }

    /// Update last seen timestamp
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    pub async fn update_last_seen(&self, user_id: Uuid) -> Result<(), StatisticsError> {
        sqlx::query!(
            r#"
            UPDATE user_statistics
            SET last_seen = NOW(), updated_at = NOW()
            WHERE user_id = $1
            "#,
            user_id
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Get leaderboard by upload
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of results
    ///
    /// # Returns
    ///
    /// Returns a list of top uploaders
    pub async fn get_upload_leaderboard(
        &self,
        limit: i64,
    ) -> Result<Vec<UserStatistics>, StatisticsError> {
        let stats = sqlx::query_as!(
            UserStatistics,
            r#"
            SELECT
                user_id,
                uploaded,
                downloaded,
                ratio,
                seedbonus,
                active_seeding,
                active_leeching,
                snatched_count,
                upload_count,
                avg_seed_time,
                total_peer_time,
                last_seen,
                created_at,
                updated_at
            FROM user_statistics
            ORDER BY uploaded DESC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(&self.db)
        .await?;

        Ok(stats)
    }

    /// Get leaderboard by ratio
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of results
    ///
    /// # Returns
    ///
    /// Returns a list of users with highest ratios
    pub async fn get_ratio_leaderboard(
        &self,
        limit: i64,
    ) -> Result<Vec<UserStatistics>, StatisticsError> {
        let stats = sqlx::query_as!(
            UserStatistics,
            r#"
            SELECT
                user_id,
                uploaded,
                downloaded,
                ratio,
                seedbonus,
                active_seeding,
                active_leeching,
                snatched_count,
                upload_count,
                avg_seed_time,
                total_peer_time,
                last_seen,
                created_at,
                updated_at
            FROM user_statistics
            WHERE downloaded > 0
            ORDER BY ratio DESC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(&self.db)
        .await?;

        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_ratio() {
        // Normal ratio
        assert_eq!(UserStatistics::calculate_ratio(100, 50), 2.0);

        // Infinite ratio (no download)
        assert_eq!(UserStatistics::calculate_ratio(100, 0), f64::INFINITY);

        // Zero ratio (no upload or download)
        assert_eq!(UserStatistics::calculate_ratio(0, 0), 0.0);

        // Less than 1 ratio
        assert_eq!(UserStatistics::calculate_ratio(50, 100), 0.5);
    }

    #[test]
    fn test_ratio_precision() {
        let ratio = UserStatistics::calculate_ratio(1073741824, 536870912); // 1GB / 512MB
        assert!((ratio - 2.0).abs() < 0.0001);
    }
}
