//! Forum System
//!
//! Provides forum category and forum management with hierarchical structure
//! and permission-based access control.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;
use validator::Validate;

/// Forum-related errors
#[derive(Debug, Error)]
pub enum ForumError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Forum not found: {0}")]
    NotFound(Uuid),

    #[error("Category not found: {0}")]
    CategoryNotFound(Uuid),

    #[error("Permission denied: user class {user_class} requires minimum {required_class}")]
    PermissionDenied {
        user_class: i32,
        required_class: i32,
    },

    #[error("Invalid parent forum: {0}")]
    InvalidParent(Uuid),

    #[error("Validation error: {0}")]
    Validation(#[from] validator::ValidationErrors),

    #[error("Cannot delete forum with topics")]
    ForumHasTopics,
}

pub type Result<T> = std::result::Result<T, ForumError>;

/// Forum category (top-level organization)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ForumCategory {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Individual forum
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Forum {
    pub id: Uuid,
    pub category_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i32,

    // Permission levels (user class required)
    pub min_class_read: i32,
    pub min_class_write: i32,
    pub min_class_create: i32,

    // Statistics
    pub topic_count: i32,
    pub post_count: i32,
    pub last_post_id: Option<Uuid>,
    pub last_post_at: Option<DateTime<Utc>>,

    // Settings
    pub is_locked: bool,
    pub auto_lock_topics: bool,
    pub auto_lock_days: Option<i32>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Forum with statistics for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForumWithStats {
    #[serde(flatten)]
    pub forum: Forum,
    pub last_topic_title: Option<String>,
    pub last_poster_username: Option<String>,
    pub subforums: Vec<Forum>,
}

/// Forum permissions for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForumPermissions {
    pub can_read: bool,
    pub can_write: bool,
    pub can_create_topic: bool,
    pub can_moderate: bool,
}

/// Request to create a new forum category
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateCategoryRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,

    #[validate(length(max = 500))]
    pub description: Option<String>,

    pub sort_order: i32,
}

/// Request to create a new forum
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateForumRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,

    #[validate(length(max = 500))]
    pub description: Option<String>,

    pub category_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub min_class_read: i32,
    pub min_class_write: i32,
}

/// Request to update a forum
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateForumRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,

    #[validate(length(max = 500))]
    pub description: Option<String>,

    pub icon: Option<String>,
    pub sort_order: Option<i32>,
    pub min_class_read: Option<i32>,
    pub min_class_write: Option<i32>,
    pub is_locked: Option<bool>,
}

/// Forum service for managing forums and categories
pub struct ForumService {
    db: PgPool,
}

impl ForumService {
    /// Creates a new forum service
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Creates a new forum category
    pub async fn create_category(&self, request: CreateCategoryRequest) -> Result<ForumCategory> {
        request.validate()?;

        let category = sqlx::query_as::<_, ForumCategory>(
            r#"
            INSERT INTO forum_categories (id, name, description, sort_order, created_at, updated_at)
            VALUES ($1, $2, $3, $4, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&request.name)
        .bind(&request.description)
        .bind(request.sort_order)
        .fetch_one(&self.db)
        .await?;

        Ok(category)
    }

    /// Gets all forum categories
    pub async fn get_categories(&self) -> Result<Vec<ForumCategory>> {
        let categories = sqlx::query_as::<_, ForumCategory>(
            r#"
            SELECT * FROM forum_categories
            ORDER BY sort_order, name
            "#,
        )
        .fetch_all(&self.db)
        .await?;

        Ok(categories)
    }

    /// Creates a new forum
    pub async fn create_forum(&self, request: CreateForumRequest) -> Result<Forum> {
        request.validate()?;

        // Verify category exists
        let category_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM forum_categories WHERE id = $1)",
        )
        .bind(request.category_id)
        .fetch_one(&self.db)
        .await?;

        if !category_exists {
            return Err(ForumError::CategoryNotFound(request.category_id));
        }

