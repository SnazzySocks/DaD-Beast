//! Wiki and Knowledge Base
//!
//! Provides wiki functionality with versioned pages, categories,
//! and collaborative editing features.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;
use validator::Validate;

/// Wiki-related errors
#[derive(Debug, Error)]
pub enum WikiError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Page not found: {0}")]
    PageNotFound(String),

    #[error("Revision not found: {0}")]
    RevisionNotFound(Uuid),

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Page already exists: {0}")]
    PageAlreadyExists(String),

    #[error("Invalid page slug: {0}")]
    InvalidSlug(String),

    #[error("Validation error: {0}")]
    Validation(#[from] validator::ValidationErrors),

    #[error("Category not found: {0}")]
    CategoryNotFound(Uuid),
}

pub type Result<T> = std::result::Result<T, WikiError>;

/// Wiki page
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WikiPage {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub content: String,
    pub content_html: String,
    pub category_id: Option<Uuid>,

    // Metadata
    pub created_by: Uuid,
    pub last_edited_by: Uuid,
    pub revision_count: i32,
    pub view_count: i32,

    // Permissions
    pub min_class_read: i32,
    pub min_class_edit: i32,

    // Status
    pub is_locked: bool,
    pub is_published: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Wiki page with extended information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiPageWithDetails {
    #[serde(flatten)]
    pub page: WikiPage,
    pub author_username: String,
    pub last_editor_username: String,
    pub category_name: Option<String>,
}

/// Wiki page revision (version history)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WikiRevision {
    pub id: Uuid,
    pub page_id: Uuid,
    pub title: String,
    pub content: String,
    pub content_html: String,
    pub revision_number: i32,
    pub edited_by: Uuid,
    pub edit_comment: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Wiki page revision with user details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiRevisionWithDetails {
    #[serde(flatten)]
    pub revision: WikiRevision,
    pub editor_username: String,
}

/// Wiki category
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WikiCategory {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub slug: String,
    pub parent_id: Option<Uuid>,
    pub sort_order: i32,
    pub page_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create a wiki page
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateWikiPageRequest {
    #[validate(length(min = 1, max = 200))]
    pub title: String,

    #[validate(length(min = 1, max = 200))]
    pub slug: String,

    #[validate(length(min = 1, max = 100000))]
    pub content: String,

    pub category_id: Option<Uuid>,
    pub min_class_read: i32,
    pub min_class_edit: i32,
}

/// Request to update a wiki page
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateWikiPageRequest {
    #[validate(length(min = 1, max = 200))]
    pub title: Option<String>,

    #[validate(length(min = 1, max = 100000))]
    pub content: Option<String>,

    #[validate(length(max = 500))]
    pub edit_comment: Option<String>,

    pub category_id: Option<Uuid>,
    pub is_published: Option<bool>,
}

/// Request to create a wiki category
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateCategoryRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,

    #[validate(length(max = 500))]
    pub description: Option<String>,

    #[validate(length(min = 1, max = 100))]
    pub slug: String,

    pub parent_id: Option<Uuid>,
    pub sort_order: i32,
}

/// Search parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiSearchParams {
    pub query: String,
    pub category_id: Option<Uuid>,
    pub page: i32,
    pub per_page: i32,
}

impl Default for WikiSearchParams {
    fn default() -> Self {
        Self {
            query: String::new(),
            category_id: None,
            page: 1,
            per_page: 25,
        }
    }
}

/// Wiki service for managing wiki pages
pub struct WikiService {
    db: PgPool,
}

