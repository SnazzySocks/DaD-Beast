//! Social follow features
//!
//! This module provides social networking features:
//! - Follow/unfollow users
//! - Followers/following lists
//! - Activity feed from followed users
//! - Notifications for followed user uploads

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

/// Follow-related errors
#[derive(Debug, Error)]
pub enum FollowError {
    #[error("User not found: {0}")]
    UserNotFound(Uuid),

    #[error("Already following user")]
    AlreadyFollowing,

    #[error("Not following user")]
    NotFollowing,

    #[error("Cannot follow yourself")]
    CannotFollowSelf,

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// User follow relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFollow {
    /// Follower user ID
    pub follower_id: Uuid,

    /// Followed user ID
    pub followed_id: Uuid,

    /// Follower username
    pub follower_username: Option<String>,

    /// Followed username
    pub followed_username: Option<String>,

    /// Whether to notify on uploads
    pub notify_on_upload: bool,

    /// Created at timestamp
    pub created_at: DateTime<Utc>,
}

/// Activity feed item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityItem {
    /// Activity ID
    pub id: Uuid,

    /// User ID who performed the activity
    pub user_id: Uuid,

    /// Username
    pub username: String,

    /// Activity type
    pub activity_type: ActivityType,

    /// Activity description
    pub description: String,

    /// Related torrent ID (for uploads)
    pub torrent_id: Option<Uuid>,

    /// Related torrent name
    pub torrent_name: Option<String>,

    /// Created at timestamp
    pub created_at: DateTime<Utc>,
}

/// Activity type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActivityType {
    /// Uploaded a new torrent
    TorrentUpload,
    /// Achieved a milestone
    Achievement,
    /// Received a badge
    Badge,
    /// Posted in forum
    ForumPost,
    /// Created a request
    Request,
    /// Filled a request
    RequestFill,
}

/// Follow service for managing user follows
pub struct FollowService {
    db: PgPool,
}

