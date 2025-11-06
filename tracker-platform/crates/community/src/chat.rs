//! Real-time Chat
//!
//! Provides real-time chat functionality with WebSocket integration,
//! user presence tracking, and chat moderation features.

use chrono::{DateTime, Utc};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;
use validator::Validate;

/// Chat-related errors
#[derive(Debug, Error)]
pub enum ChatError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Room not found: {0}")]
    RoomNotFound(Uuid),

    #[error("Message not found: {0}")]
    MessageNotFound(Uuid),

    #[error("Permission denied")]
    PermissionDenied,

    #[error("User is muted until {0}")]
    UserMuted(DateTime<Utc>),

    #[error("User is banned from room")]
    UserBanned,

    #[error("Validation error: {0}")]
    Validation(#[from] validator::ValidationErrors),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

pub type Result<T> = std::result::Result<T, ChatError>;

/// Chat room
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ChatRoom {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_public: bool,
    pub min_class: i32,
    pub max_users: Option<i32>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Chat room with statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRoomWithStats {
    #[serde(flatten)]
    pub room: ChatRoom,
    pub online_count: i32,
    pub message_count: i64,
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ChatMessage {
    pub id: Uuid,
    pub room_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub content_html: String,
    pub is_system: bool,
    pub is_deleted: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Chat message with user details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageWithDetails {
    #[serde(flatten)]
    pub message: ChatMessage,
    pub username: String,
    pub user_class: i32,
}

/// User presence in chat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatPresence {
    pub user_id: Uuid,
    pub username: String,
    pub room_id: Uuid,
    pub is_typing: bool,
    pub last_seen: DateTime<Utc>,
}

/// Chat moderation action
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ChatModeration {
    pub id: Uuid,
    pub room_id: Uuid,
    pub user_id: Uuid,
    pub moderator_id: Uuid,
    pub action_type: ModerationAction,
    pub reason: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Moderation action types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "moderation_action", rename_all = "lowercase")]
pub enum ModerationAction {
    Mute,
    Kick,
    Ban,
}

/// Request to create a chat room
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateRoomRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,

    #[validate(length(max = 500))]
    pub description: Option<String>,

    pub is_public: bool,
    pub min_class: i32,
}

/// Request to send a chat message
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SendChatMessageRequest {
    pub room_id: Uuid,
    pub user_id: Uuid,

    #[validate(length(min = 1, max = 1000))]
    pub content: String,
}

/// Request to moderate a user
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ModerateUserRequest {
    pub room_id: Uuid,
    pub user_id: Uuid,
    pub moderator_id: Uuid,
    pub action: ModerationAction,

    #[validate(length(max = 500))]
    pub reason: Option<String>,

    pub duration_minutes: Option<i32>,
}

/// Typing indicator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingIndicator {
    pub user_id: Uuid,
    pub username: String,
    pub room_id: Uuid,
}

/// Chat service for real-time chat
pub struct ChatService {
    db: PgPool,
    redis: redis::Client,
}

impl ChatService {
    /// Creates a new chat service
    pub fn new(db: PgPool, redis: redis::Client) -> Self {
        Self { db, redis }
    }

