//! Forum Posts
//!
//! Manages individual forum posts with BBCode/Markdown support, reactions,
//! edit history, and moderation features.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;
use validator::Validate;

/// Post-related errors
#[derive(Debug, Error)]
pub enum PostError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Post not found: {0}")]
    NotFound(Uuid),

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Topic is locked")]
    TopicLocked,

    #[error("Topic not found: {0}")]
    TopicNotFound(Uuid),

    #[error("Validation error: {0}")]
    Validation(#[from] validator::ValidationErrors),

    #[error("Cannot delete post: {0}")]
    CannotDelete(String),

    #[error("Invalid BBCode/Markdown: {0}")]
    InvalidMarkup(String),
}

pub type Result<T> = std::result::Result<T, PostError>;

/// Forum post
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Post {
    pub id: Uuid,
    pub topic_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub content_html: String,

    // Quoting
    pub quoted_post_id: Option<Uuid>,

    // Moderation
    pub is_deleted: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<Uuid>,
    pub delete_reason: Option<String>,

    // Editing
    pub edit_count: i32,
    pub edited_at: Option<DateTime<Utc>>,
    pub edited_by: Option<Uuid>,

    // Reactions
    pub like_count: i32,
    pub dislike_count: i32,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Post with extended information for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostWithDetails {
    #[serde(flatten)]
    pub post: Post,
    pub author_username: String,
    pub author_class: i32,
    pub author_post_count: i32,
    pub author_joined_at: DateTime<Utc>,
    pub user_reaction: Option<ReactionType>,
}

/// Post edit history entry
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PostEdit {
    pub id: Uuid,
    pub post_id: Uuid,
    pub user_id: Uuid,
    pub old_content: String,
    pub new_content: String,
    pub edit_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Post reaction (like/dislike)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PostReaction {
    pub id: Uuid,
    pub post_id: Uuid,
    pub user_id: Uuid,
    pub reaction_type: ReactionType,
    pub created_at: DateTime<Utc>,
}

/// Reaction type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "reaction_type", rename_all = "lowercase")]
pub enum ReactionType {
    Like,
    Dislike,
}

/// Post report
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PostReport {
    pub id: Uuid,
    pub post_id: Uuid,
    pub reporter_id: Uuid,
    pub reason: String,
    pub resolved: bool,
    pub resolved_by: Option<Uuid>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Request to create a new post
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreatePostRequest {
    pub topic_id: Uuid,
    pub user_id: Uuid,

    #[validate(length(min = 1, max = 65535))]
    pub content: String,

    pub quoted_post_id: Option<Uuid>,
}

/// Request to update a post
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdatePostRequest {
    #[validate(length(min = 1, max = 65535))]
    pub content: String,

    #[validate(length(max = 200))]
    pub edit_reason: Option<String>,
}

/// Request to report a post
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ReportPostRequest {
    pub post_id: Uuid,
    pub reporter_id: Uuid,

    #[validate(length(min = 10, max = 1000))]
    pub reason: String,
}

/// Pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostListParams {
    pub page: i32,
    pub per_page: i32,
}

impl Default for PostListParams {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 25,
        }
    }
}

/// Post service for managing posts
pub struct PostService {
    db: PgPool,
}

