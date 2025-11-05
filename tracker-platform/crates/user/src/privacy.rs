//! Privacy controls (Gazelle paranoia system)
//!
//! This module implements granular privacy settings inspired by Gazelle's paranoia system.
//! Users can control what information is visible to others:
//! - Upload/download stats and ratio
//! - Last seen timestamp
//! - Snatched torrent list
//! - Active seeding/leeching torrents
//! - Profile visibility
//!
//! Privacy settings include presets (Public, Normal, Paranoid) and custom configurations.
//! Staff members can override privacy settings when necessary.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

/// Privacy-related errors
#[derive(Debug, Error)]
pub enum PrivacyError {
    #[error("Privacy settings not found for user {0}")]
    NotFound(Uuid),

    #[error("Access denied due to privacy settings")]
    AccessDenied,

    #[error("User not found: {0}")]
    UserNotFound(Uuid),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Privacy level presets
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyLevel {
    /// Everything visible to everyone
    Public,
    /// Normal privacy (reasonable defaults)
    Normal,
    /// Maximum privacy (paranoid mode)
    Paranoid,
    /// Custom settings
    Custom,
}

/// Granular privacy settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettings {
    /// User ID
    pub user_id: Uuid,

    /// Privacy level (public, normal, paranoid, custom)
    pub level: PrivacyLevel,

    /// Hide uploaded amount
    pub hide_uploaded: bool,

    /// Hide downloaded amount
    pub hide_downloaded: bool,

    /// Hide ratio
    pub hide_ratio: bool,

    /// Hide seedbonus points
    pub hide_seedbonus: bool,

    /// Hide last seen timestamp
    pub hide_last_seen: bool,

    /// Hide snatched torrent list
    pub hide_snatched: bool,

    /// Hide currently seeding torrents
    pub hide_seeding: bool,

    /// Hide currently leeching torrents
    pub hide_leeching: bool,

    /// Hide upload count
    pub hide_upload_count: bool,

    /// Hide profile from non-logged-in users (guests)
    pub hide_from_guests: bool,

    /// Hide entire profile from all users (except staff)
    pub hide_profile: bool,

    /// Hide invite tree (who invited this user)
    pub hide_invite_tree: bool,

    /// Hide followers list
    pub hide_followers: bool,

    /// Hide following list
    pub hide_following: bool,

    /// Created at timestamp
    pub created_at: DateTime<Utc>,

    /// Updated at timestamp
    pub updated_at: DateTime<Utc>,
}

impl PrivacySettings {
    /// Create privacy settings from a preset level
    pub fn from_level(level: PrivacyLevel) -> Self {
        let now = Utc::now();

        match level {
            PrivacyLevel::Public => Self {
                user_id: Uuid::nil(), // Will be set when saving
                level: PrivacyLevel::Public,
                hide_uploaded: false,
                hide_downloaded: false,
                hide_ratio: false,
                hide_seedbonus: false,
                hide_last_seen: false,
                hide_snatched: false,
                hide_seeding: false,
                hide_leeching: false,
                hide_upload_count: false,
                hide_from_guests: false,
                hide_profile: false,
                hide_invite_tree: false,
                hide_followers: false,
                hide_following: false,
                created_at: now,
                updated_at: now,
            },
            PrivacyLevel::Normal => Self {
                user_id: Uuid::nil(),
                level: PrivacyLevel::Normal,
                hide_uploaded: false,
                hide_downloaded: false,
                hide_ratio: false,
                hide_seedbonus: false,
                hide_last_seen: false,
                hide_snatched: true, // Hide snatched by default
                hide_seeding: false,
                hide_leeching: true, // Hide leeching by default
                hide_upload_count: false,
                hide_from_guests: true, // Hide from guests by default
                hide_profile: false,
                hide_invite_tree: false,
                hide_followers: false,
                hide_following: false,
                created_at: now,
                updated_at: now,
            },
            PrivacyLevel::Paranoid => Self {
                user_id: Uuid::nil(),
                level: PrivacyLevel::Paranoid,
                hide_uploaded: true,
                hide_downloaded: true,
                hide_ratio: true,
                hide_seedbonus: true,
                hide_last_seen: true,
                hide_snatched: true,
                hide_seeding: true,
                hide_leeching: true,
                hide_upload_count: true,
                hide_from_guests: true,
                hide_profile: true,
                hide_invite_tree: true,
                hide_followers: true,
                hide_following: true,
                created_at: now,
                updated_at: now,
            },
            PrivacyLevel::Custom => Self::from_level(PrivacyLevel::Normal),
        }
    }
}

