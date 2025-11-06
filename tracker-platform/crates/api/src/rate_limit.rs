//! # Rate Limiting
//!
//! API rate limiting with token bucket algorithm and Redis backend.
//!
//! ## Features
//!
//! - **Per-user limits**: Track limits per authenticated user
//! - **Per-IP limits**: Track limits per IP address
//! - **Per-endpoint limits**: Different limits for different endpoints
//! - **Token bucket algorithm**: Smooth rate limiting with burst capacity
//! - **Rate limit headers**: Return X-RateLimit-* headers

use redis::AsyncCommands;
use std::net::IpAddr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{error, instrument, warn};

/// Rate limiter using Redis backend
pub struct RateLimiter {
    redis_client: redis::Client,
    default_limit: u32,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(redis_client: redis::Client, default_limit: u32) -> Self {
        Self {
            redis_client,
            default_limit,
        }
    }

    /// Check if a request should be rate limited
    ///
    /// Returns Ok(RateLimitInfo) if allowed, Err(RateLimitError) if rate limited
    #[instrument(skip(self))]
    pub async fn check_rate_limit(
        &self,
        key: &str,
        limit: Option<u32>,
        window_secs: Option<u64>,
    ) -> Result<RateLimitInfo, RateLimitError> {
        let limit = limit.unwrap_or(self.default_limit);
        let window = window_secs.unwrap_or(60); // Default: 60 seconds

        // Get current timestamp
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Redis key for this rate limit
        let redis_key = format!("ratelimit:{}", key);

        // Get connection
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| {
                error!("Failed to get Redis connection: {}", e);
                RateLimitError::InternalError
            })?;

        // Use Lua script for atomic rate limiting (token bucket algorithm)
        let script = r#"
            local key = KEYS[1]
            local limit = tonumber(ARGV[1])
            local window = tonumber(ARGV[2])
            local now = tonumber(ARGV[3])

            -- Get current bucket state
            local bucket = redis.call('HMGET', key, 'tokens', 'last_update')
            local tokens = tonumber(bucket[1])
            local last_update = tonumber(bucket[2])

            -- Initialize if not exists
            if not tokens then
                tokens = limit
                last_update = now
            end

            -- Calculate tokens to add based on time elapsed
            local elapsed = now - last_update
            local tokens_to_add = elapsed * (limit / window)
            tokens = math.min(limit, tokens + tokens_to_add)

            -- Check if we have tokens
            if tokens < 1 then
                return {0, tokens, limit}
            end

            -- Consume one token
            tokens = tokens - 1

            -- Update bucket state
            redis.call('HMSET', key, 'tokens', tokens, 'last_update', now)
            redis.call('EXPIRE', key, window * 2)

            return {1, tokens, limit}
        "#;

        let result: Vec<i64> = redis::Script::new(script)
            .key(&redis_key)
            .arg(limit)
            .arg(window)
            .arg(now)
            .invoke_async(&mut conn)
            .await
            .map_err(|e| {
                error!("Rate limit script failed: {}", e);
                RateLimitError::InternalError
            })?;

        let allowed = result[0] == 1;
        let remaining = result[1].max(0) as u32;
        let limit = result[2] as u32;

        if allowed {
            Ok(RateLimitInfo {
                limit,
                remaining,
                reset: now + window,
            })
        } else {
            warn!("Rate limit exceeded for key: {}", key);
            Err(RateLimitError::RateLimitExceeded {
                retry_after: window,
            })
        }
    }

    /// Check rate limit for a user
    #[instrument(skip(self))]
    pub async fn check_user_limit(
        &self,
        user_id: uuid::Uuid,
        endpoint: Option<&str>,
    ) -> Result<RateLimitInfo, RateLimitError> {
        let key = if let Some(ep) = endpoint {
            format!("user:{}:endpoint:{}", user_id, ep)
        } else {
            format!("user:{}", user_id)
        };

        // Higher limits for authenticated users
        self.check_rate_limit(&key, Some(120), Some(60)).await
    }

    /// Check rate limit for an IP address
    #[instrument(skip(self))]
    pub async fn check_ip_limit(
        &self,
        ip: IpAddr,
        endpoint: Option<&str>,
    ) -> Result<RateLimitInfo, RateLimitError> {
        let key = if let Some(ep) = endpoint {
            format!("ip:{}:endpoint:{}", ip, ep)
        } else {
            format!("ip:{}", ip)
        };

        // Lower limits for unauthenticated requests
        self.check_rate_limit(&key, Some(30), Some(60)).await
    }

    /// Check rate limit for API key
    #[instrument(skip(self))]
    pub async fn check_api_key_limit(
        &self,
        api_key: &str,
    ) -> Result<RateLimitInfo, RateLimitError> {
        let key = format!("apikey:{}", api_key);

        // Custom limits for API keys
        self.check_rate_limit(&key, Some(1000), Some(60)).await
    }

    /// Reset rate limit for a key (admin operation)
    #[instrument(skip(self))]
    pub async fn reset_limit(&self, key: &str) -> Result<(), RateLimitError> {
        let redis_key = format!("ratelimit:{}", key);

        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| {
                error!("Failed to get Redis connection: {}", e);
                RateLimitError::InternalError
            })?;

        conn.del(&redis_key).await.map_err(|e| {
            error!("Failed to delete rate limit key: {}", e);
            RateLimitError::InternalError
        })?;

        Ok(())
    }

    /// Get current rate limit status without consuming a token
    #[instrument(skip(self))]
    pub async fn get_limit_status(&self, key: &str) -> Result<RateLimitInfo, RateLimitError> {
        let redis_key = format!("ratelimit:{}", key);
        let limit = self.default_limit;
        let window = 60u64;

        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| {
                error!("Failed to get Redis connection: {}", e);
                RateLimitError::InternalError
            })?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Get current bucket state
        let bucket: Vec<Option<String>> = conn.hget(&redis_key, &["tokens", "last_update"]).await.map_err(|e| {
            error!("Failed to get rate limit status: {}", e);
            RateLimitError::InternalError
        })?;

        let tokens = bucket[0]
            .as_ref()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(limit as f64);

        let remaining = tokens.floor().max(0.0) as u32;

        Ok(RateLimitInfo {
            limit,
            remaining,
            reset: now + window,
        })
    }
}