        // Verify parent forum exists if specified
        if let Some(parent_id) = request.parent_id {
            let parent_exists = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM forums WHERE id = $1)",
            )
            .bind(parent_id)
            .fetch_one(&self.db)
            .await?;

            if !parent_exists {
                return Err(ForumError::InvalidParent(parent_id));
            }
        }

        let forum = sqlx::query_as::<_, Forum>(
            r#"
            INSERT INTO forums (
                id, category_id, parent_id, name, description, icon, sort_order,
                min_class_read, min_class_write, min_class_create,
                topic_count, post_count, is_locked, auto_lock_topics,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $8, 0, 0, false, false, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(request.category_id)
        .bind(request.parent_id)
        .bind(&request.name)
        .bind(&request.description)
        .bind(&request.icon)
        .bind(request.sort_order)
        .bind(request.min_class_read)
        .bind(request.min_class_write)
        .fetch_one(&self.db)
        .await?;

        Ok(forum)
    }

    /// Gets a forum by ID
    pub async fn get_forum(&self, forum_id: Uuid) -> Result<Forum> {
        let forum = sqlx::query_as::<_, Forum>("SELECT * FROM forums WHERE id = $1")
            .bind(forum_id)
            .fetch_optional(&self.db)
            .await?
            .ok_or(ForumError::NotFound(forum_id))?;

        Ok(forum)
    }

    /// Gets all forums in a category with statistics
    pub async fn get_category_forums(&self, category_id: Uuid) -> Result<Vec<ForumWithStats>> {
        let forums = sqlx::query_as::<_, Forum>(
            r#"
            SELECT * FROM forums
            WHERE category_id = $1 AND parent_id IS NULL
            ORDER BY sort_order, name
            "#,
        )
        .bind(category_id)
        .fetch_all(&self.db)
        .await?;

        let mut forums_with_stats = Vec::new();
        for forum in forums {
            let subforums = self.get_subforums(forum.id).await?;
            let (last_topic_title, last_poster_username) =
                self.get_last_post_info(forum.last_post_id).await?;

            forums_with_stats.push(ForumWithStats {
                forum,
                last_topic_title,
                last_poster_username,
                subforums,
            });
        }

        Ok(forums_with_stats)
    }

    /// Gets subforums of a parent forum
    pub async fn get_subforums(&self, parent_id: Uuid) -> Result<Vec<Forum>> {
        let subforums = sqlx::query_as::<_, Forum>(
            "SELECT * FROM forums WHERE parent_id = $1 ORDER BY sort_order, name",
        )
        .bind(parent_id)
        .fetch_all(&self.db)
        .await?;

        Ok(subforums)
    }

    /// Gets last post information for display
    async fn get_last_post_info(
        &self,
        post_id: Option<Uuid>,
    ) -> Result<(Option<String>, Option<String>)> {
        if let Some(post_id) = post_id {
            let result = sqlx::query_as::<_, (String, String)>(
                r#"
                SELECT t.title, u.username
                FROM posts p
                JOIN topics t ON p.topic_id = t.id
                JOIN users u ON p.user_id = u.id
                WHERE p.id = $1
                "#,
            )
            .bind(post_id)
            .fetch_optional(&self.db)
            .await?;

            if let Some((title, username)) = result {
                return Ok((Some(title), Some(username)));
            }
        }
        Ok((None, None))
    }

    /// Updates a forum
    pub async fn update_forum(&self, forum_id: Uuid, request: UpdateForumRequest) -> Result<Forum> {
        request.validate()?;

        let forum = self.get_forum(forum_id).await?;

        let updated_forum = sqlx::query_as::<_, Forum>(
            r#"
            UPDATE forums
            SET name = COALESCE($2, name),
                description = COALESCE($3, description),
                icon = COALESCE($4, icon),
                sort_order = COALESCE($5, sort_order),
                min_class_read = COALESCE($6, min_class_read),
                min_class_write = COALESCE($7, min_class_write),
                is_locked = COALESCE($8, is_locked),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(forum_id)
        .bind(request.name)
        .bind(request.description.or(Some(forum.description)))
        .bind(request.icon.or(Some(forum.icon)))
        .bind(request.sort_order)
        .bind(request.min_class_read)
        .bind(request.min_class_write)
        .bind(request.is_locked)
        .fetch_one(&self.db)
        .await?;

        Ok(updated_forum)
    }

    /// Deletes a forum (only if empty)
    pub async fn delete_forum(&self, forum_id: Uuid) -> Result<()> {
        // Check if forum has topics
        let has_topics = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM topics WHERE forum_id = $1)",
        )
        .bind(forum_id)
        .fetch_one(&self.db)
        .await?;

        if has_topics {
            return Err(ForumError::ForumHasTopics);
        }

        sqlx::query("DELETE FROM forums WHERE id = $1")
            .bind(forum_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// Checks user permissions for a forum
    pub async fn check_permissions(
        &self,
        forum_id: Uuid,
        user_class: i32,
        is_moderator: bool,
    ) -> Result<ForumPermissions> {
        let forum = self.get_forum(forum_id).await?;

        Ok(ForumPermissions {
            can_read: user_class >= forum.min_class_read || is_moderator,
            can_write: user_class >= forum.min_class_write || is_moderator,
            can_create_topic: user_class >= forum.min_class_create || is_moderator,
            can_moderate: is_moderator,
        })
    }

    /// Verifies user can read forum
    pub async fn verify_read_permission(&self, forum_id: Uuid, user_class: i32) -> Result<()> {
        let forum = self.get_forum(forum_id).await?;

        if user_class < forum.min_class_read {
            return Err(ForumError::PermissionDenied {
                user_class,
                required_class: forum.min_class_read,
            });
        }

        Ok(())
    }

    /// Verifies user can post in forum
    pub async fn verify_write_permission(&self, forum_id: Uuid, user_class: i32) -> Result<()> {
        let forum = self.get_forum(forum_id).await?;

        if user_class < forum.min_class_write {
            return Err(ForumError::PermissionDenied {
                user_class,
                required_class: forum.min_class_write,
            });
        }

        if forum.is_locked {
            return Err(ForumError::PermissionDenied {
                user_class,
                required_class: i32::MAX,
            });
        }

        Ok(())
    }

    /// Updates forum statistics (called when topics/posts are added)
    pub async fn update_stats(&self, forum_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE forums
            SET topic_count = (SELECT COUNT(*) FROM topics WHERE forum_id = $1),
                post_count = (SELECT COUNT(*) FROM posts p JOIN topics t ON p.topic_id = t.id WHERE t.forum_id = $1),
                last_post_id = (
                    SELECT p.id FROM posts p
                    JOIN topics t ON p.topic_id = t.id
                    WHERE t.forum_id = $1
                    ORDER BY p.created_at DESC
                    LIMIT 1
                ),
                last_post_at = (
                    SELECT p.created_at FROM posts p
                    JOIN topics t ON p.topic_id = t.id
                    WHERE t.forum_id = $1
                    ORDER BY p.created_at DESC
                    LIMIT 1
                ),
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(forum_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forum_permissions() {
        let perms = ForumPermissions {
            can_read: true,
            can_write: true,
            can_create_topic: true,
            can_moderate: false,
        };

        assert!(perms.can_read);
        assert!(perms.can_write);
        assert!(!perms.can_moderate);
    }

    #[test]
    fn test_create_forum_request_validation() {
        let request = CreateForumRequest {
            name: "Test Forum".to_string(),
            description: Some("Test description".to_string()),
            category_id: Uuid::new_v4(),
            parent_id: None,
            icon: Some("📚".to_string()),
            sort_order: 0,
            min_class_read: 0,
            min_class_write: 1,
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_empty_name_validation() {
        let request = CreateForumRequest {
            name: "".to_string(),
            description: None,
            category_id: Uuid::new_v4(),
            parent_id: None,
            icon: None,
            sort_order: 0,
            min_class_read: 0,
            min_class_write: 0,
        };

        assert!(request.validate().is_err());
    }
}
