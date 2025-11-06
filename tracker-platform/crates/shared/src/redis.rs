//! Redis client setup and caching utilities.
//!
//! This module provides Redis connection management, caching helpers,
//! and cache invalidation patterns.

use redis::{aio::ConnectionManager, AsyncCommands, Client, RedisError};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use tracing::{error, info};

use crate::config::Config;
use crate::error::AppResult;

/// Redis client wrapper with connection pooling
#[derive(Debug, Clone)]
pub struct RedisClient {
    connection_manager: ConnectionManager,
}

impl RedisClient {
    /// Create a new Redis client from configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Application configuration containing Redis URL
    ///
    /// # Errors
    ///
    /// Returns an error if the Redis connection cannot be established
    ///
    /// # Example
    ///
    /// ```no_run
    /// use shared::config::Config;
    /// use shared::redis::RedisClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = Config::from_env()?;
    /// let redis = RedisClient::new(&config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(config: &Config) -> AppResult<Self> {
        Self::from_url(&config.redis_url).await
    }

    /// Create a new Redis client from a URL
    ///
    /// # Arguments
    ///
    /// * `redis_url` - Redis connection URL
    ///
    /// # Errors
    ///
    /// Returns an error if the Redis connection cannot be established
    pub async fn from_url(redis_url: &str) -> AppResult<Self> {
        info!("Connecting to Redis at {}", redis_url);

        let client = Client::open(redis_url)?;
        let connection_manager = ConnectionManager::new(client).await?;

        info!("Redis connection established");

        Ok(Self { connection_manager })
    }

    /// Get a reference to the connection manager
    pub fn connection_manager(&self) -> &ConnectionManager {
        &self.connection_manager
    }

    /// Get a value from Redis
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> AppResult<Option<T>> {
        let mut conn = self.connection_manager.clone();
        let value: Option<String> = conn.get(key).await?;

        match value {
            Some(json) => {
                let data: T = serde_json::from_str(&json)
                    .map_err(|e| redis::RedisError::from((redis::ErrorKind::TypeError, "deserialization failed", e.to_string())))?;
                Ok(Some(data))
            }
            None => Ok(None),
        }
    }

    /// Set a value in Redis
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key
    /// * `value` - Value to store
    /// * `ttl` - Time to live (optional)
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub async fn set<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> AppResult<()> {
        let mut conn = self.connection_manager.clone();
        let json = serde_json::to_string(value)
            .map_err(|e| redis::RedisError::from((redis::ErrorKind::TypeError, "serialization failed", e.to_string())))?;

        match ttl {
            Some(duration) => {
                conn.set_ex(key, json, duration.as_secs() as u64).await?;
            }
            None => {
                conn.set(key, json).await?;
            }
        }

        Ok(())
    }

    /// Delete a key from Redis
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key to delete
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub async fn delete(&self, key: &str) -> AppResult<()> {
        let mut conn = self.connection_manager.clone();
        conn.del(key).await?;
        Ok(())
    }

    /// Delete multiple keys matching a pattern
    ///
    /// # Arguments
    ///
    /// * `pattern` - Pattern to match (e.g., "user:*")
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub async fn delete_pattern(&self, pattern: &str) -> AppResult<u64> {
        let mut conn = self.connection_manager.clone();
        let keys: Vec<String> = conn.keys(pattern).await?;

        if keys.is_empty() {
            return Ok(0);
        }

        let count = conn.del(&keys).await?;
        Ok(count)
    }

    /// Check if a key exists in Redis
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key to check
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub async fn exists(&self, key: &str) -> AppResult<bool> {
        let mut conn = self.connection_manager.clone();
        let exists: bool = conn.exists(key).await?;
        Ok(exists)
    }

    /// Set expiration time for a key
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key
    /// * `ttl` - Time to live
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub async fn expire(&self, key: &str, ttl: Duration) -> AppResult<()> {
        let mut conn = self.connection_manager.clone();
        conn.expire(key, ttl.as_secs() as i64).await?;
        Ok(())
    }

    /// Increment a counter
    ///
    /// # Arguments
    ///
    /// * `key` - Counter key
    /// * `delta` - Amount to increment by (default: 1)
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub async fn increment(&self, key: &str, delta: i64) -> AppResult<i64> {
        let mut conn = self.connection_manager.clone();
        let value: i64 = conn.incr(key, delta).await?;
        Ok(value)
    }

    /// Decrement a counter
    ///
    /// # Arguments
    ///
    /// * `key` - Counter key
    /// * `delta` - Amount to decrement by (default: 1)
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub async fn decrement(&self, key: &str, delta: i64) -> AppResult<i64> {
        let mut conn = self.connection_manager.clone();
        let value: i64 = conn.decr(key, delta).await?;
        Ok(value)
    }

    /// Get multiple values from Redis
    ///
    /// # Arguments
    ///
    /// * `keys` - List of cache keys
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub async fn mget<T: DeserializeOwned>(&self, keys: &[String]) -> AppResult<Vec<Option<T>>> {
        if keys.is_empty() {
            return Ok(vec![]);
        }

        let mut conn = self.connection_manager.clone();
        let values: Vec<Option<String>> = conn.get(keys).await?;

        let results: Vec<Option<T>> = values
            .into_iter()
            .map(|value| {
                value.and_then(|json| serde_json::from_str(&json).ok())
            })
            .collect();

        Ok(results)
    }

    /// Health check for Redis connection
    ///
    /// # Errors
    ///
    /// Returns an error if the health check fails
    pub async fn health_check(&self) -> AppResult<()> {
        let mut conn = self.connection_manager.clone();
        let _: String = redis::cmd("PING").query_async(&mut conn).await?;
        Ok(())
    }
}