/// Rate limit information
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    /// Maximum requests allowed in window
    pub limit: u32,
    /// Remaining requests in current window
    pub remaining: u32,
    /// Unix timestamp when the limit resets
    pub reset: u64,
}

impl RateLimitInfo {
    /// Convert to HTTP headers
    pub fn to_headers(&self) -> Vec<(String, String)> {
        vec![
            ("X-RateLimit-Limit".to_string(), self.limit.to_string()),
            (
                "X-RateLimit-Remaining".to_string(),
                self.remaining.to_string(),
            ),
            ("X-RateLimit-Reset".to_string(), self.reset.to_string()),
        ]
    }
}

/// Rate limit error
#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Rate limit exceeded, retry after {retry_after} seconds")]
    RateLimitExceeded { retry_after: u64 },

    #[error("Internal error")]
    InternalError,
}

/// Axum middleware for rate limiting
pub mod middleware {
    use super::*;
    use axum::{
        extract::{ConnectInfo, State},
        http::{Request, StatusCode},
        middleware::Next,
        response::{IntoResponse, Response},
    };
    use std::net::SocketAddr;
    use std::sync::Arc;

    /// Rate limiting middleware
    pub async fn rate_limit<B>(
        State(limiter): State<Arc<RateLimiter>>,
        ConnectInfo(addr): ConnectInfo<SocketAddr>,
        req: Request<B>,
        next: Next<B>,
    ) -> Result<Response, StatusCode> {
        // Extract user ID from request (if authenticated)
        // For now, just use IP-based limiting
        let ip = addr.ip();

        // Get endpoint from path
        let endpoint = req.uri().path();

        match limiter.check_ip_limit(ip, Some(endpoint)).await {
            Ok(info) => {
                let mut response = next.run(req).await;

                // Add rate limit headers
                let headers = response.headers_mut();
                for (key, value) in info.to_headers() {
                    headers.insert(
                        axum::http::HeaderName::from_bytes(key.as_bytes()).unwrap(),
                        axum::http::HeaderValue::from_str(&value).unwrap(),
                    );
                }

                Ok(response)
            }
            Err(RateLimitError::RateLimitExceeded { retry_after }) => {
                let body = serde_json::json!({
                    "error": "Rate limit exceeded",
                    "retry_after": retry_after,
                });

                let response = (
                    StatusCode::TOO_MANY_REQUESTS,
                    [(
                        axum::http::header::RETRY_AFTER,
                        retry_after.to_string(),
                    )],
                    axum::Json(body),
                );

                Ok(response.into_response())
            }
            Err(_) => {
                // On internal error, allow the request but log the error
                Ok(next.run(req).await)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_info_to_headers() {
        let info = RateLimitInfo {
            limit: 100,
            remaining: 50,
            reset: 1234567890,
        };

        let headers = info.to_headers();
        assert_eq!(headers.len(), 3);
        assert!(headers
            .iter()
            .any(|(k, v)| k == "X-RateLimit-Limit" && v == "100"));
        assert!(headers
            .iter()
            .any(|(k, v)| k == "X-RateLimit-Remaining" && v == "50"));
    }

    #[test]
    fn test_rate_limit_error_display() {
        let err = RateLimitError::RateLimitExceeded { retry_after: 60 };
        assert_eq!(
            err.to_string(),
            "Rate limit exceeded, retry after 60 seconds"
        );
    }
}