/// Privacy service for managing user privacy settings
pub struct PrivacyService {
    db: PgPool,
}

impl PrivacyService {
    /// Create a new privacy service
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Get privacy settings for a user
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    pub async fn get_privacy_settings(
        &self,
        user_id: Uuid,
    ) -> Result<PrivacySettings, PrivacyError> {
        let settings = sqlx::query_as!(
            PrivacySettings,
            r#"
            SELECT
                user_id,
                level as "level: PrivacyLevel",
                hide_uploaded,
                hide_downloaded,
                hide_ratio,
                hide_seedbonus,
                hide_last_seen,
                hide_snatched,
                hide_seeding,
                hide_leeching,
                hide_upload_count,
                hide_from_guests,
                hide_profile,
                hide_invite_tree,
                hide_followers,
                hide_following,
                created_at,
                updated_at
            FROM privacy_settings
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(PrivacyError::NotFound(user_id))?;

        Ok(settings)
    }

    /// Update privacy settings for a user
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `settings` - The new privacy settings
    pub async fn update_privacy_settings(
        &self,
        user_id: Uuid,
        mut settings: PrivacySettings,
    ) -> Result<PrivacySettings, PrivacyError> {
        settings.user_id = user_id;
        settings.updated_at = Utc::now();

        let updated = sqlx::query_as!(
            PrivacySettings,
            r#"
            INSERT INTO privacy_settings (
                user_id,
                level,
                hide_uploaded,
                hide_downloaded,
                hide_ratio,
                hide_seedbonus,
                hide_last_seen,
                hide_snatched,
                hide_seeding,
                hide_leeching,
                hide_upload_count,
                hide_from_guests,
                hide_profile,
                hide_invite_tree,
                hide_followers,
                hide_following,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, NOW(), NOW())
            ON CONFLICT (user_id)
            DO UPDATE SET
                level = $2,
                hide_uploaded = $3,
                hide_downloaded = $4,
                hide_ratio = $5,
                hide_seedbonus = $6,
                hide_last_seen = $7,
                hide_snatched = $8,
                hide_seeding = $9,
                hide_leeching = $10,
                hide_upload_count = $11,
                hide_from_guests = $12,
                hide_profile = $13,
                hide_invite_tree = $14,
                hide_followers = $15,
                hide_following = $16,
                updated_at = NOW()
            RETURNING
                user_id,
                level as "level: PrivacyLevel",
                hide_uploaded,
                hide_downloaded,
                hide_ratio,
                hide_seedbonus,
                hide_last_seen,
                hide_snatched,
                hide_seeding,
                hide_leeching,
                hide_upload_count,
                hide_from_guests,
                hide_profile,
                hide_invite_tree,
                hide_followers,
                hide_following,
                created_at,
                updated_at
            "#,
            user_id,
            settings.level as PrivacyLevel,
            settings.hide_uploaded,
            settings.hide_downloaded,
            settings.hide_ratio,
            settings.hide_seedbonus,
            settings.hide_last_seen,
            settings.hide_snatched,
            settings.hide_seeding,
            settings.hide_leeching,
            settings.hide_upload_count,
            settings.hide_from_guests,
            settings.hide_profile,
            settings.hide_invite_tree,
            settings.hide_followers,
            settings.hide_following
        )
        .fetch_one(&self.db)
        .await?;

        Ok(updated)
    }

    /// Check if a viewer can see a user's uploaded amount
    ///
    /// # Arguments
    ///
    /// * `user_id` - The profile owner's user ID
    /// * `viewer_id` - The viewer's user ID (None if not logged in)
    /// * `is_staff` - Whether the viewer is staff
    pub async fn can_view_uploaded(
        &self,
        user_id: Uuid,
        viewer_id: Option<Uuid>,
        is_staff: bool,
    ) -> Result<bool, PrivacyError> {
        if is_staff || viewer_id == Some(user_id) {
            return Ok(true);
        }

        let settings = self.get_privacy_settings(user_id).await?;
        Ok(!settings.hide_uploaded)
    }

    /// Check if a viewer can see a user's downloaded amount
    pub async fn can_view_downloaded(
        &self,
        user_id: Uuid,
        viewer_id: Option<Uuid>,
        is_staff: bool,
    ) -> Result<bool, PrivacyError> {
        if is_staff || viewer_id == Some(user_id) {
            return Ok(true);
        }

        let settings = self.get_privacy_settings(user_id).await?;
        Ok(!settings.hide_downloaded)
    }

    /// Check if a viewer can see a user's ratio
    pub async fn can_view_ratio(
        &self,
        user_id: Uuid,
        viewer_id: Option<Uuid>,
    ) -> Result<bool, PrivacyError> {
        if viewer_id == Some(user_id) {
            return Ok(true);
        }

        // Check if viewer is staff
        let is_staff = if let Some(viewer_id) = viewer_id {
            self.is_staff(viewer_id).await?
        } else {
            false
        };

        if is_staff {
            return Ok(true);
        }

        let settings = self.get_privacy_settings(user_id).await?;
        Ok(!settings.hide_ratio)
    }

    /// Check if a viewer can see a user's last seen timestamp
    pub async fn can_view_last_seen(
        &self,
        user_id: Uuid,
        viewer_id: Option<Uuid>,
        is_staff: bool,
    ) -> Result<bool, PrivacyError> {
        if is_staff || viewer_id == Some(user_id) {
            return Ok(true);
        }

        let settings = self.get_privacy_settings(user_id).await?;
        Ok(!settings.hide_last_seen)
    }

    /// Check if a viewer can see a user's snatched torrents
    pub async fn can_view_snatched(
        &self,
        user_id: Uuid,
        viewer_id: Option<Uuid>,
        is_staff: bool,
    ) -> Result<bool, PrivacyError> {
        if is_staff || viewer_id == Some(user_id) {
            return Ok(true);
        }

        let settings = self.get_privacy_settings(user_id).await?;
        Ok(!settings.hide_snatched)
    }

    /// Check if a viewer can see a user's currently seeding torrents
    pub async fn can_view_seeding(
        &self,
        user_id: Uuid,
        viewer_id: Option<Uuid>,
        is_staff: bool,
    ) -> Result<bool, PrivacyError> {
        if is_staff || viewer_id == Some(user_id) {
            return Ok(true);
        }

        let settings = self.get_privacy_settings(user_id).await?;
        Ok(!settings.hide_seeding)
    }

    /// Check if a viewer can see a user's currently leeching torrents
    pub async fn can_view_leeching(
        &self,
        user_id: Uuid,
        viewer_id: Option<Uuid>,
        is_staff: bool,
    ) -> Result<bool, PrivacyError> {
        if is_staff || viewer_id == Some(user_id) {
            return Ok(true);
        }

        let settings = self.get_privacy_settings(user_id).await?;
        Ok(!settings.hide_leeching)
    }

    /// Check if a viewer can see a user's profile
    pub async fn can_view_profile(
        &self,
        user_id: Uuid,
        viewer_id: Option<Uuid>,
        is_staff: bool,
    ) -> Result<bool, PrivacyError> {
        if is_staff || viewer_id == Some(user_id) {
            return Ok(true);
        }

        let settings = self.get_privacy_settings(user_id).await?;

        // Check if hidden from guests
        if viewer_id.is_none() && settings.hide_from_guests {
            return Ok(false);
        }

        // Check if profile is completely hidden
        Ok(!settings.hide_profile)
    }

    /// Check if a viewer can see a user's invite tree
    pub async fn can_view_invite_tree(
        &self,
        user_id: Uuid,
        viewer_id: Option<Uuid>,
        is_staff: bool,
    ) -> Result<bool, PrivacyError> {
        if is_staff || viewer_id == Some(user_id) {
            return Ok(true);
        }

        let settings = self.get_privacy_settings(user_id).await?;
        Ok(!settings.hide_invite_tree)
    }

    /// Check if a user is staff (helper method)
    async fn is_staff(&self, user_id: Uuid) -> Result<bool, PrivacyError> {
        let is_staff = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM users
                WHERE id = $1 AND (role = 'admin' OR role = 'moderator')
            )
            "#,
            user_id
        )
        .fetch_one(&self.db)
        .await?
        .unwrap_or(false);