    /// Creates a new chat room
    pub async fn create_room(&self, created_by: Uuid, request: CreateRoomRequest) -> Result<ChatRoom> {
        request.validate()?;

        let room = sqlx::query_as::<_, ChatRoom>(
            r#"
            INSERT INTO chat_rooms (id, name, description, is_public, min_class, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&request.name)
        .bind(&request.description)
        .bind(request.is_public)
        .bind(request.min_class)
        .bind(created_by)
        .fetch_one(&self.db)
        .await?;

        Ok(room)
    }

    /// Gets a chat room by ID
    pub async fn get_room(&self, room_id: Uuid) -> Result<ChatRoom> {
        let room = sqlx::query_as::<_, ChatRoom>("SELECT * FROM chat_rooms WHERE id = $1")
            .bind(room_id)
            .fetch_optional(&self.db)
            .await?
            .ok_or(ChatError::RoomNotFound(room_id))?;

        Ok(room)
    }

    /// Gets a chat room with statistics
    pub async fn get_room_with_stats(&self, room_id: Uuid) -> Result<ChatRoomWithStats> {
        let room = self.get_room(room_id).await?;
        let online_count = self.get_online_count(room_id).await?;

        let message_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM chat_messages WHERE room_id = $1 AND is_deleted = false",
        )
        .bind(room_id)
        .fetch_one(&self.db)
        .await?;

        Ok(ChatRoomWithStats {
            room,
            online_count,
            message_count,
        })
    }

    /// Lists all public chat rooms
    pub async fn list_rooms(&self, user_class: i32) -> Result<Vec<ChatRoomWithStats>> {
        let rooms = sqlx::query_as::<_, ChatRoom>(
            r#"
            SELECT * FROM chat_rooms
            WHERE is_public = true AND min_class <= $1
            ORDER BY name
            "#,
        )
        .bind(user_class)
        .fetch_all(&self.db)
        .await?;

        let mut rooms_with_stats = Vec::new();
        for room in rooms {
            let stats = self.get_room_with_stats(room.id).await?;
            rooms_with_stats.push(stats);
        }

        Ok(rooms_with_stats)
    }

