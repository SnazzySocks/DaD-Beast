//! Session management
//!
//! This module handles user sessions, storing them in Redis for fast access
//! and supporting features like session listing, individual revocation, and
//! logout from all devices.

use chrono::{DateTime, Utc};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur during session operations
#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Redis connection error: {0}")]
    RedisConnection(String),

    #[error("Session not found")]
    SessionNotFound,

    #[error("Failed to serialize/deserialize session: {0}")]
    Serialization(String),

    #[error("Session operation failed: {0}")]
    OperationFailed(String),
}

/// User session information
///
/// Tracks active login sessions with device and location information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session ID
    pub session_id: Uuid,
    /// User ID this session belongs to
    pub user_id: Uuid,
    /// JWT token ID associated with this session
    pub token_id: Uuid,
    /// Session creation time
    pub created_at: DateTime<Utc>,
    /// Session last activity time
    pub last_activity: DateTime<Utc>,
    /// Session expiration time
    pub expires_at: DateTime<Utc>,
    /// User agent string (browser/device info)
    pub user_agent: Option<String>,
    /// IP address
    pub ip_address: String,
    /// Device type (desktop, mobile, tablet, etc.)
    pub device_type: Option<String>,
    /// Operating system
    pub os: Option<String>,
    /// Browser name
    pub browser: Option<String>,
    /// Approximate location (city, country)
    pub location: Option<String>,
}

impl Session {
    /// Create a new session
    pub fn new(
        user_id: Uuid,
        token_id: Uuid,
        ip_address: String,
        user_agent: Option<String>,
        ttl_seconds: i64,
    ) -> Self {
        let now = Utc::now();
        let expires_at = now + chrono::Duration::seconds(ttl_seconds);

        Self {
            session_id: Uuid::new_v4(),
            user_id,
            token_id,
            created_at: now,
            last_activity: now,
            expires_at,
            user_agent,
            ip_address,
            device_type: None,
            os: None,
            browser: None,
            location: None,
        }
    }

    /// Update last activity timestamp
    pub fn update_activity(&mut self) {
        self.last_activity = Utc::now();
    }

    /// Check if session has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

/// Session manager for Redis-backed session storage
pub struct SessionManager {
    redis_client: redis::Client,
    key_prefix: String,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(redis_client: redis::Client) -> Self {
        Self {
            redis_client,
            key_prefix: "auth:session".to_string(),
        }
    }

    /// Create a new session manager with custom key prefix
    pub fn with_prefix(redis_client: redis::Client, key_prefix: String) -> Self {
        Self {
            redis_client,
            key_prefix,
        }
    }

    /// Generate Redis key for a session
    fn session_key(&self, session_id: Uuid) -> String {
        format!("{}:{}", self.key_prefix, session_id)
    }

    /// Generate Redis key for user's session list
    fn user_sessions_key(&self, user_id: Uuid) -> String {
        format!("{}:user:{}:sessions", self.key_prefix, user_id)
    }

    /// Store a session in Redis
    pub async fn create_session(&self, session: &Session) -> Result<(), SessionError> {
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| SessionError::RedisConnection(e.to_string()))?;

        let session_key = self.session_key(session.session_id);
        let user_sessions_key = self.user_sessions_key(session.user_id);

        // Serialize session
        let session_data = serde_json::to_string(session)
            .map_err(|e| SessionError::Serialization(e.to_string()))?;

        // Calculate TTL
        let ttl = (session.expires_at.timestamp() - Utc::now().timestamp()).max(0) as usize;

        // Store session data with expiration
        conn.set_ex(&session_key, session_data, ttl)
            .await
            .map_err(|e| SessionError::OperationFailed(e.to_string()))?;

        // Add session ID to user's session set
        conn.sadd(&user_sessions_key, session.session_id.to_string())
            .await
            .map_err(|e| SessionError::OperationFailed(e.to_string()))?;

        // Set expiration on user's session set (cleanup)
        conn.expire(&user_sessions_key, ttl as i64)
            .await
            .map_err(|e| SessionError::OperationFailed(e.to_string()))?;

        Ok(())
    }

