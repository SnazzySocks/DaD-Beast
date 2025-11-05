//! Community and Social Features
//!
//! This crate provides comprehensive community functionality for the unified
//! tracker platform, including forums, messaging, chat, and social features
//! inspired by Gazelle and Unit3d tracker systems.
//!
//! # Features
//!
//! - **Forums**: Full-featured forum system with categories, topics, and posts
//! - **Private Messaging**: Direct messaging with conversation threading
//! - **Real-time Chat**: WebSocket-based chat rooms with presence
//! - **Wiki**: Knowledge base with versioned pages
//! - **Polls**: Voting system with multiple choice support
//! - **Events**: Calendar system with RSVP and reminders
//!
//! # Architecture
//!
//! The community crate follows a layered architecture:
//!
//! 1. **Discussion Layer** (`forums`, `topics`, `posts`)
//!    - Forum categories and permissions
//!    - Topic creation and management
//!    - Post creation with BBCode/Markdown support
//!
//! 2. **Communication Layer** (`messaging`, `chat`)
//!    - Private messaging with threads
//!    - Real-time chat with WebSocket integration
//!    - User presence tracking
//!
//! 3. **Content Layer** (`wiki`, `polls`, `events`)
//!    - Wiki pages with version history
//!    - Polls and voting
//!    - Events calendar with RSVP
//!
//! # Quick Start
//!
//! ## Managing Forums
//!
//! ```rust,no_run
//! use community::forums::{ForumService, CreateForumRequest};
//! use uuid::Uuid;
//!
//! # async fn example(db_pool: sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
//! let forum_service = ForumService::new(db_pool);
//!
//! // Create a forum
//! let forum = forum_service.create_forum(CreateForumRequest {
//!     name: "General Discussion".to_string(),
//!     description: Some("General chat about anything".to_string()),
//!     category_id: Uuid::new_v4(),
//!     parent_id: None,
//!     icon: Some("💬".to_string()),
//!     sort_order: 0,
//!     min_class_read: 0,
//!     min_class_write: 0,
//! }).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Creating Topics and Posts
//!
//! ```rust,no_run
//! use community::topics::{TopicService, CreateTopicRequest};
//! use community::posts::{PostService, CreatePostRequest};
//!
//! # async fn example(db_pool: sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
//! let topic_service = TopicService::new(db_pool.clone());
//! let post_service = PostService::new(db_pool);
//!
//! // Create a topic
//! let topic = topic_service.create_topic(CreateTopicRequest {
//!     forum_id: uuid::Uuid::new_v4(),
//!     user_id: uuid::Uuid::new_v4(),
//!     title: "Welcome to the community!".to_string(),
//!     content: "Let's discuss tracker features".to_string(),
//! }).await?;
//!
//! // Add a reply
//! let post = post_service.create_post(CreatePostRequest {
//!     topic_id: topic.id,
//!     user_id: uuid::Uuid::new_v4(),
//!     content: "Great to be here!".to_string(),
//!     quoted_post_id: None,
//! }).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Private Messaging
//!
//! ```rust,no_run
//! use community::messaging::{MessagingService, SendMessageRequest};
//!
//! # async fn example(db_pool: sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
//! let messaging_service = MessagingService::new(db_pool);
//!
//! // Send a message
//! let message = messaging_service.send_message(SendMessageRequest {
//!     sender_id: uuid::Uuid::new_v4(),
//!     recipient_ids: vec![uuid::Uuid::new_v4()],
//!     subject: "Hello!".to_string(),
//!     content: "How are you?".to_string(),
//!     conversation_id: None,
//! }).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Real-time Chat
//!
//! ```rust,no_run
//! use community::chat::{ChatService, CreateRoomRequest};
//!
//! # async fn example(db_pool: sqlx::PgPool, redis: redis::Client) -> Result<(), Box<dyn std::error::Error>> {
//! let chat_service = ChatService::new(db_pool, redis);
//!
//! // Create a chat room
//! let room = chat_service.create_room(CreateRoomRequest {
//!     name: "General Chat".to_string(),
//!     description: Some("Community chat room".to_string()),
//!     is_public: true,
//!     min_class: 0,
//! }).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Database Schema
//!
//! This crate expects the following database tables:
//!
//! - `forum_categories`: Top-level forum organization
//! - `forums`: Individual forums with permissions
//! - `topics`: Discussion threads
//! - `posts`: Individual forum posts
//! - `post_edits`: Edit history for posts
//! - `conversations`: Private message threads
//! - `messages`: Individual private messages
//! - `chat_rooms`: Chat room definitions
//! - `chat_messages`: Chat message history
//! - `wiki_pages`: Wiki page content
//! - `wiki_revisions`: Page version history
//! - `polls`: Poll definitions
//! - `poll_options`: Poll choices
//! - `poll_votes`: User votes
//! - `events`: Calendar events
//! - `event_rsvps`: Event attendance tracking

