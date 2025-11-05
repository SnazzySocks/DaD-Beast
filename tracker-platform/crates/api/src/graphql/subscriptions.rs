//! # GraphQL Subscriptions
//!
//! Subscription resolvers for real-time updates via WebSocket.

use async_graphql::Result;
use async_stream::stream;
use futures_util::Stream;
use redis::AsyncCommands;
use std::pin::Pin;
use std::time::Duration;
use tracing::instrument;

use super::{types::*, GraphQLContext};

/// Subscribe to new torrents being added
///
/// Clients can optionally filter by category
#[instrument(skip(ctx))]
pub async fn subscribe_torrent_added(
    ctx: &GraphQLContext,
    category: Option<String>,
) -> Result<Pin<Box<dyn Stream<Item = Torrent> + Send>>> {
    // Get Redis pub/sub connection
    let client = ctx.redis_client.clone();
    let db_pool = ctx.db_pool.clone();

    let stream = stream! {
        let mut pubsub = match client.get_async_connection().await {
            Ok(conn) => redis::aio::PubSub::from_connection(conn).await,
            Err(e) => {
                tracing::error!("Failed to get Redis connection: {}", e);
                return;
            }
        };

        // Subscribe to torrent events
        if let Err(e) = pubsub.subscribe("torrent:added").await {
            tracing::error!("Failed to subscribe to Redis channel: {}", e);
            return;
        }

        let mut stream = pubsub.on_message();

        while let Some(msg) = stream.next().await {
            // Parse torrent ID from message
            let payload: String = match msg.get_payload() {
                Ok(p) => p,
                Err(e) => {
                    tracing::error!("Failed to parse message payload: {}", e);
                    continue;
                }
            };

            let torrent_id = match uuid::Uuid::parse_str(&payload) {
                Ok(id) => id,
                Err(e) => {
                    tracing::error!("Failed to parse torrent ID: {}", e);
                    continue;
                }
            };

            // Fetch torrent from database
            let torrent = match sqlx::query_as::<_, Torrent>(
                "SELECT * FROM torrents WHERE id = $1"
            )
            .bind(torrent_id)
            .fetch_optional(&db_pool)
            .await
            {
                Ok(Some(t)) => t,
                Ok(None) => {
                    tracing::warn!("Torrent not found: {}", torrent_id);
                    continue;
                }
                Err(e) => {
                    tracing::error!("Failed to fetch torrent: {}", e);
                    continue;
                }
            };

            // Filter by category if specified
            if let Some(ref cat) = category {
                if &torrent.category != cat {
                    continue;
                }
            }

            yield torrent;
        }
    };

    Ok(Box::pin(stream))
}

/// Subscribe to new messages for the current user
#[instrument(skip(ctx))]
pub async fn subscribe_message_received(
    ctx: &GraphQLContext,
) -> Result<Pin<Box<dyn Stream<Item = Message> + Send>>> {
    let user_id = ctx.require_auth()?;
    let client = ctx.redis_client.clone();
    let db_pool = ctx.db_pool.clone();

    let stream = stream! {
        let mut pubsub = match client.get_async_connection().await {
            Ok(conn) => redis::aio::PubSub::from_connection(conn).await,
            Err(e) => {
                tracing::error!("Failed to get Redis connection: {}", e);
                return;
            }
        };

        // Subscribe to user-specific message channel
        let channel = format!("user:{}:messages", user_id);
        if let Err(e) = pubsub.subscribe(&channel).await {
            tracing::error!("Failed to subscribe to Redis channel: {}", e);
            return;
        }

        let mut stream = pubsub.on_message();

        while let Some(msg) = stream.next().await {
            // Parse message ID from payload
            let payload: String = match msg.get_payload() {
                Ok(p) => p,
                Err(e) => {
                    tracing::error!("Failed to parse message payload: {}", e);
                    continue;
                }
            };

            let message_id = match uuid::Uuid::parse_str(&payload) {
                Ok(id) => id,
                Err(e) => {
                    tracing::error!("Failed to parse message ID: {}", e);
                    continue;
                }
            };

            // Fetch message from database
            let message = match sqlx::query_as::<_, Message>(
                "SELECT * FROM messages WHERE id = $1"
            )
            .bind(message_id)
            .fetch_optional(&db_pool)
            .await
            {
                Ok(Some(m)) => m,
                Ok(None) => {
                    tracing::warn!("Message not found: {}", message_id);
                    continue;
                }
                Err(e) => {
                    tracing::error!("Failed to fetch message: {}", e);
                    continue;
                }
            };

            yield message;
        }
    };

    Ok(Box::pin(stream))
}

