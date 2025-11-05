//! User profile management
//!
//! This module provides functionality for managing user profiles, including:
//! - Profile information (custom title, about me, signature)
//! - Avatar upload and management
//! - Profile visibility controls
//! - Profile view counter

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;
use validator::Validate;

/// Profile-related errors
#[derive(Debug, Error)]
pub enum ProfileError {
    #[error("Profile not found for user {0}")]
    NotFound(Uuid),

    #[error("Invalid avatar format: {0}")]
    InvalidAvatar(String),

    #[error("Avatar size exceeds limit (max {max_size} bytes)")]
    AvatarTooLarge { max_size: usize },

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Image processing error: {0}")]
    ImageProcessing(String),
}

/// User profile with customizable fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// User ID
    pub user_id: Uuid,

    /// Username
    pub username: String,

    /// Email address
    pub email: String,

    /// Custom user title (e.g., "Power User", "VIP")
    pub custom_title: Option<String>,

    /// About me text (profile description)
    pub about_me: Option<String>,

    /// User signature for forum posts
    pub signature: Option<String>,

    /// Avatar URL or path
    pub avatar_url: Option<String>,

    /// Number of profile views
    pub profile_views: i64,

    /// Profile created at timestamp
    pub created_at: DateTime<Utc>,

    /// Profile last updated at timestamp
    pub updated_at: DateTime<Utc>,
}

/// Request to update user profile
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateProfileRequest {
    /// Custom user title (max 50 characters)
    #[validate(length(max = 50))]
    pub custom_title: Option<String>,

    /// About me text (max 2000 characters)
    #[validate(length(max = 2000))]
    pub about_me: Option<String>,

    /// User signature (max 500 characters)
    #[validate(length(max = 500))]
    pub signature: Option<String>,
}

/// Avatar upload data
#[derive(Debug, Clone)]
pub struct AvatarUpload {
    /// Image data in bytes
    pub data: Vec<u8>,

    /// Content type (e.g., "image/jpeg", "image/png")
    pub content_type: String,

    /// File extension
    pub extension: String,
}

/// Profile visibility settings
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProfileVisibility {
    /// Visible to everyone
    Public,
    /// Visible to logged-in users only
    Users,
    /// Visible to nobody (private)
    Private,
}

impl Default for ProfileVisibility {
    fn default() -> Self {
        Self::Public
    }
}

/// Profile service for managing user profiles
pub struct ProfileService {
    db: PgPool,
}