impl WikiService {
    /// Creates a new wiki service
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Creates a new wiki page
    pub async fn create_page(
        &self,
        user_id: Uuid,
        request: CreateWikiPageRequest,
    ) -> Result<WikiPage> {
        request.validate()?;

        // Validate slug
        if !self.is_valid_slug(&request.slug) {
            return Err(WikiError::InvalidSlug(request.slug));
        }

        // Check if page already exists
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM wiki_pages WHERE slug = $1)",
        )
        .bind(&request.slug)
        .fetch_one(&self.db)
        .await?;

        if exists {
            return Err(WikiError::PageAlreadyExists(request.slug));
        }

        // Verify category exists if specified
        if let Some(category_id) = request.category_id {
            let category_exists = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM wiki_categories WHERE id = $1)",
            )
            .bind(category_id)
            .fetch_one(&self.db)
            .await?;

            if !category_exists {
                return Err(WikiError::CategoryNotFound(category_id));
            }
        }

        // Process content to HTML
        let content_html = self.process_markdown(&request.content)?;

        let mut tx = self.db.begin().await?;

        // Create page
        let page = sqlx::query_as::<_, WikiPage>(
            r#"
            INSERT INTO wiki_pages (
                id, slug, title, content, content_html, category_id,
                created_by, last_edited_by, revision_count, view_count,
                min_class_read, min_class_edit, is_locked, is_published,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $7, 1, 0, $8, $9, false, true, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&request.slug)
        .bind(&request.title)
        .bind(&request.content)
        .bind(&content_html)
        .bind(request.category_id)
        .bind(user_id)
        .bind(request.min_class_read)
        .bind(request.min_class_edit)
        .fetch_one(&mut *tx)
        .await?;

        // Create first revision
        sqlx::query(
            r#"
            INSERT INTO wiki_revisions (
                id, page_id, title, content, content_html,
                revision_number, edited_by, edit_comment, created_at
            )
            VALUES ($1, $2, $3, $4, $5, 1, $6, 'Initial version', NOW())
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(page.id)
        .bind(&request.title)
        .bind(&request.content)
        .bind(&content_html)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

        // Update category page count
        if let Some(category_id) = request.category_id {
            sqlx::query(
                "UPDATE wiki_categories SET page_count = page_count + 1 WHERE id = $1",
            )
            .bind(category_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(page)
    }

    /// Gets a wiki page by slug
    pub async fn get_page(&self, slug: &str) -> Result<WikiPage> {
        let page = sqlx::query_as::<_, WikiPage>(
            "SELECT * FROM wiki_pages WHERE slug = $1 AND is_published = true",
        )
        .bind(slug)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| WikiError::PageNotFound(slug.to_string()))?;

        Ok(page)
    }

    /// Gets a wiki page with details
    pub async fn get_page_with_details(&self, slug: &str) -> Result<WikiPageWithDetails> {
        let page = self.get_page(slug).await?;

        let author_username = sqlx::query_scalar::<_, String>(
            "SELECT username FROM users WHERE id = $1",
        )
        .bind(page.created_by)
        .fetch_one(&self.db)
        .await?;

        let last_editor_username = sqlx::query_scalar::<_, String>(
            "SELECT username FROM users WHERE id = $1",
        )
        .bind(page.last_edited_by)
        .fetch_one(&self.db)
        .await?;

        let category_name = if let Some(category_id) = page.category_id {
            sqlx::query_scalar::<_, String>(
                "SELECT name FROM wiki_categories WHERE id = $1",
            )
            .bind(category_id)
            .fetch_optional(&self.db)
            .await?
        } else {
            None
        };

        Ok(WikiPageWithDetails {
            page,
            author_username,
            last_editor_username,
            category_name,
        })
    }

    /// Updates a wiki page
    pub async fn update_page(
        &self,
        slug: &str,
        user_id: Uuid,
        user_class: i32,
        request: UpdateWikiPageRequest,
    ) -> Result<WikiPage> {
        request.validate()?;

        let page = self.get_page(slug).await?;

        // Check permissions
        if user_class < page.min_class_edit {
            return Err(WikiError::PermissionDenied);
        }

        if page.is_locked {
            return Err(WikiError::PermissionDenied);
        }

        let mut tx = self.db.begin().await?;

        // Process new content if provided
        let (content, content_html) = if let Some(new_content) = &request.content {
            let html = self.process_markdown(new_content)?;
            (new_content.clone(), html)
        } else {
            (page.content.clone(), page.content_html.clone())
        };

        let title = request.title.unwrap_or(page.title.clone());

        // Update page
        let updated_page = sqlx::query_as::<_, WikiPage>(
            r#"
            UPDATE wiki_pages
            SET title = $2,
                content = $3,
                content_html = $4,
                category_id = COALESCE($5, category_id),
                is_published = COALESCE($6, is_published),
                last_edited_by = $7,
                revision_count = revision_count + 1,
                updated_at = NOW()
            WHERE slug = $1
            RETURNING *
            "#,
        )
        .bind(slug)
        .bind(&title)
        .bind(&content)
        .bind(&content_html)
        .bind(request.category_id)
        .bind(request.is_published)
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?;

        // Create revision
        sqlx::query(
            r#"
            INSERT INTO wiki_revisions (
                id, page_id, title, content, content_html,
                revision_number, edited_by, edit_comment, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(updated_page.id)
        .bind(&title)
        .bind(&content)
        .bind(&content_html)
        .bind(updated_page.revision_count)
        .bind(user_id)
        .bind(request.edit_comment)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(updated_page)
    }

    /// Deletes a wiki page
    pub async fn delete_page(&self, slug: &str, user_id: Uuid, is_admin: bool) -> Result<()> {
        let page = self.get_page(slug).await?;

        // Only admin or creator can delete
        if !is_admin && page.created_by != user_id {
            return Err(WikiError::PermissionDenied);
        }

        let mut tx = self.db.begin().await?;

        // Delete revisions
        sqlx::query("DELETE FROM wiki_revisions WHERE page_id = $1")
            .bind(page.id)
            .execute(&mut *tx)
            .await?;

        // Delete page
        sqlx::query("DELETE FROM wiki_pages WHERE id = $1")
            .bind(page.id)
            .execute(&mut *tx)
            .await?;

        // Update category page count
        if let Some(category_id) = page.category_id {
            sqlx::query(
                "UPDATE wiki_categories SET page_count = page_count - 1 WHERE id = $1",
            )
            .bind(category_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    /// Gets page revision history
    pub async fn get_revisions(&self, slug: &str) -> Result<Vec<WikiRevisionWithDetails>> {
        let page = self.get_page(slug).await?;

        let revisions = sqlx::query_as::<_, WikiRevision>(
            "SELECT * FROM wiki_revisions WHERE page_id = $1 ORDER BY revision_number DESC",
        )
        .bind(page.id)
        .fetch_all(&self.db)
        .await?;

        let mut revisions_with_details = Vec::new();
        for revision in revisions {
            let editor_username = sqlx::query_scalar::<_, String>(
                "SELECT username FROM users WHERE id = $1",
            )
            .bind(revision.edited_by)
            .fetch_one(&self.db)
            .await?;

            revisions_with_details.push(WikiRevisionWithDetails {
                revision,
                editor_username,
            });
        }

        Ok(revisions_with_details)
    }

    /// Gets a specific revision
    pub async fn get_revision(&self, revision_id: Uuid) -> Result<WikiRevision> {
        let revision = sqlx::query_as::<_, WikiRevision>(
            "SELECT * FROM wiki_revisions WHERE id = $1",
        )
        .bind(revision_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or(WikiError::RevisionNotFound(revision_id))?;

        Ok(revision)
    }

    /// Reverts page to a specific revision
    pub async fn revert_to_revision(
        &self,
        slug: &str,
        revision_id: Uuid,
        user_id: Uuid,
    ) -> Result<WikiPage> {
        let page = self.get_page(slug).await?;
        let revision = self.get_revision(revision_id).await?;

        if revision.page_id != page.id {
            return Err(WikiError::RevisionNotFound(revision_id));
        }

        let update_request = UpdateWikiPageRequest {
            title: Some(revision.title),
            content: Some(revision.content),
            edit_comment: Some(format!("Reverted to revision #{}", revision.revision_number)),
            category_id: None,
            is_published: None,
        };

        self.update_page(slug, user_id, i32::MAX, update_request).await
    }

    /// Searches wiki pages
    pub async fn search_pages(&self, params: WikiSearchParams) -> Result<(Vec<WikiPageWithDetails>, i64)> {
        let offset = (params.page - 1) * params.per_page;
        let search_term = format!("%{}%", params.query);

        let mut query = String::from(
            r#"
            SELECT * FROM wiki_pages
            WHERE is_published = true
              AND (title ILIKE $1 OR content ILIKE $1)
            "#,
        );

        if params.category_id.is_some() {
            query.push_str(" AND category_id = $2");
        }

        query.push_str(" ORDER BY view_count DESC, title LIMIT $3 OFFSET $4");

        let pages = if let Some(category_id) = params.category_id {
            sqlx::query_as::<_, WikiPage>(&query)
                .bind(&search_term)
                .bind(category_id)
                .bind(params.per_page)
                .bind(offset)
                .fetch_all(&self.db)
                .await?
        } else {
            sqlx::query_as::<_, WikiPage>(&query)
                .bind(&search_term)
                .bind(params.per_page)
                .bind(offset)
                .fetch_all(&self.db)
                .await?
        };

        let total = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM wiki_pages
            WHERE is_published = true AND (title ILIKE $1 OR content ILIKE $1)
            "#,
        )
        .bind(&search_term)
        .fetch_one(&self.db)
        .await?;

        let mut pages_with_details = Vec::new();
        for page in pages {
            let details = self.get_page_with_details(&page.slug).await?;
            pages_with_details.push(details);
        }

        Ok((pages_with_details, total))
    }

    /// Creates a wiki category
    pub async fn create_category(&self, request: CreateCategoryRequest) -> Result<WikiCategory> {
        request.validate()?;

        let category = sqlx::query_as::<_, WikiCategory>(
            r#"
            INSERT INTO wiki_categories (
                id, name, description, slug, parent_id, sort_order,
                page_count, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, 0, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&request.name)
        .bind(&request.description)
        .bind(&request.slug)
        .bind(request.parent_id)
        .bind(request.sort_order)
        .fetch_one(&self.db)
        .await?;

        Ok(category)
    }

    /// Gets all categories
    pub async fn get_categories(&self) -> Result<Vec<WikiCategory>> {
        let categories = sqlx::query_as::<_, WikiCategory>(
            "SELECT * FROM wiki_categories ORDER BY sort_order, name",
        )
        .fetch_all(&self.db)
        .await?;

        Ok(categories)
    }

    /// Gets pages in a category
    pub async fn get_pages_by_category(&self, category_id: Uuid) -> Result<Vec<WikiPageWithDetails>> {
        let pages = sqlx::query_as::<_, WikiPage>(
            r#"
            SELECT * FROM wiki_pages
            WHERE category_id = $1 AND is_published = true
            ORDER BY title
            "#,
        )
        .bind(category_id)
        .fetch_all(&self.db)
        .await?;

        let mut pages_with_details = Vec::new();
        for page in pages {
            let details = self.get_page_with_details(&page.slug).await?;
            pages_with_details.push(details);
        }

        Ok(pages_with_details)
    }

    /// Increments page view count
    pub async fn increment_views(&self, slug: &str) -> Result<()> {
        sqlx::query("UPDATE wiki_pages SET view_count = view_count + 1 WHERE slug = $1")
            .bind(slug)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// Validates slug format
    fn is_valid_slug(&self, slug: &str) -> bool {
        // Slug should be lowercase alphanumeric with hyphens
        slug.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            && !slug.is_empty()
    }

    /// Processes Markdown to HTML
    fn process_markdown(&self, content: &str) -> Result<String> {
        use pulldown_cmark::{html, Parser};

        let parser = Parser::new(content);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        // Sanitize HTML
        Ok(ammonia::clean(&html_output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slug_validation() {
        let service = WikiService::new(PgPool::connect("").await.unwrap());

        assert!(service.is_valid_slug("hello-world"));
        assert!(service.is_valid_slug("test_page_123"));
        assert!(!service.is_valid_slug("hello world"));
        assert!(!service.is_valid_slug("hello@world"));
    }

    #[test]
    fn test_create_page_validation() {
        let request = CreateWikiPageRequest {
            title: "Test Page".to_string(),
            slug: "test-page".to_string(),
            content: "# Test Content\n\nThis is a test.".to_string(),
            category_id: None,
            min_class_read: 0,
            min_class_edit: 1,
        };

        assert!(request.validate().is_ok());
    }
}
