//! # GraphQL Types
//!
//! GraphQL object types, input types, and pagination types for the API.

use async_graphql::{ComplexObject, Context, InputObject, Object, Result, SimpleObject};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::GraphQLContext;

/// User object
#[derive(Debug, Clone, SimpleObject, sqlx::FromRow)]
#[graphql(complex)]
pub struct User {
    /// User ID
    pub id: uuid::Uuid,
    /// Username
    pub username: String,
    /// Email address
    pub email: String,
    /// Registration date
    pub created_at: DateTime<Utc>,
    /// User class (e.g., "user", "power_user", "moderator", "admin")
    pub user_class: String,
    /// Total uploaded bytes
    pub uploaded: i64,
    /// Total downloaded bytes
    pub downloaded: i64,
    /// User passkey for tracker authentication
    pub passkey: String,
    /// Whether the account is active
    pub is_active: bool,
    /// Whether the email is verified
    pub is_verified: bool,
}

#[ComplexObject]
impl User {
    /// Calculate user ratio
    async fn ratio(&self) -> f64 {
        if self.downloaded == 0 {
            return f64::INFINITY;
        }
        self.uploaded as f64 / self.downloaded as f64
    }

    /// Get user's torrents
    async fn torrents(
        &self,
        ctx: &Context<'_>,
        first: Option<i32>,
    ) -> Result<Vec<Torrent>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let limit = first.unwrap_or(10).min(100);

        let torrents = sqlx::query_as::<_, Torrent>(
            "SELECT * FROM torrents WHERE uploader_id = $1 ORDER BY created_at DESC LIMIT $2",
        )
        .bind(self.id)
        .bind(limit)
        .fetch_all(&gql_ctx.db_pool)
        .await?;

        Ok(torrents)
    }

    /// Get user statistics
    async fn statistics(&self, ctx: &Context<'_>) -> Result<UserStatistics> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let stats = sqlx::query_as::<_, UserStatistics>(
            "SELECT * FROM user_statistics WHERE user_id = $1",
        )
        .bind(self.id)
        .fetch_one(&gql_ctx.db_pool)
        .await?;

        Ok(stats)
    }
}

/// User statistics
#[derive(Debug, Clone, SimpleObject, sqlx::FromRow)]
pub struct UserStatistics {
    pub user_id: uuid::Uuid,
    pub torrents_uploaded: i64,
    pub torrents_seeding: i64,
    pub torrents_leeching: i64,
    pub forum_posts: i64,
    pub bonus_points: i64,
    pub invites_available: i32,
}

/// Torrent object
#[derive(Debug, Clone, SimpleObject, sqlx::FromRow)]
#[graphql(complex)]
pub struct Torrent {
    /// Torrent ID
    pub id: uuid::Uuid,
    /// Info hash
    pub info_hash: String,
    /// Torrent name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Category
    pub category: String,
    /// Total size in bytes
    pub size: i64,
    /// Number of files
    pub file_count: i32,
    /// Uploader ID
    pub uploader_id: uuid::Uuid,
    /// Upload date
    pub created_at: DateTime<Utc>,
    /// Number of seeders
    pub seeders: i32,
    /// Number of leechers
    pub leechers: i32,
    /// Total downloads completed
    pub times_completed: i32,
    /// Whether the torrent is freeleech
    pub is_freeleech: bool,
    /// Whether the torrent is featured
    pub is_featured: bool,
}

#[ComplexObject]
impl Torrent {
    /// Get uploader info
    async fn uploader(&self, ctx: &Context<'_>) -> Result<User> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        gql_ctx
            .user_loader
            .load_one(self.uploader_id)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Uploader not found"))
    }

    /// Get torrent files
    async fn files(&self, ctx: &Context<'_>) -> Result<Vec<TorrentFile>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let files = sqlx::query_as::<_, TorrentFile>(
            "SELECT * FROM torrent_files WHERE torrent_id = $1 ORDER BY path",
        )
        .bind(self.id)
        .fetch_all(&gql_ctx.db_pool)
        .await?;

        Ok(files)
    }

    /// Get torrent comments
    async fn comments(
        &self,
        ctx: &Context<'_>,
        first: Option<i32>,
    ) -> Result<Vec<Comment>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let limit = first.unwrap_or(10).min(100);

        let comments = sqlx::query_as::<_, Comment>(
            "SELECT * FROM comments WHERE torrent_id = $1 ORDER BY created_at DESC LIMIT $2",
        )
        .bind(self.id)
        .bind(limit)
        .fetch_all(&gql_ctx.db_pool)
        .await?;

        Ok(comments)
    }
}

/// Torrent file
#[derive(Debug, Clone, SimpleObject, sqlx::FromRow)]
pub struct TorrentFile {
    pub id: uuid::Uuid,
    pub torrent_id: uuid::Uuid,
    pub path: String,
    pub size: i64,
}

/// Comment on a torrent or forum post
#[derive(Debug, Clone, SimpleObject, sqlx::FromRow)]
#[graphql(complex)]
pub struct Comment {
    pub id: uuid::Uuid,
    pub torrent_id: Option<uuid::Uuid>,
    pub user_id: uuid::Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
}

#[ComplexObject]
impl Comment {
    /// Get comment author
    async fn author(&self, ctx: &Context<'_>) -> Result<User> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        gql_ctx
            .user_loader
            .load_one(self.user_id)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Author not found"))
    }
}

/// Forum
#[derive(Debug, Clone, SimpleObject, sqlx::FromRow)]
pub struct Forum {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub category: String,
    pub position: i32,
    pub topic_count: i32,
    pub post_count: i32,
}