impl PostService {
    /// Creates a new post service
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Creates a new post
    pub async fn create_post(&self, request: CreatePostRequest) -> Result<Post> {
        request.validate()?;

        // Verify topic exists and is not locked
        let topic = sqlx::query_as::<_, (Uuid, bool)>(
            "SELECT id, is_locked FROM topics WHERE id = $1",
        )
        .bind(request.topic_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or(PostError::TopicNotFound(request.topic_id))?;

        if topic.1 {
            return Err(PostError::TopicLocked);
        }

        // Process content (BBCode/Markdown to HTML)
        let content_html = self.process_markup(&request.content)?;

        let mut tx = self.db.begin().await?;

        // Create post
        let post = sqlx::query_as::<_, Post>(
            r#"
            INSERT INTO posts (
                id, topic_id, user_id, content, content_html, quoted_post_id,
                is_deleted, edit_count, like_count, dislike_count,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, false, 0, 0, 0, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(request.topic_id)
        .bind(request.user_id)
        .bind(&request.content)
        .bind(&content_html)
        .bind(request.quoted_post_id)
        .fetch_one(&mut *tx)
        .await?;

        // Update topic stats
        sqlx::query(
            r#"
            UPDATE topics
            SET post_count = post_count + 1,
                last_post_id = $2,
                last_post_at = NOW(),
                last_poster_id = $3,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(request.topic_id)
        .bind(post.id)
        .bind(request.user_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(post)
    }

    /// Gets a post by ID
    pub async fn get_post(&self, post_id: Uuid) -> Result<Post> {
        let post = sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE id = $1 AND is_deleted = false")
            .bind(post_id)
            .fetch_optional(&self.db)
            .await?
            .ok_or(PostError::NotFound(post_id))?;

        Ok(post)
    }

    /// Gets a post with extended details
    pub async fn get_post_with_details(
        &self,
        post_id: Uuid,
        viewer_id: Option<Uuid>,
    ) -> Result<PostWithDetails> {
        let post = self.get_post(post_id).await?;

        // Get author details
        let (username, class, post_count, joined_at) = sqlx::query_as::<_, (String, i32, i32, DateTime<Utc>)>(
            r#"
            SELECT u.username, u.class,
                   (SELECT COUNT(*) FROM posts WHERE user_id = u.id) as post_count,
                   u.created_at
            FROM users u
            WHERE u.id = $1
            "#,
        )
        .bind(post.user_id)
        .fetch_one(&self.db)
        .await?;

        // Get user's reaction if logged in
        let user_reaction = if let Some(user_id) = viewer_id {
            sqlx::query_scalar::<_, ReactionType>(
                "SELECT reaction_type FROM post_reactions WHERE post_id = $1 AND user_id = $2",
            )
            .bind(post_id)
            .bind(user_id)
            .fetch_optional(&self.db)
            .await?
        } else {
            None
        };

        Ok(PostWithDetails {
            post,
            author_username: username,
            author_class: class,
            author_post_count: post_count,
            author_joined_at: joined_at,
            user_reaction,
        })
    }

    /// Lists posts in a topic with pagination
    pub async fn list_posts(
        &self,
        topic_id: Uuid,
        params: PostListParams,
        viewer_id: Option<Uuid>,
    ) -> Result<(Vec<PostWithDetails>, i64)> {
        let offset = (params.page - 1) * params.per_page;

        let posts = sqlx::query_as::<_, Post>(
            r#"
            SELECT * FROM posts
            WHERE topic_id = $1 AND is_deleted = false
            ORDER BY created_at ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(topic_id)
        .bind(params.per_page)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM posts WHERE topic_id = $1 AND is_deleted = false",
        )
        .bind(topic_id)
        .fetch_one(&self.db)
        .await?;

        let mut posts_with_details = Vec::new();
        for post in posts {
            let details = self.get_post_with_details(post.id, viewer_id).await?;
            posts_with_details.push(details);
        }

        Ok((posts_with_details, total))
    }

    /// Updates a post
    pub async fn update_post(
        &self,
        post_id: Uuid,
        user_id: Uuid,
        is_moderator: bool,
        request: UpdatePostRequest,
    ) -> Result<Post> {
        request.validate()?;

        let post = self.get_post(post_id).await?;

        // Check permissions
        if post.user_id != user_id && !is_moderator {
            return Err(PostError::PermissionDenied);
        }

        // Process new content
        let content_html = self.process_markup(&request.content)?;

        let mut tx = self.db.begin().await?;

        // Save edit history
        sqlx::query(
            r#"
            INSERT INTO post_edits (id, post_id, user_id, old_content, new_content, edit_reason, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(post_id)
        .bind(user_id)
        .bind(&post.content)
        .bind(&request.content)
        .bind(&request.edit_reason)
        .execute(&mut *tx)
        .await?;

        // Update post
        let updated_post = sqlx::query_as::<_, Post>(
            r#"
            UPDATE posts
            SET content = $2,
                content_html = $3,
                edit_count = edit_count + 1,
                edited_at = NOW(),
                edited_by = $4,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(post_id)
        .bind(&request.content)
        .bind(&content_html)
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(updated_post)
    }

    /// Soft deletes a post
    pub async fn delete_post(
        &self,
        post_id: Uuid,
        user_id: Uuid,
        is_moderator: bool,
        reason: Option<String>,
    ) -> Result<()> {
        let post = self.get_post(post_id).await?;

        // Check permissions
        if post.user_id != user_id && !is_moderator {
            return Err(PostError::PermissionDenied);
        }

        // Check if it's the first post (cannot delete)
        let is_first_post = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM topics
                WHERE id = $1 AND last_post_id = $2
                AND post_count = 1
            )
            "#,
        )
        .bind(post.topic_id)
        .bind(post_id)
        .fetch_one(&self.db)
        .await?;

        if is_first_post {
            return Err(PostError::CannotDelete(
                "Cannot delete the first post. Delete the entire topic instead.".to_string(),
            ));
        }

        let mut tx = self.db.begin().await?;

        // Soft delete post
        sqlx::query(
            r#"
            UPDATE posts
            SET is_deleted = true,
                deleted_at = NOW(),
                deleted_by = $2,
                delete_reason = $3,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(post_id)
        .bind(user_id)
        .bind(reason)
        .execute(&mut *tx)
        .await?;

        // Update topic stats
        sqlx::query(
            r#"
            UPDATE topics
            SET post_count = post_count - 1,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(post.topic_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    /// Adds a reaction to a post
    pub async fn add_reaction(
        &self,
        post_id: Uuid,
        user_id: Uuid,
        reaction_type: ReactionType,
    ) -> Result<PostReaction> {
        let mut tx = self.db.begin().await?;

        // Remove existing reaction if any
        sqlx::query("DELETE FROM post_reactions WHERE post_id = $1 AND user_id = $2")
            .bind(post_id)
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        // Add new reaction
        let reaction = sqlx::query_as::<_, PostReaction>(
            r#"
            INSERT INTO post_reactions (id, post_id, user_id, reaction_type, created_at)
            VALUES ($1, $2, $3, $4, NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(post_id)
        .bind(user_id)
        .bind(reaction_type)
        .fetch_one(&mut *tx)
        .await?;

        // Update post reaction counts
        sqlx::query(
            r#"
            UPDATE posts
            SET like_count = (SELECT COUNT(*) FROM post_reactions WHERE post_id = $1 AND reaction_type = 'like'),
                dislike_count = (SELECT COUNT(*) FROM post_reactions WHERE post_id = $1 AND reaction_type = 'dislike')
            WHERE id = $1
            "#,
        )
        .bind(post_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(reaction)
    }

    /// Removes a reaction from a post
    pub async fn remove_reaction(&self, post_id: Uuid, user_id: Uuid) -> Result<()> {
        let mut tx = self.db.begin().await?;

        sqlx::query("DELETE FROM post_reactions WHERE post_id = $1 AND user_id = $2")
            .bind(post_id)
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        // Update post reaction counts
        sqlx::query(
            r#"
            UPDATE posts
            SET like_count = (SELECT COUNT(*) FROM post_reactions WHERE post_id = $1 AND reaction_type = 'like'),
                dislike_count = (SELECT COUNT(*) FROM post_reactions WHERE post_id = $1 AND reaction_type = 'dislike')
            WHERE id = $1
            "#,
        )
        .bind(post_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    /// Gets edit history for a post
    pub async fn get_edit_history(&self, post_id: Uuid) -> Result<Vec<PostEdit>> {
        let edits = sqlx::query_as::<_, PostEdit>(
            "SELECT * FROM post_edits WHERE post_id = $1 ORDER BY created_at DESC",
        )
        .bind(post_id)
        .fetch_all(&self.db)
        .await?;

        Ok(edits)
    }

    /// Reports a post for moderation
    pub async fn report_post(&self, request: ReportPostRequest) -> Result<PostReport> {
        request.validate()?;

        let report = sqlx::query_as::<_, PostReport>(
            r#"
            INSERT INTO post_reports (id, post_id, reporter_id, reason, resolved, created_at)
            VALUES ($1, $2, $3, $4, false, NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(request.post_id)
        .bind(request.reporter_id)
        .bind(&request.reason)
        .fetch_one(&self.db)
        .await?;

        Ok(report)
    }

    /// Gets quotes of a post
    pub async fn get_quotes(&self, post_id: Uuid) -> Result<Vec<Post>> {
        let quotes = sqlx::query_as::<_, Post>(
            "SELECT * FROM posts WHERE quoted_post_id = $1 AND is_deleted = false ORDER BY created_at",
        )
        .bind(post_id)
        .fetch_all(&self.db)
        .await?;

        Ok(quotes)
    }

    /// Processes BBCode/Markdown to HTML
    fn process_markup(&self, content: &str) -> Result<String> {
        // Basic sanitization and conversion
        // In production, use proper BBCode parser or Markdown parser
        let html = ammonia::clean(content);

        // Basic BBCode support
        let html = html
            .replace("[b]", "<strong>")
            .replace("[/b]", "</strong>")
            .replace("[i]", "<em>")
            .replace("[/i]", "</em>")
            .replace("[u]", "<u>")
            .replace("[/u]", "</u>");

        Ok(html)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reaction_type() {
        assert_eq!(ReactionType::Like, ReactionType::Like);
        assert_ne!(ReactionType::Like, ReactionType::Dislike);
    }

    #[test]
    fn test_create_post_validation() {
        let request = CreatePostRequest {
            topic_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            content: "This is a test post.".to_string(),
            quoted_post_id: None,
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_post_list_params_default() {
        let params = PostListParams::default();
        assert_eq!(params.page, 1);
        assert_eq!(params.per_page, 25);
    }
}
