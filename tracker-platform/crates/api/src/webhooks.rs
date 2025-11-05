//! # Webhook System
//!
//! Event-driven webhook system with delivery retry and logging.
//!
//! ## Features
//!
//! - **Event-based**: Trigger webhooks on platform events
//! - **Retry Logic**: Exponential backoff for failed deliveries
//! - **Logging**: Track all webhook deliveries
//! - **Filtering**: Subscribe to specific event types
//! - **Security**: HMAC signature verification

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info, instrument, warn};

/// Webhook manager for handling webhook registrations and deliveries
pub struct WebhookManager {
    db_pool: sqlx::PgPool,
    redis_client: redis::Client,
    http_client: reqwest::Client,
}

impl WebhookManager {
    /// Create a new webhook manager
    pub fn new(db_pool: sqlx::PgPool, redis_client: redis::Client) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            db_pool,
            redis_client,
            http_client,
        }
    }

    /// Register a new webhook
    #[instrument(skip(self))]
    pub async fn register_webhook(
        &self,
        user_id: uuid::Uuid,
        url: String,
        events: Vec<WebhookEvent>,
        secret: Option<String>,
    ) -> Result<Webhook, WebhookError> {
        // Validate URL
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(WebhookError::InvalidUrl);
        }

        // Generate secret if not provided
        let secret = secret.unwrap_or_else(|| {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            (0..32)
                .map(|_| format!("{:02x}", rng.gen::<u8>()))
                .collect::<String>()
        });

        // Insert webhook
        let webhook = sqlx::query_as::<_, Webhook>(
            "INSERT INTO webhooks (id, user_id, url, events, secret, is_active, created_at)
             VALUES ($1, $2, $3, $4, $5, true, $6)
             RETURNING *",
        )
        .bind(uuid::Uuid::new_v4())
        .bind(user_id)
        .bind(&url)
        .bind(serde_json::to_value(&events).unwrap())
        .bind(&secret)
        .bind(Utc::now())
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| WebhookError::DatabaseError(e.to_string()))?;

        info!("Webhook registered: {} for user {}", webhook.id, user_id);

        Ok(webhook)
    }

    /// Trigger a webhook event
    #[instrument(skip(self, payload))]
    pub async fn trigger_event(
        &self,
        event: WebhookEvent,
        payload: serde_json::Value,
    ) -> Result<(), WebhookError> {
        // Get all webhooks subscribed to this event
        let webhooks = sqlx::query_as::<_, Webhook>(
            "SELECT * FROM webhooks WHERE is_active = true AND events @> $1",
        )
        .bind(serde_json::to_value(&vec![event]).unwrap())
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| WebhookError::DatabaseError(e.to_string()))?;

        info!("Triggering {} webhooks for event {:?}", webhooks.len(), event);

        // Deliver webhooks asynchronously
        for webhook in webhooks {
            let manager = self.clone();
            let payload = payload.clone();
            tokio::spawn(async move {
                if let Err(e) = manager.deliver_webhook(webhook, event, payload).await {
                    error!("Failed to deliver webhook: {}", e);
                }
            });
        }

        Ok(())
    }

    /// Deliver a webhook with retry logic
    #[instrument(skip(self, webhook, payload))]
    async fn deliver_webhook(
        &self,
        webhook: Webhook,
        event: WebhookEvent,
        payload: serde_json::Value,
    ) -> Result<(), WebhookError> {
        let delivery_payload = WebhookPayload {
            event,
            timestamp: Utc::now(),
            data: payload,
        };

        let body = serde_json::to_string(&delivery_payload)
            .map_err(|e| WebhookError::SerializationError(e.to_string()))?;

        // Generate HMAC signature
        let signature = self.generate_signature(&webhook.secret, &body);

        // Attempt delivery with retries
        let max_retries = 3;
        let mut last_error = None;

        for attempt in 0..max_retries {
            info!(
                "Delivering webhook {} (attempt {}/{})",
                webhook.id,
                attempt + 1,
                max_retries
            );

            match self
                .http_client
                .post(&webhook.url)
                .header("Content-Type", "application/json")
                .header("X-Webhook-Signature", &signature)
                .header("X-Webhook-Event", format!("{:?}", event))
                .body(body.clone())
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        // Log successful delivery
                        self.log_delivery(
                            webhook.id,
                            event,
                            true,
                            Some(response.status().as_u16() as i32),
                            None,
                        )
                        .await?;

                        info!("Webhook delivered successfully: {}", webhook.id);
                        return Ok(());
                    } else {
                        let status = response.status().as_u16() as i32;
                        let error = format!("HTTP {}", status);
                        last_error = Some(error.clone());

                        warn!(
                            "Webhook delivery failed with status {}: {}",
                            status, webhook.id
                        );
                    }
                }
                Err(e) => {
                    last_error = Some(e.to_string());
                    warn!("Webhook delivery failed: {}", e);
                }
            }

            // Exponential backoff
            if attempt < max_retries - 1 {
                let delay = std::time::Duration::from_secs(2u64.pow(attempt));
                tokio::time::sleep(delay).await;
            }
        }

        // Log failed delivery
        self.log_delivery(
            webhook.id,
            event,
            false,
            None,
            last_error.as_deref(),
        )
        .await?;

        Err(WebhookError::DeliveryFailed(
            last_error.unwrap_or_else(|| "Unknown error".to_string()),
        ))
    }

    /// Generate HMAC signature for webhook payload
    fn generate_signature(&self, secret: &str, payload: &str) -> String {
        use sha2::{Digest, Sha256};
        use hmac::{Hmac, Mac};

        type HmacSha256 = Hmac<Sha256>;

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(payload.as_bytes());

        let result = mac.finalize();
        hex::encode(result.into_bytes())
    }

    /// Log webhook delivery
    #[instrument(skip(self))]
    async fn log_delivery(
        &self,
        webhook_id: uuid::Uuid,
        event: WebhookEvent,
        success: bool,
        status_code: Option<i32>,
        error_message: Option<&str>,
    ) -> Result<(), WebhookError> {
        sqlx::query(
            "INSERT INTO webhook_logs
             (id, webhook_id, event, success, status_code, error_message, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(uuid::Uuid::new_v4())
        .bind(webhook_id)
        .bind(format!("{:?}", event))
        .bind(success)
        .bind(status_code)
        .bind(error_message)
        .bind(Utc::now())
        .execute(&self.db_pool)
        .await
        .map_err(|e| WebhookError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Get webhook delivery logs
    #[instrument(skip(self))]
    pub async fn get_webhook_logs(
        &self,
        webhook_id: uuid::Uuid,
        limit: i32,
    ) -> Result<Vec<WebhookLog>, WebhookError> {
        let logs = sqlx::query_as::<_, WebhookLog>(
            "SELECT * FROM webhook_logs WHERE webhook_id = $1 ORDER BY created_at DESC LIMIT $2",
        )
        .bind(webhook_id)
        .bind(limit)
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| WebhookError::DatabaseError(e.to_string()))?;

        Ok(logs)
    }

    /// Delete a webhook
    #[instrument(skip(self))]
    pub async fn delete_webhook(&self, webhook_id: uuid::Uuid) -> Result<(), WebhookError> {
        sqlx::query("DELETE FROM webhooks WHERE id = $1")
            .bind(webhook_id)
            .execute(&self.db_pool)
            .await
            .map_err(|e| WebhookError::DatabaseError(e.to_string()))?;

        info!("Webhook deleted: {}", webhook_id);

        Ok(())
    }
}

// Allow cloning for async operations
impl Clone for WebhookManager {
    fn clone(&self) -> Self {
        Self {
            db_pool: self.db_pool.clone(),
            redis_client: self.redis_client.clone(),
            http_client: self.http_client.clone(),
        }
    }
}

/// Webhook event types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEvent {
    TorrentAdded,
    TorrentUpdated,
    TorrentDeleted,
    UserRegistered,
    UserUpdated,
    MessageReceived,
    ForumPostCreated,
    CommentAdded,
}

/// Webhook configuration
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Webhook {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub url: String,
    #[sqlx(json)]
    pub events: serde_json::Value,
    pub secret: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// Webhook delivery payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub event: WebhookEvent,
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
}

/// Webhook delivery log
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WebhookLog {
    pub id: uuid::Uuid,
    pub webhook_id: uuid::Uuid,
    pub event: String,
    pub success: bool,
    pub status_code: Option<i32>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Webhook error types
#[derive(Debug, thiserror::Error)]
pub enum WebhookError {
    #[error("Invalid webhook URL")]
    InvalidUrl,

    #[error("Webhook delivery failed: {0}")]
    DeliveryFailed(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Not found")]
    NotFound,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_event_serialization() {
        let event = WebhookEvent::TorrentAdded;
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(json, r#""torrent_added""#);
    }

    #[test]
    fn test_webhook_payload_serialization() {
        let payload = WebhookPayload {
            event: WebhookEvent::TorrentAdded,
            timestamp: Utc::now(),
            data: serde_json::json!({"id": "123"}),
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("torrent_added"));
    }
}