// Re-export commonly used types
pub use uuid::Uuid;

// Module declarations
pub mod chat;
pub mod events;
pub mod forums;
pub mod messaging;
pub mod polls;
pub mod posts;
pub mod topics;
pub mod wiki;

// Re-export key types for convenience
pub use chat::{
    ChatError, ChatMessage, ChatPresence, ChatRoom, ChatService, CreateRoomRequest, SendChatMessageRequest,
};
pub use events::{
    CreateEventRequest, Event, EventError, EventRecurrence, EventRsvp, EventService, RsvpStatus,
};
pub use forums::{
    CreateForumRequest, Forum, ForumCategory, ForumError, ForumPermissions, ForumService,
};
pub use messaging::{
    Conversation, Message, MessagingError, MessagingService, SendMessageRequest,
};
pub use polls::{
    CreatePollRequest, Poll, PollError, PollOption, PollService, PollVote, VoteChangePolicy,
};
pub use posts::{
    CreatePostRequest, Post, PostEdit, PostError, PostReaction, PostService, ReactionType,
};
pub use topics::{
    CreateTopicRequest, Topic, TopicError, TopicService, TopicStatus, TopicSubscription,
};
pub use wiki::{
    CreateWikiPageRequest, WikiError, WikiPage, WikiRevision, WikiService,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Community module for complete community functionality
///
/// This is a convenience re-export that groups all community-related
/// functionality in one place.
pub mod community {
    pub use crate::chat::*;
    pub use crate::events::*;
    pub use crate::forums::*;
    pub use crate::messaging::*;
    pub use crate::polls::*;
    pub use crate::posts::*;
    pub use crate::topics::*;
    pub use crate::wiki::*;
}

/// Unified community service providing access to all community features
///
/// This service aggregates all community functionality into a single
/// interface for easier usage and dependency injection.
pub struct CommunityService {
    pub forums: ForumService,
    pub topics: TopicService,
    pub posts: PostService,
    pub messaging: MessagingService,
    pub chat: ChatService,
    pub wiki: WikiService,
    pub polls: PollService,
    pub events: EventService,
}

impl CommunityService {
    /// Creates a new community service with all sub-services
    pub fn new(db_pool: sqlx::PgPool, redis_client: redis::Client) -> Self {
        Self {
            forums: ForumService::new(db_pool.clone()),
            topics: TopicService::new(db_pool.clone()),
            posts: PostService::new(db_pool.clone()),
            messaging: MessagingService::new(db_pool.clone()),
            chat: ChatService::new(db_pool.clone(), redis_client),
            wiki: WikiService::new(db_pool.clone()),
            polls: PollService::new(db_pool.clone()),
            events: EventService::new(db_pool),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_exports() {
        // Verify key types are exported
        let _: Result<(), ForumError> = Ok(());
        let _: Result<(), TopicError> = Ok(());
        let _: Result<(), PostError> = Ok(());
        let _: Result<(), MessagingError> = Ok(());
        let _: Result<(), ChatError> = Ok(());
        let _: Result<(), WikiError> = Ok(());
        let _: Result<(), PollError> = Ok(());
        let _: Result<(), EventError> = Ok(());
    }
}