/// Cache key builder helper
pub struct CacheKey;

impl CacheKey {
    /// Build a user cache key
    pub fn user(user_id: &str) -> String {
        format!("user:{}", user_id)
    }

    /// Build a torrent cache key
    pub fn torrent(torrent_id: &str) -> String {
        format!("torrent:{}", torrent_id)
    }

    /// Build a peer list cache key
    pub fn peers(info_hash: &str) -> String {
        format!("peers:{}", info_hash)
    }

    /// Build a session cache key
    pub fn session(session_id: &str) -> String {
        format!("session:{}", session_id)
    }

    /// Build a rate limit cache key
    pub fn rate_limit(identifier: &str) -> String {
        format!("rate_limit:{}", identifier)
    }

    /// Build a stats cache key
    pub fn stats(entity: &str, id: &str) -> String {
        format!("stats:{}:{}", entity, id)
    }
}

/// Common cache TTL durations
pub struct CacheTTL;

impl CacheTTL {
    /// 5 minutes
    pub const SHORT: Duration = Duration::from_secs(300);

    /// 1 hour
    pub const MEDIUM: Duration = Duration::from_secs(3600);

    /// 1 day
    pub const LONG: Duration = Duration::from_secs(86400);

    /// 1 week
    pub const VERY_LONG: Duration = Duration::from_secs(604800);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_builder() {
        assert_eq!(CacheKey::user("123"), "user:123");
        assert_eq!(CacheKey::torrent("abc"), "torrent:abc");
        assert_eq!(CacheKey::peers("hash"), "peers:hash");
        assert_eq!(CacheKey::session("sess_123"), "session:sess_123");
        assert_eq!(CacheKey::rate_limit("ip:127.0.0.1"), "rate_limit:ip:127.0.0.1");
    }

    #[tokio::test]
    #[ignore] // Requires a running Redis instance
    async fn test_redis_connection() {
        let redis_url = "redis://localhost:6379";
        let client = RedisClient::from_url(redis_url).await;
        assert!(client.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn test_set_and_get() {
        let redis_url = "redis://localhost:6379";
        let client = RedisClient::from_url(redis_url).await.unwrap();

        let key = "test_key";
        let value = "test_value";

        client.set(key, &value, None).await.unwrap();
        let result: Option<String> = client.get(key).await.unwrap();

        assert_eq!(result, Some(value.to_string()));

        client.delete(key).await.unwrap();
    }
}