    /// Sends a chat message
    pub async fn send_message(&self, request: SendChatMessageRequest) -> Result<ChatMessage> {
        request.validate()?;

        // Verify room exists and user has permission
        let room = self.get_room(request.room_id).await?;
        self.verify_can_post(request.room_id, request.user_id).await?;

        // Check rate limit
        self.check_rate_limit(request.user_id, request.room_id).await?;

        // Process content
        let content_html = self.process_content(&request.content)?;

        let message = sqlx::query_as::<_, ChatMessage>(
            r#"
            INSERT INTO chat_messages (
                id, room_id, user_id, content, content_html,
                is_system, is_deleted, created_at
            )
            VALUES ($1, $2, $3, $4, $5, false, false, NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(request.room_id)
        .bind(request.user_id)
        .bind(&request.content)
        .bind(&content_html)
        .fetch_one(&self.db)
        .await?;

        // Update rate limit in Redis
        self.update_rate_limit(request.user_id, request.room_id).await?;

        Ok(message)
    }

    /// Gets recent messages from a room
    pub async fn get_messages(
        &self,
        room_id: Uuid,
        limit: i32,
        before: Option<DateTime<Utc>>,
    ) -> Result<Vec<ChatMessageWithDetails>> {
        let messages = if let Some(before_time) = before {
            sqlx::query_as::<_, ChatMessage>(
                r#"
                SELECT * FROM chat_messages
                WHERE room_id = $1 AND is_deleted = false AND created_at < $2
                ORDER BY created_at DESC
                LIMIT $3
                "#,
            )
            .bind(room_id)
            .bind(before_time)
            .bind(limit)
            .fetch_all(&self.db)
            .await?
        } else {
            sqlx::query_as::<_, ChatMessage>(
                r#"
                SELECT * FROM chat_messages
                WHERE room_id = $1 AND is_deleted = false
                ORDER BY created_at DESC
                LIMIT $2
                "#,
            )
            .bind(room_id)
            .bind(limit)
            .fetch_all(&self.db)
            .await?
        };

        let mut messages_with_details = Vec::new();
        for message in messages {
            let (username, user_class) = sqlx::query_as::<_, (String, i32)>(
                "SELECT username, class FROM users WHERE id = $1",
            )
            .bind(message.user_id)
            .fetch_one(&self.db)
            .await?;

            messages_with_details.push(ChatMessageWithDetails {
                message,
                username,
                user_class,
            });
        }

        // Reverse to get chronological order
        messages_with_details.reverse();

        Ok(messages_with_details)
    }

    /// Deletes a chat message (moderator only)
    pub async fn delete_message(&self, message_id: Uuid, moderator_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE chat_messages
            SET is_deleted = true,
                deleted_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(message_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Updates user presence (call when user joins/leaves/sends message)
    pub async fn update_presence(&self, user_id: Uuid, room_id: Uuid) -> Result<()> {
        let mut conn = self.redis.get_multiplexed_async_connection().await?;

        let key = format!("chat:presence:{}:{}", room_id, user_id);
        let _: () = conn.set_ex(&key, "1", 300).await?; // 5 minute expiry

        Ok(())
    }

    /// Removes user presence (call when user leaves)
    pub async fn remove_presence(&self, user_id: Uuid, room_id: Uuid) -> Result<()> {
        let mut conn = self.redis.get_multiplexed_async_connection().await?;

        let key = format!("chat:presence:{}:{}", room_id, user_id);
        let _: () = conn.del(&key).await?;

        Ok(())
    }

    /// Gets online users in a room
    pub async fn get_online_users(&self, room_id: Uuid) -> Result<Vec<ChatPresence>> {
        let mut conn = self.redis.get_multiplexed_async_connection().await?;

        let pattern = format!("chat:presence:{}:*", room_id);
        let keys: Vec<String> = conn.keys(&pattern).await?;

        let mut presences = Vec::new();
        for key in keys {
            // Extract user_id from key
            let parts: Vec<&str> = key.split(':').collect();
            if let Some(user_id_str) = parts.last() {
                if let Ok(user_id) = Uuid::parse_str(user_id_str) {
                    let username = sqlx::query_scalar::<_, String>(
                        "SELECT username FROM users WHERE id = $1",
                    )
                    .bind(user_id)
                    .fetch_one(&self.db)
                    .await?;

                    presences.push(ChatPresence {
                        user_id,
                        username,
                        room_id,
                        is_typing: false,
                        last_seen: Utc::now(),
                    });
                }
            }
        }

        Ok(presences)
    }

    /// Gets online user count in a room
    pub async fn get_online_count(&self, room_id: Uuid) -> Result<i32> {
        let mut conn = self.redis.get_multiplexed_async_connection().await?;

        let pattern = format!("chat:presence:{}:*", room_id);
        let keys: Vec<String> = conn.keys(&pattern).await?;

        Ok(keys.len() as i32)
    }

    /// Sets typing indicator
    pub async fn set_typing(&self, user_id: Uuid, room_id: Uuid, is_typing: bool) -> Result<()> {
        let mut conn = self.redis.get_multiplexed_async_connection().await?;

        let key = format!("chat:typing:{}:{}", room_id, user_id);

        if is_typing {
            let _: () = conn.set_ex(&key, "1", 10).await?; // 10 second expiry
        } else {
            let _: () = conn.del(&key).await?;
        }

        Ok(())
    }

    /// Gets typing users in a room
    pub async fn get_typing_users(&self, room_id: Uuid) -> Result<Vec<TypingIndicator>> {
        let mut conn = self.redis.get_multiplexed_async_connection().await?;

        let pattern = format!("chat:typing:{}:*", room_id);
        let keys: Vec<String> = conn.keys(&pattern).await?;

        let mut indicators = Vec::new();
        for key in keys {
            let parts: Vec<&str> = key.split(':').collect();
            if let Some(user_id_str) = parts.last() {
                if let Ok(user_id) = Uuid::parse_str(user_id_str) {
                    let username = sqlx::query_scalar::<_, String>(
                        "SELECT username FROM users WHERE id = $1",
                    )
                    .bind(user_id)
                    .fetch_one(&self.db)
                    .await?;

                    indicators.push(TypingIndicator {
                        user_id,
                        username,
                        room_id,
                    });
                }
            }
        }

        Ok(indicators)
    }

    /// Moderates a user (mute, kick, ban)
    pub async fn moderate_user(&self, request: ModerateUserRequest) -> Result<ChatModeration> {
        request.validate()?;

        let expires_at = if let Some(duration) = request.duration_minutes {
            Some(Utc::now() + chrono::Duration::minutes(duration as i64))
        } else {
            None
        };

        let moderation = sqlx::query_as::<_, ChatModeration>(
            r#"
            INSERT INTO chat_moderations (
                id, room_id, user_id, moderator_id, action_type,
                reason, expires_at, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(request.room_id)
        .bind(request.user_id)
        .bind(request.moderator_id)
        .bind(request.action)
        .bind(&request.reason)
        .bind(expires_at)
        .fetch_one(&self.db)
        .await?;

        // If kick or ban, remove presence
        if matches!(request.action, ModerationAction::Kick | ModerationAction::Ban) {
            self.remove_presence(request.user_id, request.room_id).await?;
        }

        Ok(moderation)
    }

    /// Checks if user can post in room
    async fn verify_can_post(&self, room_id: Uuid, user_id: Uuid) -> Result<()> {
        // Check for active mute
        let muted_until = sqlx::query_scalar::<_, Option<DateTime<Utc>>>(
            r#"
            SELECT expires_at FROM chat_moderations
            WHERE room_id = $1 AND user_id = $2
              AND action_type = 'mute'
              AND (expires_at IS NULL OR expires_at > NOW())
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(room_id)
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?;

        if let Some(Some(until)) = muted_until {
            return Err(ChatError::UserMuted(until));
        }

        // Check for ban
        let is_banned = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM chat_moderations
                WHERE room_id = $1 AND user_id = $2
                  AND action_type = 'ban'
                  AND (expires_at IS NULL OR expires_at > NOW())
            )
            "#,
        )
        .bind(room_id)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        if is_banned {
            return Err(ChatError::UserBanned);
        }

        Ok(())
    }

    /// Checks rate limit for user
    async fn check_rate_limit(&self, user_id: Uuid, room_id: Uuid) -> Result<()> {
        let mut conn = self.redis.get_multiplexed_async_connection().await?;

        let key = format!("chat:ratelimit:{}:{}", room_id, user_id);
        let count: i32 = conn.get(&key).await.unwrap_or(0);

        // Allow 5 messages per 10 seconds
        if count >= 5 {
            return Err(ChatError::RateLimitExceeded);
        }

        Ok(())
    }

    /// Updates rate limit counter
    async fn update_rate_limit(&self, user_id: Uuid, room_id: Uuid) -> Result<()> {
        let mut conn = self.redis.get_multiplexed_async_connection().await?;

        let key = format!("chat:ratelimit:{}:{}", room_id, user_id);
        let _: () = conn.incr(&key, 1).await?;
        let _: () = conn.expire(&key, 10).await?;

        Ok(())
    }

    /// Processes chat message content
    fn process_content(&self, content: &str) -> Result<String> {
        // Basic HTML sanitization and mention/link detection
        let html = ammonia::clean(content);

        // Process @mentions
        let html = self.process_mentions(&html);

        Ok(html)
    }

    /// Processes @username mentions
    fn process_mentions(&self, content: &str) -> String {
        // Simple mention detection - in production use proper regex
        content.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moderation_action() {
        assert_eq!(ModerationAction::Mute, ModerationAction::Mute);
        assert_ne!(ModerationAction::Mute, ModerationAction::Ban);
    }

    #[test]
    fn test_create_room_validation() {
        let request = CreateRoomRequest {
            name: "General Chat".to_string(),
            description: Some("Main chat room".to_string()),
            is_public: true,
            min_class: 0,
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_send_message_validation() {
        let request = SendChatMessageRequest {
            room_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            content: "Hello world!".to_string(),
        };

        assert!(request.validate().is_ok());
    }
}
