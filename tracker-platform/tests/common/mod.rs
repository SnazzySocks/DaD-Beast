/// Common test utilities shared across integration tests
use sqlx::{PgPool, Pool, Postgres};
use redis::aio::ConnectionManager;
use std::sync::Arc;
use tokio::sync::OnceCell;
use uuid::Uuid;

pub mod fixtures;
pub mod mocks;
pub mod helpers;

static TEST_DB_POOL: OnceCell<PgPool> = OnceCell::const_new();
static TEST_REDIS: OnceCell<ConnectionManager> = OnceCell::const_new();

/// Initialize test database pool
pub async fn init_test_db() -> PgPool {
    TEST_DB_POOL
        .get_or_init(|| async {
            let database_url = std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://tracker:tracker_password@localhost:5432/tracker_test".to_string());

            PgPool::connect(&database_url)
                .await
                .expect("Failed to connect to test database")
        })
        .await
        .clone()
}

/// Initialize test Redis connection
pub async fn init_test_redis() -> ConnectionManager {
    TEST_REDIS
        .get_or_init(|| async {
            let redis_url = std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string());

            let client = redis::Client::open(redis_url)
                .expect("Failed to create Redis client");

            ConnectionManager::new(client)
                .await
                .expect("Failed to connect to Redis")
        })
        .await
        .clone()
}

/// Test context containing all services
pub struct TestContext {
    pub db: PgPool,
    pub redis: ConnectionManager,
    pub test_id: String,
}

impl TestContext {
    /// Create a new test context
    pub async fn new() -> Self {
        Self {
            db: init_test_db().await,
            redis: init_test_redis().await,
            test_id: Uuid::new_v4().to_string(),
        }
    }

    /// Run migrations
    pub async fn migrate(&self) -> Result<(), sqlx::Error> {
        sqlx::migrate!("./migrations")
            .run(&self.db)
            .await
    }

    /// Clean up test data
    pub async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Clean up database
        sqlx::query("DELETE FROM sessions WHERE user_id LIKE $1")
            .bind(format!("test_{}_%", self.test_id))
            .execute(&self.db)
            .await?;

        // Clean up Redis
        let mut conn = self.redis.clone();
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(format!("test:{}:*", self.test_id))
            .query_async(&mut conn)
            .await?;

        if !keys.is_empty() {
            redis::cmd("DEL")
                .arg(&keys)
                .query_async(&mut conn)
                .await?;
        }

        Ok(())
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        // Attempt cleanup on drop, but don't panic if it fails
        let test_id = self.test_id.clone();
        tokio::spawn(async move {
            let _ = cleanup_test_data(&test_id).await;
        });
    }
}

async fn cleanup_test_data(test_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let ctx = TestContext::new().await;
    ctx.cleanup().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_context_creation() {
        let ctx = TestContext::new().await;
        assert!(!ctx.test_id.is_empty());
    }

    #[tokio::test]
    async fn test_db_connection() {
        let pool = init_test_db().await;
        let result: (i32,) = sqlx::query_as("SELECT 1")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(result.0, 1);
    }

    #[tokio::test]
    async fn test_redis_connection() {
        let mut conn = init_test_redis().await;
        let pong: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .unwrap();
        assert_eq!(pong, "PONG");
    }
}