impl ProfileService {
    /// Create a new profile service
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Get user profile by user ID
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID to fetch the profile for
    ///
    /// # Returns
    ///
    /// Returns the user profile or an error if not found
    pub async fn get_profile(&self, user_id: Uuid) -> Result<UserProfile, ProfileError> {
        let profile = sqlx::query_as!(
            UserProfile,
            r#"
            SELECT
                user_id,
                username,
                email,
                custom_title,
                about_me,
                signature,
                avatar_url,
                profile_views,
                created_at,
                updated_at
            FROM user_profiles
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(ProfileError::NotFound(user_id))?;

        Ok(profile)
    }

    /// Update user profile
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID to update
    /// * `request` - The profile update data
    ///
    /// # Returns
    ///
    /// Returns the updated profile or an error
    pub async fn update_profile(
        &self,
        user_id: Uuid,
        request: UpdateProfileRequest,
    ) -> Result<UserProfile, ProfileError> {
        // Validate request
        request
            .validate()
            .map_err(|e| ProfileError::Validation(e.to_string()))?;

        // Update profile
        let profile = sqlx::query_as!(
            UserProfile,
            r#"
            UPDATE user_profiles
            SET
                custom_title = COALESCE($2, custom_title),
                about_me = COALESCE($3, about_me),
                signature = COALESCE($4, signature),
                updated_at = NOW()
            WHERE user_id = $1
            RETURNING
                user_id,
                username,
                email,
                custom_title,
                about_me,
                signature,
                avatar_url,
                profile_views,
                created_at,
                updated_at
            "#,
            user_id,
            request.custom_title,
            request.about_me,
            request.signature
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(ProfileError::NotFound(user_id))?;

        Ok(profile)
    }

    /// Upload and set user avatar
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `avatar` - The avatar upload data
    ///
    /// # Returns
    ///
    /// Returns the avatar URL or an error
    pub async fn upload_avatar(
        &self,
        user_id: Uuid,
        avatar: AvatarUpload,
    ) -> Result<String, ProfileError> {
        // Validate avatar size (max 5MB)
        const MAX_AVATAR_SIZE: usize = 5 * 1024 * 1024;
        if avatar.data.len() > MAX_AVATAR_SIZE {
            return Err(ProfileError::AvatarTooLarge {
                max_size: MAX_AVATAR_SIZE,
            });
        }

        // Validate content type
        if !matches!(
            avatar.content_type.as_str(),
            "image/jpeg" | "image/png" | "image/gif" | "image/webp"
        ) {
            return Err(ProfileError::InvalidAvatar(format!(
                "Unsupported content type: {}",
                avatar.content_type
            )));
        }

        // Process and resize image
        let processed_data = self.process_avatar_image(&avatar.data)?;

        // Generate filename
        let filename = format!("avatar_{}_{}.{}", user_id, Uuid::new_v4(), avatar.extension);

        // In a real implementation, this would upload to S3 or similar storage
        // For now, we'll simulate by creating a URL
        let avatar_url = format!("/avatars/{}", filename);

        // TODO: Actual file upload to storage service would happen here
        // For example: s3_client.upload(filename, processed_data).await?;

        // Update user profile with new avatar URL
        sqlx::query!(
            r#"
            UPDATE user_profiles
            SET avatar_url = $2, updated_at = NOW()
            WHERE user_id = $1
            "#,
            user_id,
            avatar_url
        )
        .execute(&self.db)
        .await?;

        Ok(avatar_url)
    }

    /// Delete user avatar
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    pub async fn delete_avatar(&self, user_id: Uuid) -> Result<(), ProfileError> {
        sqlx::query!(
            r#"
            UPDATE user_profiles
            SET avatar_url = NULL, updated_at = NOW()
            WHERE user_id = $1
            "#,
            user_id
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Increment profile view counter
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID whose profile was viewed
    pub async fn increment_profile_views(&self, user_id: Uuid) -> Result<i64, ProfileError> {
        let result = sqlx::query!(
            r#"
            UPDATE user_profiles
            SET profile_views = profile_views + 1
            WHERE user_id = $1
            RETURNING profile_views
            "#,
            user_id
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(ProfileError::NotFound(user_id))?;

        Ok(result.profile_views)
    }

    /// Get profile view count
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    pub async fn get_profile_views(&self, user_id: Uuid) -> Result<i64, ProfileError> {
        let result = sqlx::query!(
            r#"
            SELECT profile_views
            FROM user_profiles
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(ProfileError::NotFound(user_id))?;

        Ok(result.profile_views)
    }

    /// Search profiles by username
    ///
    /// # Arguments
    ///
    /// * `query` - The search query
    /// * `limit` - Maximum number of results
    ///
    /// # Returns
    ///
    /// Returns a list of matching profiles
    pub async fn search_profiles(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<UserProfile>, ProfileError> {
        let search_pattern = format!("%{}%", query);

        let profiles = sqlx::query_as!(
            UserProfile,
            r#"
            SELECT
                user_id,
                username,
                email,
                custom_title,
                about_me,
                signature,
                avatar_url,
                profile_views,
                created_at,
                updated_at
            FROM user_profiles
            WHERE username ILIKE $1 OR custom_title ILIKE $1
            ORDER BY profile_views DESC
            LIMIT $2
            "#,
            search_pattern,
            limit
        )
        .fetch_all(&self.db)
        .await?;

        Ok(profiles)
    }

    /// Process avatar image (resize and optimize)
    fn process_avatar_image(&self, data: &[u8]) -> Result<Vec<u8>, ProfileError> {
        use image::{imageops::FilterType, ImageFormat};
        use std::io::Cursor;

        // Load image
        let img = image::load_from_memory(data)
            .map_err(|e| ProfileError::ImageProcessing(e.to_string()))?;

        // Resize to 200x200 (maintaining aspect ratio)
        let resized = img.resize(200, 200, FilterType::Lanczos3);

        // Encode as JPEG with quality 85
        let mut output = Cursor::new(Vec::new());
        resized
            .write_to(&mut output, ImageFormat::Jpeg)
            .map_err(|e| ProfileError::ImageProcessing(e.to_string()))?;

        Ok(output.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_visibility_default() {
        assert_eq!(ProfileVisibility::default(), ProfileVisibility::Public);
    }

    #[test]
    fn test_update_profile_request_validation() {
        let valid_request = UpdateProfileRequest {
            custom_title: Some("Power User".to_string()),
            about_me: Some("I love torrents".to_string()),
            signature: Some("Happy seeding!".to_string()),
        };

        assert!(valid_request.validate().is_ok());

        // Test title too long
        let invalid_request = UpdateProfileRequest {
            custom_title: Some("x".repeat(51)),
            about_me: None,
            signature: None,
        };

        assert!(invalid_request.validate().is_err());

        // Test about_me too long
        let invalid_request = UpdateProfileRequest {
            custom_title: None,
            about_me: Some("x".repeat(2001)),
            signature: None,
        };

        assert!(invalid_request.validate().is_err());

        // Test signature too long
        let invalid_request = UpdateProfileRequest {
            custom_title: None,
            about_me: None,
            signature: Some("x".repeat(501)),
        };

        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_avatar_upload_content_types() {
        let valid_types = ["image/jpeg", "image/png", "image/gif", "image/webp"];

        for content_type in valid_types {
            assert!(matches!(
                content_type,
                "image/jpeg" | "image/png" | "image/gif" | "image/webp"
            ));
        }
    }
}