        Ok(is_staff)
    }

    /// Apply privacy filters to user statistics
    ///
    /// Returns filtered statistics based on privacy settings
    pub async fn filter_statistics(
        &self,
        user_id: Uuid,
        viewer_id: Option<Uuid>,
        is_staff: bool,
        stats: crate::statistics::UserStatistics,
    ) -> Result<crate::statistics::UserStatistics, PrivacyError> {
        if is_staff || viewer_id == Some(user_id) {
            return Ok(stats);
        }

        let settings = self.get_privacy_settings(user_id).await?;

        let mut filtered = stats;

        if settings.hide_uploaded {
            filtered.uploaded = 0;
        }
        if settings.hide_downloaded {
            filtered.downloaded = 0;
        }
        if settings.hide_ratio {
            filtered.ratio = 0.0;
        }
        if settings.hide_seedbonus {
            filtered.seedbonus = 0.0;
        }
        if settings.hide_last_seen {
            filtered.last_seen = None;
        }
        if settings.hide_seeding {
            filtered.active_seeding = 0;
        }
        if settings.hide_leeching {
            filtered.active_leeching = 0;
        }
        if settings.hide_upload_count {
            filtered.upload_count = 0;
        }

        Ok(filtered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privacy_level_public() {
        let settings = PrivacySettings::from_level(PrivacyLevel::Public);

        assert_eq!(settings.level, PrivacyLevel::Public);
        assert!(!settings.hide_uploaded);
        assert!(!settings.hide_downloaded);
        assert!(!settings.hide_ratio);
        assert!(!settings.hide_last_seen);
        assert!(!settings.hide_snatched);
        assert!(!settings.hide_profile);
    }

    #[test]
    fn test_privacy_level_normal() {
        let settings = PrivacySettings::from_level(PrivacyLevel::Normal);

        assert_eq!(settings.level, PrivacyLevel::Normal);
        assert!(!settings.hide_uploaded);
        assert!(!settings.hide_ratio);
        assert!(settings.hide_snatched); // Hidden by default
        assert!(settings.hide_leeching); // Hidden by default
        assert!(settings.hide_from_guests); // Hidden from guests by default
        assert!(!settings.hide_profile);
    }

    #[test]
    fn test_privacy_level_paranoid() {
        let settings = PrivacySettings::from_level(PrivacyLevel::Paranoid);

        assert_eq!(settings.level, PrivacyLevel::Paranoid);
        assert!(settings.hide_uploaded);
        assert!(settings.hide_downloaded);
        assert!(settings.hide_ratio);
        assert!(settings.hide_seedbonus);
        assert!(settings.hide_last_seen);
        assert!(settings.hide_snatched);
        assert!(settings.hide_seeding);
        assert!(settings.hide_leeching);
        assert!(settings.hide_profile);
        assert!(settings.hide_from_guests);
    }

    #[test]
    fn test_privacy_levels() {
        let levels = vec![
            PrivacyLevel::Public,
            PrivacyLevel::Normal,
            PrivacyLevel::Paranoid,
            PrivacyLevel::Custom,
        ];

        // Ensure all levels are distinct
        for (i, l1) in levels.iter().enumerate() {
            for (j, l2) in levels.iter().enumerate() {
                if i == j {
                    assert_eq!(l1, l2);
                } else {
                    assert_ne!(l1, l2);
                }
            }
        }
    }
}
