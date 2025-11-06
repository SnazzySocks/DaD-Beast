//! Private Messaging
//!
//! Provides private messaging functionality with conversation threading,
//! multiple participants, and attachment support.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;
use validator::Validate;

/// Messaging-related errors
#[derive(Debug, Error)]
pub enum MessagingError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Message not found: {0}")]
    MessageNotFound(Uuid),

    #[error("Conversation not found: {0}")]
    ConversationNotFound(Uuid),

    #[error("Permission denied")]
    PermissionDenied,

    #[error("User not in conversation")]
    NotParticipant,

    #[error("Validation error: {0}")]
    Validation(#[from] validator::ValidationErrors),

    #[error("Cannot send message: {0}")]
    CannotSend(String),

    #[error("User not found: {0}")]
    UserNotFound(Uuid),
}

pub type Result<T> = std::result::Result<T, MessagingError>;

/// Conversation thread
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Conversation {
    pub id: Uuid,
    pub subject: String,
    pub created_by: Uuid,
    pub message_count: i32,
    pub last_message_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Conversation with extended information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationWithDetails {
    #[serde(flatten)]
    pub conversation: Conversation,
    pub participants: Vec<ConversationParticipant>,
    pub unread_count: i32,
    pub last_message_preview: Option<String>,
}

/// Conversation participant
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ConversationParticipant {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub joined_at: DateTime<Utc>,
    pub last_read_at: Option<DateTime<Utc>>,
    pub is_deleted: bool,
}

/// Individual message
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub sender_id: Uuid,
    pub content: String,
    pub content_html: String,

    // Attachments
    pub has_attachments: bool,

    // Status
    pub is_deleted: bool,
    pub deleted_at: Option<DateTime<Utc>>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Message with extended information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageWithDetails {
    #[serde(flatten)]
    pub message: Message,
    pub sender_username: String,
    pub is_read: bool,
}

/// Message attachment
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MessageAttachment {
    pub id: Uuid,
    pub message_id: Uuid,
    pub filename: String,
    pub file_path: String,
    pub file_size: i64,
    pub mime_type: String,
    pub created_at: DateTime<Utc>,
}

/// Read receipt tracking
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MessageRead {
    pub id: Uuid,
    pub message_id: Uuid,
    pub user_id: Uuid,
    pub read_at: DateTime<Utc>,
}

/// Request to send a message
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SendMessageRequest {
    pub sender_id: Uuid,

    #[validate(length(min = 1))]
    pub recipient_ids: Vec<Uuid>,

    #[validate(length(min = 1, max = 200))]
    pub subject: String,

    #[validate(length(min = 1, max = 65535))]
    pub content: String,

    pub conversation_id: Option<Uuid>,
}

/// Request to reply to a conversation
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ReplyMessageRequest {
    pub conversation_id: Uuid,
    pub sender_id: Uuid,

    #[validate(length(min = 1, max = 65535))]
    pub content: String,
}

/// Search parameters for messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSearchParams {
    pub query: String,
    pub page: i32,
    pub per_page: i32,
}

impl Default for MessageSearchParams {
    fn default() -> Self {
        Self {
            query: String::new(),
            page: 1,
            per_page: 25,
        }
    }
}

/// Pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationListParams {
    pub page: i32,
    pub per_page: i32,
    pub unread_only: bool,
}

impl Default for ConversationListParams {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 25,
            unread_only: false,
        }
    }
}

/// Messaging service for private messages
pub struct MessagingService {
    db: PgPool,
}

impl MessagingService {
    /// Creates a new messaging service
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Sends a new message (creates conversation if needed)
    pub async fn send_message(&self, request: SendMessageRequest) -> Result<Message> {
        request.validate()?;

        // Verify all recipients exist
        for recipient_id in &request.recipient_ids {
            let exists = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)",
            )
            .bind(recipient_id)
            .fetch_one(&self.db)
            .await?;