    /// Retrieve a session from Redis
    pub async fn get_session(&self, session_id: Uuid) -> Result<Session, SessionError> {
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| SessionError::RedisConnection(e.to_string()))?;

        let session_key = self.session_key(session_id);

        let session_data: Option<String> = conn
            .get(&session_key)
            .await
            .map_err(|e| SessionError::OperationFailed(e.to_string()))?;

        match session_data {
            Some(data) => {
                let session: Session = serde_json::from_str(&data)
                    .map_err(|e| SessionError::Serialization(e.to_string()))?;

                // Check if expired
                if session.is_expired() {
                    self.delete_session(session_id).await?;
                    return Err(SessionError::SessionNotFound);
                }

                Ok(session)
            }
            None => Err(SessionError::SessionNotFound),
        }
    }

    /// Update session activity timestamp
    pub async fn update_activity(&self, session_id: Uuid) -> Result<(), SessionError> {
        let mut session = self.get_session(session_id).await?;
        session.update_activity();
        self.create_session(&session).await
    }

    /// Delete a specific session
    pub async fn delete_session(&self, session_id: Uuid) -> Result<(), SessionError> {
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| SessionError::RedisConnection(e.to_string()))?;

        // Get session to find user_id
        if let Ok(session) = self.get_session(session_id).await {
            let user_sessions_key = self.user_sessions_key(session.user_id);

            // Remove from user's session set
            let _: () = conn
                .srem(&user_sessions_key, session_id.to_string())
                .await
                .map_err(|e| SessionError::OperationFailed(e.to_string()))?;
        }

        // Delete session data
        let session_key = self.session_key(session_id);
        conn.del(&session_key)
            .await
            .map_err(|e| SessionError::OperationFailed(e.to_string()))?;

        Ok(())
    }

    /// List all active sessions for a user
    pub async fn list_user_sessions(&self, user_id: Uuid) -> Result<Vec<Session>, SessionError> {
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| SessionError::RedisConnection(e.to_string()))?;

        let user_sessions_key = self.user_sessions_key(user_id);

        // Get all session IDs for the user
        let session_ids: Vec<String> = conn
            .smembers(&user_sessions_key)
            .await
            .map_err(|e| SessionError::OperationFailed(e.to_string()))?;

        let mut sessions = Vec::new();
        for session_id_str in session_ids {
            if let Ok(session_id) = Uuid::parse_str(&session_id_str) {
                if let Ok(session) = self.get_session(session_id).await {
                    sessions.push(session);
                }
            }
        }

        // Sort by last activity (most recent first)
        sessions.sort_by(|a, b| b.last_activity.cmp(&a.last_activity));

        Ok(sessions)
    }

    /// Delete all sessions for a user (logout from all devices)
    pub async fn delete_all_user_sessions(&self, user_id: Uuid) -> Result<usize, SessionError> {
        let sessions = self.list_user_sessions(user_id).await?;
        let count = sessions.len();

        for session in sessions {
            self.delete_session(session.session_id).await?;
        }

        Ok(count)
    }

    /// Get session by token ID
    pub async fn get_session_by_token(&self, token_id: Uuid) -> Result<Session, SessionError> {
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| SessionError::RedisConnection(e.to_string()))?;

        // Scan for session with matching token_id
        // This is less efficient but works for token-based lookup
        let pattern = format!("{}:*", self.key_prefix);
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query_async(&mut conn)
            .await
            .map_err(|e| SessionError::OperationFailed(e.to_string()))?;

        for key in keys {
            if let Ok(session_data) = conn.get::<_, Option<String>>(&key).await {
                if let Some(data) = session_data {
                    if let Ok(session) = serde_json::from_str::<Session>(&data) {
                        if session.token_id == token_id && !session.is_expired() {
                            return Ok(session);
                        }
                    }
                }
            }
        }

        Err(SessionError::SessionNotFound)
    }

    /// Clean up expired sessions for a user
    pub async fn cleanup_expired_sessions(&self, user_id: Uuid) -> Result<usize, SessionError> {
        let sessions = self.list_user_sessions(user_id).await?;
        let mut cleaned = 0;

        for session in sessions {
            if session.is_expired() {
                self.delete_session(session.session_id).await?;
                cleaned += 1;
            }
        }

        Ok(cleaned)
    }
}

