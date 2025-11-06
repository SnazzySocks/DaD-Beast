//! Metadata caching layer
//!
//! Caches API responses in the database to minimize external requests and
//! avoid hitting rate limits. Uses a TTL-based expiry system.

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{debug, warn};

/// Cache entry for metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub key: String,
    pub value: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Metadata cache
pub struct MetadataCache {
    db: PgPool,
    default_ttl: Duration,
}

impl MetadataCache {
    /// Create a new metadata cache
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            default_ttl: Duration::days(30),
        }
    }

    /// Set custom TTL
    pub fn with_ttl(mut self, days: i64) -> Self {
        self.default_ttl = Duration::days(days);
        self
    }

    /// Generate cache key from components
    pub fn make_key(source: &str, identifier: &str) -> String {
        format!("media:{}:{}", source, identifier)
    }

    /// Get cached value if it exists and hasn't expired
    pub async fn get(&self, key: &str) -> Result<Option<serde_json::Value>> {
        let result = sqlx::query!(
            r#"
            SELECT value, expires_at
            FROM media_cache
            WHERE key = $1
            "#,
            key
        )
        .fetch_optional(&self.db)
        .await?;

        match result {
            Some(row) => {
                let expires_at: DateTime<Utc> = row.expires_at;
                if expires_at > Utc::now() {
                    debug!("Cache hit for key: {}", key);
                    Ok(Some(row.value))
                } else {
                    debug!("Cache expired for key: {}", key);
                    // Clean up expired entry
                    self.delete(key).await?;
                    Ok(None)
                }
            }
            None => {
                debug!("Cache miss for key: {}", key);
                Ok(None)
            }
        }
    }

    /// Store value in cache with TTL
    pub async fn set(
        &self,
        key: &str,
        value: &serde_json::Value,
        ttl: Option<Duration>,
    ) -> Result<()> {
        let ttl = ttl.unwrap_or(self.default_ttl);
        let now = Utc::now();
        let expires_at = now + ttl;

        sqlx::query!(
            r#"
            INSERT INTO media_cache (key, value, created_at, expires_at)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (key) DO UPDATE
            SET value = $2, created_at = $3, expires_at = $4
            "#,
            key,
            value,
            now,
            expires_at
        )
        .execute(&self.db)
        .await?;

        debug!("Cached value for key: {} (expires: {})", key, expires_at);
        Ok(())
    }

    /// Delete a cache entry
    pub async fn delete(&self, key: &str) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM media_cache WHERE key = $1
            "#,
            key
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Get or compute a value (cache-aside pattern)
    pub async fn get_or_compute<F, Fut>(
        &self,
        key: &str,
        compute: F,
    ) -> Result<serde_json::Value>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<serde_json::Value>>,
    {
        // Try cache first
        if let Some(cached) = self.get(key).await? {
            return Ok(cached);
        }

        // Compute value
        let value = compute().await?;

        // Store in cache
        if let Err(e) = self.set(key, &value, None).await {
            warn!("Failed to cache value for key {}: {}", key, e);
        }

        Ok(value)
    }

    /// Clear all expired entries
    pub async fn clear_expired(&self) -> Result<u64> {
        let result = sqlx::query!(
            r#"
            DELETE FROM media_cache
            WHERE expires_at < $1
            "#,
            Utc::now()
        )
        .execute(&self.db)
        .await?;

        Ok(result.rows_affected())
    }

    /// Clear all cache entries
    pub async fn clear_all(&self) -> Result<u64> {
        let result = sqlx::query!("DELETE FROM media_cache")
            .execute(&self.db)
            .await?;

        Ok(result.rows_affected())
    }

    /// Get cache statistics
    pub async fn stats(&self) -> Result<CacheStats> {
        let result = sqlx::query!(
            r#"
            SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE expires_at > $1) as active,
                COUNT(*) FILTER (WHERE expires_at <= $1) as expired
            FROM media_cache
            "#,
            Utc::now()
        )
        .fetch_one(&self.db)
        .await?;

        Ok(CacheStats {
            total_entries: result.total.unwrap_or(0),
            active_entries: result.active.unwrap_or(0),
            expired_entries: result.expired.unwrap_or(0),
        })
    }

    /// Bulk prefetch entries by keys
    pub async fn bulk_get(&self, keys: &[String]) -> Result<Vec<(String, serde_json::Value)>> {
        let result = sqlx::query!(
            r#"
            SELECT key, value, expires_at
            FROM media_cache
            WHERE key = ANY($1) AND expires_at > $2
            "#,
            keys,
            Utc::now()
        )
        .fetch_all(&self.db)
        .await?;

        Ok(result
            .into_iter()
            .map(|row| (row.key, row.value))
            .collect())
    }

    /// Bulk insert cache entries
    pub async fn bulk_set(
        &self,
        entries: &[(String, serde_json::Value)],
        ttl: Option<Duration>,
    ) -> Result<()> {
        let ttl = ttl.unwrap_or(self.default_ttl);
        let now = Utc::now();
        let expires_at = now + ttl;

        for (key, value) in entries {
            sqlx::query!(
                r#"
                INSERT INTO media_cache (key, value, created_at, expires_at)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (key) DO UPDATE
                SET value = $2, created_at = $3, expires_at = $4
                "#,
                key,
                value,
                now,
                expires_at
            )
            .execute(&self.db)
            .await?;
        }

        debug!("Bulk cached {} entries", entries.len());
        Ok(())
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_entries: i64,
    pub active_entries: i64,
    pub expired_entries: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_key() {
        let key = MetadataCache::make_key("tmdb", "12345");
        assert_eq!(key, "media:tmdb:12345");

        let key = MetadataCache::make_key("imdb", "tt0111161");
        assert_eq!(key, "media:imdb:tt0111161");
    }
}