            if !exists {
                return Err(MessagingError::UserNotFound(*recipient_id));
            }
        }

        let mut tx = self.db.begin().await?;

        // Get or create conversation
        let conversation_id = if let Some(conv_id) = request.conversation_id {
            // Verify sender is participant
            let is_participant = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM conversation_participants WHERE conversation_id = $1 AND user_id = $2)",
            )
            .bind(conv_id)
            .bind(request.sender_id)
            .fetch_one(&mut *tx)
            .await?;

            if !is_participant {
                return Err(MessagingError::NotParticipant);
            }

            conv_id
        } else {
            // Create new conversation
            let conv_id = Uuid::new_v4();
            sqlx::query(
                r#"
                INSERT INTO conversations (id, subject, created_by, message_count, created_at, updated_at)
                VALUES ($1, $2, $3, 0, NOW(), NOW())
                "#,
            )
            .bind(conv_id)
            .bind(&request.subject)
            .bind(request.sender_id)
            .execute(&mut *tx)
            .await?;

            // Add sender as participant
            sqlx::query(
                r#"
                INSERT INTO conversation_participants (id, conversation_id, user_id, joined_at, is_deleted)
                VALUES ($1, $2, $3, NOW(), false)
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(conv_id)
            .bind(request.sender_id)
            .execute(&mut *tx)
            .await?;

            // Add recipients as participants
            for recipient_id in &request.recipient_ids {
                if recipient_id != &request.sender_id {
                    sqlx::query(
                        r#"
                        INSERT INTO conversation_participants (id, conversation_id, user_id, joined_at, is_deleted)
                        VALUES ($1, $2, $3, NOW(), false)
                        "#,
                    )
                    .bind(Uuid::new_v4())
                    .bind(conv_id)
                    .bind(recipient_id)
                    .execute(&mut *tx)
                    .await?;
                }
            }

            conv_id
        };

        // Process content
        let content_html = self.process_content(&request.content)?;

        // Create message
        let message = sqlx::query_as::<_, Message>(
            r#"
            INSERT INTO messages (
                id, conversation_id, sender_id, content, content_html,
                has_attachments, is_deleted, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, false, false, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(conversation_id)
        .bind(request.sender_id)
        .bind(&request.content)
        .bind(&content_html)
        .fetch_one(&mut *tx)
        .await?;

        // Update conversation
        sqlx::query(
            r#"
            UPDATE conversations
            SET message_count = message_count + 1,
                last_message_at = NOW(),
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(conversation_id)
        .execute(&mut *tx)
        .await?;

        // Mark as read for sender
        sqlx::query(
            r#"
            INSERT INTO message_reads (id, message_id, user_id, read_at)
            VALUES ($1, $2, $3, NOW())
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(message.id)
        .bind(request.sender_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(message)
    }

    /// Gets a conversation by ID
    pub async fn get_conversation(&self, conversation_id: Uuid) -> Result<Conversation> {
        let conversation = sqlx::query_as::<_, Conversation>(
            "SELECT * FROM conversations WHERE id = $1",
        )
        .bind(conversation_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or(MessagingError::ConversationNotFound(conversation_id))?;

        Ok(conversation)
    }

    /// Gets a conversation with details
    pub async fn get_conversation_with_details(
        &self,
        conversation_id: Uuid,
        user_id: Uuid,
    ) -> Result<ConversationWithDetails> {
        let conversation = self.get_conversation(conversation_id).await?;

        // Verify user is participant
        self.verify_participant(conversation_id, user_id).await?;

        // Get participants
        let participants = sqlx::query_as::<_, ConversationParticipant>(
            r#"
            SELECT cp.id, cp.conversation_id, cp.user_id, u.username,
                   cp.joined_at, cp.last_read_at, cp.is_deleted
            FROM conversation_participants cp
            JOIN users u ON cp.user_id = u.id
            WHERE cp.conversation_id = $1 AND cp.is_deleted = false
            "#,
        )
        .bind(conversation_id)
        .fetch_all(&self.db)
        .await?;

        // Get unread count for user
        let unread_count = self.get_unread_count(conversation_id, user_id).await?;

        // Get last message preview
        let last_message_preview = sqlx::query_scalar::<_, String>(
            r#"
            SELECT LEFT(content, 100) FROM messages
            WHERE conversation_id = $1 AND is_deleted = false
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(conversation_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(ConversationWithDetails {
            conversation,
            participants,
            unread_count,
            last_message_preview,
        })
    }

    /// Lists user's conversations
    pub async fn list_conversations(
        &self,
        user_id: Uuid,
        params: ConversationListParams,
    ) -> Result<(Vec<ConversationWithDetails>, i64)> {
        let offset = (params.page - 1) * params.per_page;

        let conversations = sqlx::query_as::<_, Conversation>(
            r#"
            SELECT c.* FROM conversations c
            JOIN conversation_participants cp ON c.id = cp.conversation_id
            WHERE cp.user_id = $1 AND cp.is_deleted = false
            ORDER BY c.last_message_at DESC NULLS LAST
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(params.per_page)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        let total = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM conversations c
            JOIN conversation_participants cp ON c.id = cp.conversation_id
            WHERE cp.user_id = $1 AND cp.is_deleted = false
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        let mut conversations_with_details = Vec::new();
        for conversation in conversations {
            let details = self
                .get_conversation_with_details(conversation.id, user_id)
                .await?;

            if params.unread_only && details.unread_count == 0 {
                continue;
            }

            conversations_with_details.push(details);
        }

        Ok((conversations_with_details, total))
    }

    /// Gets messages in a conversation
    pub async fn get_messages(
        &self,
        conversation_id: Uuid,
        user_id: Uuid,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<MessageWithDetails>, i64)> {
        // Verify user is participant
        self.verify_participant(conversation_id, user_id).await?;

        let offset = (page - 1) * per_page;

        let messages = sqlx::query_as::<_, Message>(
            r#"
            SELECT * FROM messages
            WHERE conversation_id = $1 AND is_deleted = false
            ORDER BY created_at ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(conversation_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM messages WHERE conversation_id = $1 AND is_deleted = false",
        )
        .bind(conversation_id)
        .fetch_one(&self.db)
        .await?;

        let mut messages_with_details = Vec::new();
        for message in messages {
            let sender_username = sqlx::query_scalar::<_, String>(
                "SELECT username FROM users WHERE id = $1",
            )
            .bind(message.sender_id)
            .fetch_one(&self.db)
            .await?;

            let is_read = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM message_reads WHERE message_id = $1 AND user_id = $2)",
            )
            .bind(message.id)
            .bind(user_id)
            .fetch_one(&self.db)
            .await?;

            messages_with_details.push(MessageWithDetails {
                message,
                sender_username,
                is_read,
            });
        }

        Ok((messages_with_details, total))
    }

    /// Marks message as read
    pub async fn mark_as_read(&self, message_id: Uuid, user_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO message_reads (id, message_id, user_id, read_at)
            VALUES ($1, $2, $3, NOW())
            ON CONFLICT (message_id, user_id) DO NOTHING
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(message_id)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        // Update last_read_at for participant
        let conversation_id = sqlx::query_scalar::<_, Uuid>(
            "SELECT conversation_id FROM messages WHERE id = $1",
        )
        .bind(message_id)
        .fetch_one(&self.db)
        .await?;

        sqlx::query(
            r#"
            UPDATE conversation_participants
            SET last_read_at = NOW()
            WHERE conversation_id = $1 AND user_id = $2
            "#,
        )
        .bind(conversation_id)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Marks all messages in a conversation as read
    pub async fn mark_conversation_as_read(&self, conversation_id: Uuid, user_id: Uuid) -> Result<()> {
        self.verify_participant(conversation_id, user_id).await?;

        let message_ids = sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM messages WHERE conversation_id = $1 AND is_deleted = false",
        )
        .bind(conversation_id)
        .fetch_all(&self.db)
        .await?;

        for message_id in message_ids {
            self.mark_as_read(message_id, user_id).await?;
        }

        Ok(())
    }

    /// Deletes a conversation for a user (soft delete)
    pub async fn delete_conversation(&self, conversation_id: Uuid, user_id: Uuid) -> Result<()> {
        self.verify_participant(conversation_id, user_id).await?;

        sqlx::query(
            r#"
            UPDATE conversation_participants
            SET is_deleted = true
            WHERE conversation_id = $1 AND user_id = $2
            "#,
        )
        .bind(conversation_id)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Searches messages
    pub async fn search_messages(
        &self,
        user_id: Uuid,
        params: MessageSearchParams,
    ) -> Result<(Vec<MessageWithDetails>, i64)> {
        let offset = (params.page - 1) * params.per_page;
        let search_term = format!("%{}%", params.query);

        let messages = sqlx::query_as::<_, Message>(
            r#"
            SELECT DISTINCT m.* FROM messages m
            JOIN conversation_participants cp ON m.conversation_id = cp.conversation_id
            WHERE cp.user_id = $1 AND cp.is_deleted = false
              AND m.is_deleted = false
              AND m.content ILIKE $2
            ORDER BY m.created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(user_id)
        .bind(&search_term)
        .bind(params.per_page)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        let total = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(DISTINCT m.id) FROM messages m
            JOIN conversation_participants cp ON m.conversation_id = cp.conversation_id
            WHERE cp.user_id = $1 AND cp.is_deleted = false
              AND m.is_deleted = false
              AND m.content ILIKE $2
            "#,
        )
        .bind(user_id)
        .bind(&search_term)
        .fetch_one(&self.db)
        .await?;

        let mut messages_with_details = Vec::new();
        for message in messages {
            let sender_username = sqlx::query_scalar::<_, String>(
                "SELECT username FROM users WHERE id = $1",
            )
            .bind(message.sender_id)
            .fetch_one(&self.db)
            .await?;

            let is_read = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM message_reads WHERE message_id = $1 AND user_id = $2)",
            )
            .bind(message.id)
            .bind(user_id)
            .fetch_one(&self.db)
            .await?;

            messages_with_details.push(MessageWithDetails {
                message,
                sender_username,
                is_read,
            });
        }

        Ok((messages_with_details, total))
    }

    /// Gets unread message count for a user
    pub async fn get_total_unread_count(&self, user_id: Uuid) -> Result<i64> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM messages m
            JOIN conversation_participants cp ON m.conversation_id = cp.conversation_id
            WHERE cp.user_id = $1 AND cp.is_deleted = false
              AND m.sender_id != $1
              AND m.is_deleted = false
              AND NOT EXISTS (
                  SELECT 1 FROM message_reads
                  WHERE message_id = m.id AND user_id = $1
              )
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        Ok(count)
    }

    /// Gets unread count for a specific conversation
    async fn get_unread_count(&self, conversation_id: Uuid, user_id: Uuid) -> Result<i32> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM messages m
            WHERE m.conversation_id = $1
              AND m.sender_id != $2
              AND m.is_deleted = false
              AND NOT EXISTS (
                  SELECT 1 FROM message_reads
                  WHERE message_id = m.id AND user_id = $2
              )
            "#,
        )
        .bind(conversation_id)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        Ok(count as i32)
    }

    /// Verifies user is participant in conversation
    async fn verify_participant(&self, conversation_id: Uuid, user_id: Uuid) -> Result<()> {
        let is_participant = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM conversation_participants WHERE conversation_id = $1 AND user_id = $2 AND is_deleted = false)",
        )
        .bind(conversation_id)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        if !is_participant {
            return Err(MessagingError::NotParticipant);
        }

        Ok(())
    }

    /// Processes message content
    fn process_content(&self, content: &str) -> Result<String> {
        // Basic HTML sanitization
        Ok(ammonia::clean(content))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_message_validation() {
        let request = SendMessageRequest {
            sender_id: Uuid::new_v4(),
            recipient_ids: vec![Uuid::new_v4()],
            subject: "Test Subject".to_string(),
            content: "Test message content.".to_string(),
            conversation_id: None,
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_conversation_list_params_default() {
        let params = ConversationListParams::default();
        assert_eq!(params.page, 1);
        assert_eq!(params.per_page, 25);
        assert!(!params.unread_only);
    }
}