impl FollowService {
    /// Create a new follow service
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Follow a user
    ///
    /// # Arguments
    ///
    /// * `follower_id` - The follower's user ID
    /// * `followed_id` - The user to follow
    /// * `notify_on_upload` - Whether to notify on uploads
    pub async fn follow_user(
        &self,
        follower_id: Uuid,
        followed_id: Uuid,
        notify_on_upload: bool,
    ) -> Result<UserFollow, FollowError> {
        // Prevent self-following
        if follower_id == followed_id {
            return Err(FollowError::CannotFollowSelf);
        }

        // Check if users exist
        self.verify_user_exists(follower_id).await?;
        self.verify_user_exists(followed_id).await?;

        // Check if already following
        let already_following = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM user_follows
                WHERE follower_id = $1 AND followed_id = $2
            )
            "#,
            follower_id,
            followed_id
        )
        .fetch_one(&self.db)
        .await?
        .unwrap_or(false);

        if already_following {
            return Err(FollowError::AlreadyFollowing);
        }

        // Create follow relationship
        let follow = sqlx::query!(
            r#"
            INSERT INTO user_follows (follower_id, followed_id, notify_on_upload, created_at)
            VALUES ($1, $2, $3, NOW())
            RETURNING follower_id, followed_id, notify_on_upload, created_at
            "#,
            follower_id,
            followed_id,
            notify_on_upload
        )
        .fetch_one(&self.db)
        .await?;

        // Get usernames
        let usernames = sqlx::query!(
            r#"
            SELECT
                follower.username as follower_username,
                followed.username as followed_username
            FROM users follower, users followed
            WHERE follower.id = $1 AND followed.id = $2
            "#,
            follower_id,
            followed_id
        )
        .fetch_one(&self.db)
        .await?;

        Ok(UserFollow {
            follower_id: follow.follower_id,
            followed_id: follow.followed_id,
            follower_username: Some(usernames.follower_username),
            followed_username: Some(usernames.followed_username),
            notify_on_upload: follow.notify_on_upload,
            created_at: follow.created_at,
        })
    }

    /// Unfollow a user
    ///
    /// # Arguments
    ///
    /// * `follower_id` - The follower's user ID
    /// * `followed_id` - The user to unfollow
    pub async fn unfollow_user(
        &self,
        follower_id: Uuid,
        followed_id: Uuid,
    ) -> Result<(), FollowError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM user_follows
            WHERE follower_id = $1 AND followed_id = $2
            "#,
            follower_id,
            followed_id
        )
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(FollowError::NotFollowing);
        }

        Ok(())
    }

    /// Check if user A is following user B
    ///
    /// # Arguments
    ///
    /// * `follower_id` - The potential follower's user ID
    /// * `followed_id` - The potential followed user ID
    pub async fn is_following(
        &self,
        follower_id: Uuid,
        followed_id: Uuid,
    ) -> Result<bool, FollowError> {
        let is_following = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM user_follows
                WHERE follower_id = $1 AND followed_id = $2
            )
            "#,
            follower_id,
            followed_id
        )
        .fetch_one(&self.db)
        .await?
        .unwrap_or(false);

        Ok(is_following)
    }

    /// Get list of users following a specific user (followers)
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `limit` - Maximum number of results
    /// * `offset` - Offset for pagination
    pub async fn get_followers(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<UserFollow>, FollowError> {
        let followers = sqlx::query!(
            r#"
            SELECT
                uf.follower_id,
                uf.followed_id,
                uf.notify_on_upload,
                uf.created_at,
                follower.username as follower_username,
                followed.username as followed_username
            FROM user_follows uf
            JOIN users follower ON uf.follower_id = follower.id
            JOIN users followed ON uf.followed_id = followed.id
            WHERE uf.followed_id = $1
            ORDER BY uf.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            limit,
            offset
        )
        .fetch_all(&self.db)
        .await?;

        Ok(followers
            .into_iter()
            .map(|row| UserFollow {
                follower_id: row.follower_id,
                followed_id: row.followed_id,
                follower_username: Some(row.follower_username),
                followed_username: Some(row.followed_username),
                notify_on_upload: row.notify_on_upload,
                created_at: row.created_at,
            })
            .collect())
    }

    /// Get list of users that a specific user is following (following)
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `limit` - Maximum number of results
    /// * `offset` - Offset for pagination
    pub async fn get_following(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<UserFollow>, FollowError> {
        let following = sqlx::query!(
            r#"
            SELECT
                uf.follower_id,
                uf.followed_id,
                uf.notify_on_upload,
                uf.created_at,
                follower.username as follower_username,
                followed.username as followed_username
            FROM user_follows uf
            JOIN users follower ON uf.follower_id = follower.id
            JOIN users followed ON uf.followed_id = followed.id
            WHERE uf.follower_id = $1
            ORDER BY uf.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            limit,
            offset
        )
        .fetch_all(&self.db)
        .await?;

        Ok(following
            .into_iter()
            .map(|row| UserFollow {
                follower_id: row.follower_id,
                followed_id: row.followed_id,
                follower_username: Some(row.follower_username),
                followed_username: Some(row.followed_username),
                notify_on_upload: row.notify_on_upload,
                created_at: row.created_at,
            })
            .collect())
    }

    /// Get follower and following counts for a user
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    ///
    /// # Returns
    ///
    /// Returns (followers_count, following_count)
    pub async fn get_follow_counts(&self, user_id: Uuid) -> Result<(i64, i64), FollowError> {
        let counts = sqlx::query!(
            r#"
            SELECT
                (SELECT COUNT(*) FROM user_follows WHERE followed_id = $1) as "followers_count!",
                (SELECT COUNT(*) FROM user_follows WHERE follower_id = $1) as "following_count!"
            "#,
            user_id
        )
        .fetch_one(&self.db)
        .await?;

        Ok((counts.followers_count, counts.following_count))
    }

    /// Get activity feed for a user (activities from followed users)
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `limit` - Maximum number of results
    /// * `offset` - Offset for pagination
    pub async fn get_activity_feed(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ActivityItem>, FollowError> {
        let activities = sqlx::query!(
            r#"
            SELECT
                a.id,
                a.user_id,
                u.username,
                a.activity_type as "activity_type: ActivityType",
                a.description,
                a.torrent_id,
                t.name as torrent_name,
                a.created_at
            FROM activities a
            JOIN users u ON a.user_id = u.id
            LEFT JOIN torrents t ON a.torrent_id = t.id
            WHERE a.user_id IN (
                SELECT followed_id
                FROM user_follows
                WHERE follower_id = $1
            )
            ORDER BY a.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            limit,
            offset
        )
        .fetch_all(&self.db)
        .await?;

        Ok(activities
            .into_iter()
            .map(|row| ActivityItem {
                id: row.id,
                user_id: row.user_id,
                username: row.username,
                activity_type: row.activity_type,
                description: row.description,
                torrent_id: row.torrent_id,
                torrent_name: row.torrent_name,
                created_at: row.created_at,
            })
            .collect())
    }

    /// Record an activity for a user
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `activity_type` - The activity type
    /// * `description` - Activity description
    /// * `torrent_id` - Optional torrent ID
    pub async fn record_activity(
        &self,
        user_id: Uuid,
        activity_type: ActivityType,
        description: String,
        torrent_id: Option<Uuid>,
    ) -> Result<ActivityItem, FollowError> {
        let activity = sqlx::query!(
            r#"
            INSERT INTO activities (id, user_id, activity_type, description, torrent_id, created_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            RETURNING id, user_id, activity_type as "activity_type: ActivityType", description, torrent_id, created_at
            "#,
            Uuid::new_v4(),
            user_id,
            activity_type as ActivityType,
            description,
            torrent_id
        )
        .fetch_one(&self.db)
        .await?;

        // Get username and torrent name
        let details = sqlx::query!(
            r#"
            SELECT
                u.username,
                t.name as torrent_name
            FROM users u
            LEFT JOIN torrents t ON t.id = $2
            WHERE u.id = $1
            "#,
            user_id,
            torrent_id
        )
        .fetch_one(&self.db)
        .await?;

        Ok(ActivityItem {
            id: activity.id,
            user_id: activity.user_id,
            username: details.username,
            activity_type: activity.activity_type,
            description: activity.description,
            torrent_id: activity.torrent_id,
            torrent_name: details.torrent_name,
            created_at: activity.created_at,
        })
    }

    /// Get users who should be notified about an upload
    ///
    /// # Arguments
    ///
    /// * `uploader_id` - The uploader's user ID
    ///
    /// # Returns
    ///
    /// Returns list of user IDs who should be notified
    pub async fn get_upload_notification_users(
        &self,
        uploader_id: Uuid,
    ) -> Result<Vec<Uuid>, FollowError> {
        let users = sqlx::query_scalar!(
            r#"
            SELECT follower_id
            FROM user_follows
            WHERE followed_id = $1 AND notify_on_upload = true
            "#,
            uploader_id
        )
        .fetch_all(&self.db)
        .await?;

        Ok(users)
    }

    /// Update notification preference for a follow
    ///
    /// # Arguments
    ///
    /// * `follower_id` - The follower's user ID
    /// * `followed_id` - The followed user ID
    /// * `notify_on_upload` - Whether to notify on uploads
    pub async fn update_notification_preference(
        &self,
        follower_id: Uuid,
        followed_id: Uuid,
        notify_on_upload: bool,
    ) -> Result<(), FollowError> {
        let result = sqlx::query!(
            r#"
            UPDATE user_follows
            SET notify_on_upload = $3
            WHERE follower_id = $1 AND followed_id = $2
            "#,
            follower_id,
            followed_id,
            notify_on_upload
        )
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(FollowError::NotFollowing);
        }

        Ok(())
    }

    /// Get mutual follows (users who follow each other)
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    pub async fn get_mutual_follows(&self, user_id: Uuid) -> Result<Vec<Uuid>, FollowError> {
        let mutuals = sqlx::query_scalar!(
            r#"
            SELECT f1.followed_id
            FROM user_follows f1
            JOIN user_follows f2 ON f1.followed_id = f2.follower_id AND f1.follower_id = f2.followed_id
            WHERE f1.follower_id = $1
            "#,
            user_id
        )
        .fetch_all(&self.db)
        .await?;

        Ok(mutuals)
    }

    /// Helper method to verify user exists
    async fn verify_user_exists(&self, user_id: Uuid) -> Result<(), FollowError> {
        let exists = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)
            "#,
            user_id
        )
        .fetch_one(&self.db)
        .await?
        .unwrap_or(false);

        if !exists {
            return Err(FollowError::UserNotFound(user_id));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_types() {
        let types = vec![
            ActivityType::TorrentUpload,
            ActivityType::Achievement,
            ActivityType::Badge,
            ActivityType::ForumPost,
            ActivityType::Request,
            ActivityType::RequestFill,
        ];

        // Ensure all types are distinct
        for (i, t1) in types.iter().enumerate() {
            for (j, t2) in types.iter().enumerate() {
                if i == j {
                    assert_eq!(t1, t2);
                } else {
                    assert_ne!(t1, t2);
                }
            }
        }
    }

    #[test]
    fn test_user_follow_structure() {
        let follow = UserFollow {
            follower_id: Uuid::new_v4(),
            followed_id: Uuid::new_v4(),
            follower_username: Some("follower".to_string()),
            followed_username: Some("followed".to_string()),
            notify_on_upload: true,
            created_at: Utc::now(),
        };

        assert!(follow.notify_on_upload);
        assert_ne!(follow.follower_id, follow.followed_id);
    }
}
