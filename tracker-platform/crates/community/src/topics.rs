//! Discussion Topics
//!
//! Manages forum topics (threads) including creation, editing, moderation,
//! and subscription features.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;
use validator::Validate;

/// Topic-related errors
#[derive(Debug, Error)]
pub enum TopicError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Topic not found: {0}")]
    NotFound(Uuid),

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Topic is locked")]
    TopicLocked,

    #[error("Forum not found: {0}")]
    ForumNotFound(Uuid),

    #[error("Validation error: {0}")]
    Validation(#[from] validator::ValidationErrors),

    #[error("Poll already exists for this topic")]
    PollAlreadyExists,
}

pub type Result<T> = std::result::Result<T, TopicError>;

/// Topic status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "topic_status", rename_all = "lowercase")]
pub enum TopicStatus {
    Open,
    Locked,
    Sticky,
    Announcement,
}

/// Discussion topic
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Topic {
    pub id: Uuid,
    pub forum_id: Uuid,
    pub user_id: Uuid,
    pub title: String,

    // Status
    pub status: TopicStatus,
    pub is_pinned: bool,
    pub is_locked: bool,
    pub is_sticky: bool,

    // Statistics
    pub view_count: i32,
    pub post_count: i32,
    pub last_post_id: Option<Uuid>,
    pub last_post_at: Option<DateTime<Utc>>,
    pub last_poster_id: Option<Uuid>,

    // Poll
    pub has_poll: bool,
    pub poll_id: Option<Uuid>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Topic with extended information for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicWithDetails {
    #[serde(flatten)]
    pub topic: Topic,
    pub author_username: String,
    pub last_poster_username: Option<String>,
    pub is_subscribed: bool,
}

/// Topic subscription for notifications
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TopicSubscription {
    pub id: Uuid,
    pub topic_id: Uuid,
    pub user_id: Uuid,
    pub notify_email: bool,
    pub created_at: DateTime<Utc>,
}

/// Request to create a new topic
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateTopicRequest {
    pub forum_id: Uuid,
    pub user_id: Uuid,

    #[validate(length(min = 3, max = 200))]
    pub title: String,

    #[validate(length(min = 1, max = 65535))]
    pub content: String,
}

/// Request to update a topic
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateTopicRequest {
    #[validate(length(min = 3, max = 200))]
    pub title: Option<String>,

    pub is_pinned: Option<bool>,
    pub is_locked: Option<bool>,
    pub is_sticky: Option<bool>,
}

/// Request to move topic to different forum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveTopicRequest {
    pub topic_id: Uuid,
    pub new_forum_id: Uuid,
    pub leave_redirect: bool,
}

/// Pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicListParams {
    pub page: i32,
    pub per_page: i32,
    pub sort_by: TopicSortBy,
}

impl Default for TopicListParams {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 25,
            sort_by: TopicSortBy::LastPost,
        }
    }
}

/// Sort options for topic listing
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TopicSortBy {
    LastPost,
    CreatedAt,
    Title,
    PostCount,
    ViewCount,
}

/// Topic service for managing topics
pub struct TopicService {
    db: PgPool,
}