/// Subscribe to updates for a specific torrent
#[instrument(skip(ctx))]
pub async fn subscribe_torrent_updated(
    ctx: &GraphQLContext,
    torrent_id: uuid::Uuid,
) -> Result<Pin<Box<dyn Stream<Item = Torrent> + Send>>> {
    let client = ctx.redis_client.clone();
    let db_pool = ctx.db_pool.clone();

    let stream = stream! {
        let mut pubsub = match client.get_async_connection().await {
            Ok(conn) => redis::aio::PubSub::from_connection(conn).await,
            Err(e) => {
                tracing::error!("Failed to get Redis connection: {}", e);
                return;
            }
        };

        // Subscribe to torrent-specific update channel
        let channel = format!("torrent:{}:updated", torrent_id);
        if let Err(e) = pubsub.subscribe(&channel).await {
            tracing::error!("Failed to subscribe to Redis channel: {}", e);
            return;
        }

        let mut stream = pubsub.on_message();

        while let Some(_msg) = stream.next().await {
            // Fetch latest torrent data
            let torrent = match sqlx::query_as::<_, Torrent>(
                "SELECT * FROM torrents WHERE id = $1"
            )
            .bind(torrent_id)
            .fetch_optional(&db_pool)
            .await
            {
                Ok(Some(t)) => t,
                Ok(None) => {
                    tracing::warn!("Torrent not found: {}", torrent_id);
                    return;
                }
                Err(e) => {
                    tracing::error!("Failed to fetch torrent: {}", e);
                    continue;
                }
            };

            yield torrent;
        }
    };

    Ok(Box::pin(stream))
}

/// Subscribe to new posts in a topic
#[instrument(skip(ctx))]
pub async fn subscribe_topic_posts(
    ctx: &GraphQLContext,
    topic_id: uuid::Uuid,
) -> Result<Pin<Box<dyn Stream<Item = Post> + Send>>> {
    let client = ctx.redis_client.clone();
    let db_pool = ctx.db_pool.clone();

    let stream = stream! {
        let mut pubsub = match client.get_async_connection().await {
            Ok(conn) => redis::aio::PubSub::from_connection(conn).await,
            Err(e) => {
                tracing::error!("Failed to get Redis connection: {}", e);
                return;
            }
        };

        // Subscribe to topic-specific posts channel
        let channel = format!("topic:{}:posts", topic_id);
        if let Err(e) = pubsub.subscribe(&channel).await {
            tracing::error!("Failed to subscribe to Redis channel: {}", e);
            return;
        }

        let mut stream = pubsub.on_message();

        while let Some(msg) = stream.next().await {
            // Parse post ID from payload
            let payload: String = match msg.get_payload() {
                Ok(p) => p,
                Err(e) => {
                    tracing::error!("Failed to parse message payload: {}", e);
                    continue;
                }
            };

            let post_id = match uuid::Uuid::parse_str(&payload) {
                Ok(id) => id,
                Err(e) => {
                    tracing::error!("Failed to parse post ID: {}", e);
                    continue;
                }
            };

            // Fetch post from database
            let post = match sqlx::query_as::<_, Post>(
                "SELECT * FROM posts WHERE id = $1"
            )
            .bind(post_id)
            .fetch_optional(&db_pool)
            .await
            {
                Ok(Some(p)) => p,
                Ok(None) => {
                    tracing::warn!("Post not found: {}", post_id);
                    continue;
                }
                Err(e) => {
                    tracing::error!("Failed to fetch post: {}", e);
                    continue;
                }
            };

            yield post;
        }
    };

    Ok(Box::pin(stream))
}

/// Helper function to publish events to Redis
///
/// This would be called from mutations to trigger subscription updates
#[instrument(skip(redis_client))]
pub async fn publish_torrent_added(
    redis_client: &redis::Client,
    torrent_id: uuid::Uuid,
) -> Result<()> {
    let mut conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| async_graphql::Error::new(format!("Redis connection error: {}", e)))?;

    conn.publish("torrent:added", torrent_id.to_string())
        .await
        .map_err(|e| async_graphql::Error::new(format!("Redis publish error: {}", e)))?;

    Ok(())
}

/// Publish message received event
#[instrument(skip(redis_client))]
pub async fn publish_message_received(
    redis_client: &redis::Client,
    user_id: uuid::Uuid,
    message_id: uuid::Uuid,
) -> Result<()> {
    let mut conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| async_graphql::Error::new(format!("Redis connection error: {}", e)))?;

    let channel = format!("user:{}:messages", user_id);
    conn.publish(&channel, message_id.to_string())
        .await
        .map_err(|e| async_graphql::Error::new(format!("Redis publish error: {}", e)))?;

    Ok(())
}

/// Publish torrent updated event
#[instrument(skip(redis_client))]
pub async fn publish_torrent_updated(
    redis_client: &redis::Client,
    torrent_id: uuid::Uuid,
) -> Result<()> {
    let mut conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| async_graphql::Error::new(format!("Redis connection error: {}", e)))?;

    let channel = format!("torrent:{}:updated", torrent_id);
    conn.publish(&channel, "updated")
        .await
        .map_err(|e| async_graphql::Error::new(format!("Redis publish error: {}", e)))?;

    Ok(())
}

/// Publish topic post event
#[instrument(skip(redis_client))]
pub async fn publish_topic_post(
    redis_client: &redis::Client,
    topic_id: uuid::Uuid,
    post_id: uuid::Uuid,
) -> Result<()> {
    let mut conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| async_graphql::Error::new(format!("Redis connection error: {}", e)))?;

    let channel = format!("topic:{}:posts", topic_id);
    conn.publish(&channel, post_id.to_string())
        .await
        .map_err(|e| async_graphql::Error::new(format!("Redis publish error: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscriptions_compile() {
        // Just ensure code compiles
        assert!(true);
    }
}