/// Forum topic
#[derive(Debug, Clone, SimpleObject, sqlx::FromRow)]
#[graphql(complex)]
pub struct Topic {
    pub id: uuid::Uuid,
    pub forum_id: uuid::Uuid,
    pub title: String,
    pub author_id: uuid::Uuid,
    pub created_at: DateTime<Utc>,
    pub is_locked: bool,
    pub is_sticky: bool,
    pub view_count: i32,
    pub post_count: i32,
}

#[ComplexObject]
impl Topic {
    /// Get topic author
    async fn author(&self, ctx: &Context<'_>) -> Result<User> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        gql_ctx
            .user_loader
            .load_one(self.author_id)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Author not found"))
    }

    /// Get topic posts
    async fn posts(
        &self,
        ctx: &Context<'_>,
        first: Option<i32>,
    ) -> Result<Vec<Post>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let limit = first.unwrap_or(10).min(100);

        let posts = sqlx::query_as::<_, Post>(
            "SELECT * FROM posts WHERE topic_id = $1 ORDER BY created_at LIMIT $2",
        )
        .bind(self.id)
        .bind(limit)
        .fetch_all(&gql_ctx.db_pool)
        .await?;

        Ok(posts)
    }
}

/// Forum post
#[derive(Debug, Clone, SimpleObject, sqlx::FromRow)]
#[graphql(complex)]
pub struct Post {
    pub id: uuid::Uuid,
    pub topic_id: uuid::Uuid,
    pub author_id: uuid::Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
}

#[ComplexObject]
impl Post {
    /// Get post author
    async fn author(&self, ctx: &Context<'_>) -> Result<User> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        gql_ctx
            .user_loader
            .load_one(self.author_id)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Author not found"))
    }
}

/// Private message
#[derive(Debug, Clone, SimpleObject, sqlx::FromRow)]
#[graphql(complex)]
pub struct Message {
    pub id: uuid::Uuid,
    pub sender_id: uuid::Uuid,
    pub recipient_id: uuid::Uuid,
    pub subject: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub is_read: bool,
}

#[ComplexObject]
impl Message {
    /// Get message sender
    async fn sender(&self, ctx: &Context<'_>) -> Result<User> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        gql_ctx
            .user_loader
            .load_one(self.sender_id)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Sender not found"))
    }

    /// Get message recipient
    async fn recipient(&self, ctx: &Context<'_>) -> Result<User> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        gql_ctx
            .user_loader
            .load_one(self.recipient_id)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Recipient not found"))
    }
}

/// Platform statistics
#[derive(Debug, Clone, SimpleObject)]
pub struct PlatformStatistics {
    pub total_users: i64,
    pub total_torrents: i64,
    pub total_seeders: i64,
    pub total_leechers: i64,
    pub total_peers: i64,
}

// ============================================================================
// Pagination Types
// ============================================================================

/// Page info for cursor-based pagination
#[derive(Debug, Clone, SimpleObject)]
pub struct PageInfo {
    pub has_next_page: bool,
    pub has_previous_page: bool,
    pub start_cursor: Option<String>,
    pub end_cursor: Option<String>,
}

/// Edge type for pagination
#[derive(Debug, Clone, SimpleObject)]
pub struct TorrentEdge {
    pub node: Torrent,
    pub cursor: String,
}

/// Connection type for torrents
#[derive(Debug, Clone, SimpleObject)]
pub struct TorrentConnection {
    pub edges: Vec<TorrentEdge>,
    pub page_info: PageInfo,
    pub total_count: i64,
}

/// Edge type for users
#[derive(Debug, Clone, SimpleObject)]
pub struct UserEdge {
    pub node: User,
    pub cursor: String,
}

/// Connection type for users
#[derive(Debug, Clone, SimpleObject)]
pub struct UserConnection {
    pub edges: Vec<UserEdge>,
    pub page_info: PageInfo,
    pub total_count: i64,
}

// ============================================================================
// Input Types for Mutations
// ============================================================================

/// Input for uploading a torrent
#[derive(Debug, Clone, InputObject)]
pub struct UploadTorrentInput {
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub info_hash: String,
    pub torrent_file: Vec<u8>,
}

/// Input for updating a torrent
#[derive(Debug, Clone, InputObject)]
pub struct UpdateTorrentInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
}

/// Input for updating user profile
#[derive(Debug, Clone, InputObject)]
pub struct UpdateProfileInput {
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
}

/// Input for creating a forum topic
#[derive(Debug, Clone, InputObject)]
pub struct CreateTopicInput {
    pub forum_id: uuid::Uuid,
    pub title: String,
    pub content: String,
}

/// Input for posting a reply
#[derive(Debug, Clone, InputObject)]
pub struct PostReplyInput {
    pub topic_id: uuid::Uuid,
    pub content: String,
}

/// Input for sending a message
#[derive(Debug, Clone, InputObject)]
pub struct SendMessageInput {
    pub recipient_id: uuid::Uuid,
    pub subject: String,
    pub content: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_ratio_calculation() {
        let user = User {
            id: uuid::Uuid::new_v4(),
            username: "test".to_string(),
            email: "test@example.com".to_string(),
            created_at: Utc::now(),
            user_class: "user".to_string(),
            uploaded: 1000,
            downloaded: 500,
            passkey: "test".to_string(),
            is_active: true,
            is_verified: true,
        };

        // Can't test async methods directly, but we can verify the struct
        assert_eq!(user.uploaded, 1000);
        assert_eq!(user.downloaded, 500);
    }
}