/// Parse user agent string to extract device information
///
/// This is a simple parser. For production, consider using a library like `woothee` or `uap-rs`.
pub fn parse_user_agent(user_agent: &str) -> (Option<String>, Option<String>, Option<String>) {
    let ua_lower = user_agent.to_lowercase();

    // Detect device type
    let device_type = if ua_lower.contains("mobile") {
        Some("Mobile".to_string())
    } else if ua_lower.contains("tablet") || ua_lower.contains("ipad") {
        Some("Tablet".to_string())
    } else {
        Some("Desktop".to_string())
    };

    // Detect OS
    let os = if ua_lower.contains("windows") {
        Some("Windows".to_string())
    } else if ua_lower.contains("mac os") || ua_lower.contains("macos") {
        Some("macOS".to_string())
    } else if ua_lower.contains("linux") {
        Some("Linux".to_string())
    } else if ua_lower.contains("android") {
        Some("Android".to_string())
    } else if ua_lower.contains("ios") || ua_lower.contains("iphone") {
        Some("iOS".to_string())
    } else {
        None
    };

    // Detect browser
    let browser = if ua_lower.contains("chrome") && !ua_lower.contains("edg") {
        Some("Chrome".to_string())
    } else if ua_lower.contains("firefox") {
        Some("Firefox".to_string())
    } else if ua_lower.contains("safari") && !ua_lower.contains("chrome") {
        Some("Safari".to_string())
    } else if ua_lower.contains("edg") {
        Some("Edge".to_string())
    } else if ua_lower.contains("opera") || ua_lower.contains("opr") {
        Some("Opera".to_string())
    } else {
        None
    };

    (device_type, os, browser)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let user_id = Uuid::new_v4();
        let token_id = Uuid::new_v4();
        let session = Session::new(
            user_id,
            token_id,
            "127.0.0.1".to_string(),
            Some("Mozilla/5.0".to_string()),
            3600,
        );

        assert_eq!(session.user_id, user_id);
        assert_eq!(session.token_id, token_id);
        assert!(!session.is_expired());
    }

    #[test]
    fn test_session_expiry() {
        let user_id = Uuid::new_v4();
        let token_id = Uuid::new_v4();
        let mut session = Session::new(
            user_id,
            token_id,
            "127.0.0.1".to_string(),
            None,
            3600,
        );

        assert!(!session.is_expired());

        // Force expiry
        session.expires_at = Utc::now() - chrono::Duration::seconds(1);
        assert!(session.is_expired());
    }

    #[test]
    fn test_parse_user_agent_chrome() {
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";
        let (device, os, browser) = parse_user_agent(ua);

        assert_eq!(device, Some("Desktop".to_string()));
        assert_eq!(os, Some("Windows".to_string()));
        assert_eq!(browser, Some("Chrome".to_string()));
    }

    #[test]
    fn test_parse_user_agent_firefox() {
        let ua = "Mozilla/5.0 (X11; Linux x86_64; rv:89.0) Gecko/20100101 Firefox/89.0";
        let (device, os, browser) = parse_user_agent(ua);

        assert_eq!(device, Some("Desktop".to_string()));
        assert_eq!(os, Some("Linux".to_string()));
        assert_eq!(browser, Some("Firefox".to_string()));
    }

    #[test]
    fn test_parse_user_agent_mobile() {
        let ua = "Mozilla/5.0 (iPhone; CPU iPhone OS 14_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0 Mobile/15E148 Safari/604.1";
        let (device, os, browser) = parse_user_agent(ua);

        assert_eq!(device, Some("Mobile".to_string()));
        assert_eq!(os, Some("iOS".to_string()));
        assert_eq!(browser, Some("Safari".to_string()));
    }

    #[test]
    fn test_session_activity_update() {
        let user_id = Uuid::new_v4();
        let token_id = Uuid::new_v4();
        let mut session = Session::new(
            user_id,
            token_id,
            "127.0.0.1".to_string(),
            None,
            3600,
        );

        let original_activity = session.last_activity;
        std::thread::sleep(std::time::Duration::from_millis(10));
        session.update_activity();

        assert!(session.last_activity > original_activity);
    }
}