impl TopicService {
    /// Creates a new topic service
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Creates a new topic
    pub async fn create_topic(&self, request: CreateTopicRequest) -> Result<Topic> {
        request.validate()?;

        // Verify forum exists
        let forum_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM forums WHERE id = $1)",
        )
        .bind(request.forum_id)
        .fetch_one(&self.db)
        .await?;

        if !forum_exists {
            return Err(TopicError::ForumNotFound(request.forum_id));
        }

        // Start transaction
        let mut tx = self.db.begin().await?;

        // Create topic
        let topic_id = Uuid::new_v4();
        let topic = sqlx::query_as::<_, Topic>(
            r#"
            INSERT INTO topics (
                id, forum_id, user_id, title, status, is_pinned, is_locked, is_sticky,
                view_count, post_count, has_poll, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, 'open', false, false, false, 0, 1, false, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(topic_id)
        .bind(request.forum_id)
        .bind(request.user_id)
        .bind(&request.title)
        .fetch_one(&mut *tx)
        .await?;

        // Create first post
        let post_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO posts (id, topic_id, user_id, content, created_at, updated_at)
            VALUES ($1, $2, $3, $4, NOW(), NOW())
            "#,
        )
        .bind(post_id)
        .bind(topic_id)
        .bind(request.user_id)
        .bind(&request.content)
        .execute(&mut *tx)
        .await?;

        // Update topic with first post
        sqlx::query(
            r#"
            UPDATE topics
            SET last_post_id = $2,
                last_post_at = NOW(),
                last_poster_id = $3
            WHERE id = $1
            "#,
        )
        .bind(topic_id)
        .bind(post_id)
        .bind(request.user_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(topic)
    }

    /// Gets a topic by ID
    pub async fn get_topic(&self, topic_id: Uuid) -> Result<Topic> {
        let topic = sqlx::query_as::<_, Topic>("SELECT * FROM topics WHERE id = $1")
            .bind(topic_id)
            .fetch_optional(&self.db)
            .await?
            .ok_or(TopicError::NotFound(topic_id))?;

        Ok(topic)
    }

    /// Gets a topic with extended details
    pub async fn get_topic_with_details(
        &self,
        topic_id: Uuid,
        viewer_id: Option<Uuid>,
    ) -> Result<TopicWithDetails> {
        let topic = self.get_topic(topic_id).await?;

        // Get author username
        let author_username = sqlx::query_scalar::<_, String>(
            "SELECT username FROM users WHERE id = $1",
        )
        .bind(topic.user_id)
        .fetch_one(&self.db)
        .await?;

        // Get last poster username
        let last_poster_username = if let Some(poster_id) = topic.last_poster_id {
            sqlx::query_scalar::<_, String>("SELECT username FROM users WHERE id = $1")
                .bind(poster_id)
                .fetch_optional(&self.db)
                .await?
        } else {
            None
        };

        // Check subscription
        let is_subscribed = if let Some(user_id) = viewer_id {
            sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM topic_subscriptions WHERE topic_id = $1 AND user_id = $2)",
            )
            .bind(topic_id)
            .bind(user_id)
            .fetch_one(&self.db)
            .await?
        } else {
            false
        };

        Ok(TopicWithDetails {
            topic,
            author_username,
            last_poster_username,
            is_subscribed,
        })
    }

    /// Lists topics in a forum with pagination
    pub async fn list_topics(
        &self,
        forum_id: Uuid,
        params: TopicListParams,
    ) -> Result<(Vec<TopicWithDetails>, i64)> {
        let offset = (params.page - 1) * params.per_page;

        let order_by = match params.sort_by {
            TopicSortBy::LastPost => "last_post_at DESC NULLS LAST",
            TopicSortBy::CreatedAt => "created_at DESC",
            TopicSortBy::Title => "title ASC",
            TopicSortBy::PostCount => "post_count DESC",
            TopicSortBy::ViewCount => "view_count DESC",
        };

        let query = format!(
            r#"
            SELECT t.*, u.username as author_username
            FROM topics t
            JOIN users u ON t.user_id = u.id
            WHERE t.forum_id = $1
            ORDER BY t.is_pinned DESC, t.is_sticky DESC, {}
            LIMIT $2 OFFSET $3
            "#,
            order_by
        );

        let topics: Vec<(Topic, String)> = sqlx::query_as(&query)
            .bind(forum_id)
            .bind(params.per_page)
            .bind(offset)
            .fetch_all(&self.db)
            .await?;

        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM topics WHERE forum_id = $1",
        )
        .bind(forum_id)
        .fetch_one(&self.db)
        .await?;

        let mut topics_with_details = Vec::new();
        for (topic, author_username) in topics {
            let last_poster_username = if let Some(poster_id) = topic.last_poster_id {
                sqlx::query_scalar::<_, String>("SELECT username FROM users WHERE id = $1")
                    .bind(poster_id)
                    .fetch_optional(&self.db)
                    .await?
            } else {
                None
            };

            topics_with_details.push(TopicWithDetails {
                topic,
                author_username,
                last_poster_username,
                is_subscribed: false,
            });
        }

        Ok((topics_with_details, total))
    }

    /// Updates a topic
    pub async fn update_topic(
        &self,
        topic_id: Uuid,
        user_id: Uuid,
        is_moderator: bool,
        request: UpdateTopicRequest,
    ) -> Result<Topic> {
        request.validate()?;

        let topic = self.get_topic(topic_id).await?;

        // Check permissions
        if topic.user_id != user_id && !is_moderator {
            return Err(TopicError::PermissionDenied);
        }

        let updated_topic = sqlx::query_as::<_, Topic>(
            r#"
            UPDATE topics
            SET title = COALESCE($2, title),
                is_pinned = COALESCE($3, is_pinned),
                is_locked = COALESCE($4, is_locked),
                is_sticky = COALESCE($5, is_sticky),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(topic_id)
        .bind(request.title)
        .bind(request.is_pinned)
        .bind(request.is_locked)
        .bind(request.is_sticky)
        .fetch_one(&self.db)
        .await?;

        Ok(updated_topic)
    }

    /// Locks a topic (moderator only)
    pub async fn lock_topic(&self, topic_id: Uuid) -> Result<()> {
        sqlx::query("UPDATE topics SET is_locked = true, updated_at = NOW() WHERE id = $1")
            .bind(topic_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// Unlocks a topic (moderator only)
    pub async fn unlock_topic(&self, topic_id: Uuid) -> Result<()> {
        sqlx::query("UPDATE topics SET is_locked = false, updated_at = NOW() WHERE id = $1")
            .bind(topic_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// Pins a topic (moderator only)
    pub async fn pin_topic(&self, topic_id: Uuid) -> Result<()> {
        sqlx::query("UPDATE topics SET is_pinned = true, updated_at = NOW() WHERE id = $1")
            .bind(topic_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// Unpins a topic (moderator only)
    pub async fn unpin_topic(&self, topic_id: Uuid) -> Result<()> {
        sqlx::query("UPDATE topics SET is_pinned = false, updated_at = NOW() WHERE id = $1")
            .bind(topic_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// Moves a topic to a different forum
    pub async fn move_topic(&self, request: MoveTopicRequest) -> Result<()> {
        // Verify new forum exists
        let forum_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM forums WHERE id = $1)",
        )
        .bind(request.new_forum_id)
        .fetch_one(&self.db)
        .await?;

        if !forum_exists {
            return Err(TopicError::ForumNotFound(request.new_forum_id));
        }

        let mut tx = self.db.begin().await?;

        // Get old forum_id
        let old_forum_id = sqlx::query_scalar::<_, Uuid>(
            "SELECT forum_id FROM topics WHERE id = $1",
        )
        .bind(request.topic_id)
        .fetch_one(&mut *tx)
        .await?;

        // Move topic
        sqlx::query("UPDATE topics SET forum_id = $2, updated_at = NOW() WHERE id = $1")
            .bind(request.topic_id)
            .bind(request.new_forum_id)
            .execute(&mut *tx)
            .await?;

        // Leave redirect if requested
        if request.leave_redirect {
            sqlx::query(
                r#"
                INSERT INTO topic_redirects (id, old_forum_id, new_forum_id, topic_id, created_at)
                VALUES ($1, $2, $3, $4, NOW())
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(old_forum_id)
            .bind(request.new_forum_id)
            .bind(request.topic_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    /// Deletes a topic and all its posts
    pub async fn delete_topic(&self, topic_id: Uuid, user_id: Uuid, is_moderator: bool) -> Result<()> {
        let topic = self.get_topic(topic_id).await?;

        // Check permissions
        if topic.user_id != user_id && !is_moderator {
            return Err(TopicError::PermissionDenied);
        }

        let mut tx = self.db.begin().await?;

        // Delete all posts
        sqlx::query("DELETE FROM posts WHERE topic_id = $1")
            .bind(topic_id)
            .execute(&mut *tx)
            .await?;

        // Delete topic
        sqlx::query("DELETE FROM topics WHERE id = $1")
            .bind(topic_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }

    /// Subscribes a user to a topic
    pub async fn subscribe(&self, topic_id: Uuid, user_id: Uuid, notify_email: bool) -> Result<TopicSubscription> {
        let subscription = sqlx::query_as::<_, TopicSubscription>(
            r#"
            INSERT INTO topic_subscriptions (id, topic_id, user_id, notify_email, created_at)
            VALUES ($1, $2, $3, $4, NOW())
            ON CONFLICT (topic_id, user_id) DO UPDATE
            SET notify_email = $4
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(topic_id)
        .bind(user_id)
        .bind(notify_email)
        .fetch_one(&self.db)
        .await?;

        Ok(subscription)
    }

    /// Unsubscribes a user from a topic
    pub async fn unsubscribe(&self, topic_id: Uuid, user_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM topic_subscriptions WHERE topic_id = $1 AND user_id = $2")
            .bind(topic_id)
            .bind(user_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// Gets all subscribers for a topic
    pub async fn get_subscribers(&self, topic_id: Uuid) -> Result<Vec<TopicSubscription>> {
        let subscriptions = sqlx::query_as::<_, TopicSubscription>(
            "SELECT * FROM topic_subscriptions WHERE topic_id = $1",
        )
        .bind(topic_id)
        .fetch_all(&self.db)
        .await?;

        Ok(subscriptions)
    }

    /// Increments view count
    pub async fn increment_views(&self, topic_id: Uuid) -> Result<()> {
        sqlx::query("UPDATE topics SET view_count = view_count + 1 WHERE id = $1")
            .bind(topic_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// Updates topic statistics (called when posts are added/removed)
    pub async fn update_stats(&self, topic_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE topics
            SET post_count = (SELECT COUNT(*) FROM posts WHERE topic_id = $1),
                last_post_id = (
                    SELECT id FROM posts
                    WHERE topic_id = $1
                    ORDER BY created_at DESC
                    LIMIT 1
                ),
                last_post_at = (
                    SELECT created_at FROM posts
                    WHERE topic_id = $1
                    ORDER BY created_at DESC
                    LIMIT 1
                ),
                last_poster_id = (
                    SELECT user_id FROM posts
                    WHERE topic_id = $1
                    ORDER BY created_at DESC
                    LIMIT 1
                ),
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(topic_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic_status() {
        assert_eq!(TopicStatus::Open as i32, TopicStatus::Open as i32);
        assert_ne!(TopicStatus::Open, TopicStatus::Locked);
    }

    #[test]
    fn test_create_topic_validation() {
        let request = CreateTopicRequest {
            forum_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            title: "Test Topic".to_string(),
            content: "This is a test topic content.".to_string(),
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_topic_list_params_default() {
        let params = TopicListParams::default();
        assert_eq!(params.page, 1);
        assert_eq!(params.per_page, 25);
    }
}
