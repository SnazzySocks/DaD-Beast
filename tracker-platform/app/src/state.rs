use crate::config::Config;
use anyhow::{Context, Result};
use meilisearch_sdk::Client as MeilisearchClient;
use redis::aio::ConnectionManager;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;

/// Application state shared across all request handlers
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: PgPool,
    pub redis: ConnectionManager,
    pub meilisearch: MeilisearchClient,
    pub auth_service: Arc<auth::AuthService>,
    pub tracker_service: Arc<tracker::TrackerService>,
    pub torrent_service: Arc<torrent::TorrentService>,
    pub user_service: Arc<user::UserService>,
    pub search_service: Arc<search::SearchService>,
    pub media_service: Arc<media::MediaService>,
    pub community_service: Arc<community::CommunityService>,
}

impl AppState {
    pub async fn new(config: Config) -> Result<Self> {
        let config = Arc::new(config);

        // Initialize database connection pool
        tracing::info!("Connecting to database...");
        let db = create_db_pool(&config.database).await?;
        tracing::info!("Database connection established");

        // Run migrations if enabled
        if config.database.run_migrations {
            tracing::info!("Running database migrations...");
            sqlx::migrate!("../migrations")
                .run(&db)
                .await
                .context("Failed to run migrations")?;
            tracing::info!("Database migrations completed");
        }

        // Initialize Redis connection
        tracing::info!("Connecting to Redis...");
        let redis = create_redis_client(&config.redis).await?;
        tracing::info!("Redis connection established");

        // Initialize Meilisearch client
        tracing::info!("Connecting to Meilisearch...");
        let meilisearch = create_meilisearch_client(&config.meilisearch)?;
        tracing::info!("Meilisearch client initialized");

        // Initialize services
        tracing::info!("Initializing services...");

        let auth_service = Arc::new(
            auth::AuthService::new(
                db.clone(),
                redis.clone(),
                &config.auth.jwt_secret,
                config.auth.jwt_expiration_hours,
            )
            .await?,
        );

        let tracker_service = Arc::new(
            tracker::TrackerService::new(db.clone(), redis.clone()).await?,
        );

        let torrent_service = Arc::new(
            torrent::TorrentService::new(
                db.clone(),
                redis.clone(),
                config.storage.upload_dir.clone(),
            )
            .await?,
        );

        let user_service = Arc::new(
            user::UserService::new(db.clone(), redis.clone()).await?,
        );

        let search_service = Arc::new(
            search::SearchService::new(
                meilisearch.clone(),
                db.clone(),
            )
            .await?,
        );

        let media_service = Arc::new(
            media::MediaService::new(
                db.clone(),
                config.storage.upload_dir.clone(),
            )
            .await?,
        );

        let community_service = Arc::new(
            community::CommunityService::new(db.clone(), redis.clone()).await?,
        );

        tracing::info!("All services initialized successfully");

        Ok(Self {
            config,
            db,
            redis,
            meilisearch,
            auth_service,
            tracker_service,
            torrent_service,
            user_service,
            search_service,
            media_service,
            community_service,
        })
    }

    /// Health check for the application
    pub async fn health_check(&self) -> Result<HealthStatus> {
        let mut status = HealthStatus::default();

        // Check database
        match sqlx::query("SELECT 1").fetch_one(&self.db).await {
            Ok(_) => status.database = true,
            Err(e) => {
                tracing::error!("Database health check failed: {}", e);
                status.database = false;
            }
        }

        // Check Redis
        match redis::cmd("PING")
            .query_async::<_, String>(&mut self.redis.clone())
            .await
        {
            Ok(_) => status.redis = true,
            Err(e) => {
                tracing::error!("Redis health check failed: {}", e);
                status.redis = false;
            }
        }

        // Check Meilisearch
        match self.meilisearch.health().await {
            Ok(_) => status.meilisearch = true,
            Err(e) => {
                tracing::error!("Meilisearch health check failed: {}", e);
                status.meilisearch = false;
            }
        }

        status.overall = status.database && status.redis && status.meilisearch;

        Ok(status)
    }

    /// Gracefully close all connections
    pub async fn close(self) -> Result<()> {
        tracing::info!("Closing database connections...");
        self.db.close().await;

        tracing::info!("All connections closed");
        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct HealthStatus {
    pub overall: bool,
    pub database: bool,
    pub redis: bool,
    pub meilisearch: bool,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self {
            overall: false,
            database: false,
            redis: false,
            meilisearch: false,
        }
    }
}

/// Create a PostgreSQL connection pool
async fn create_db_pool(config: &crate::config::DatabaseConfig) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(Duration::from_secs(config.connect_timeout_secs))
        .idle_timeout(Duration::from_secs(config.idle_timeout_secs))
        .max_lifetime(Duration::from_secs(config.max_lifetime_secs))
        .connect(&config.url)
        .await
        .context("Failed to create database pool")?;

    Ok(pool)
}

/// Create a Redis connection manager
async fn create_redis_client(config: &crate::config::RedisConfig) -> Result<ConnectionManager> {
    let client = redis::Client::open(config.url.as_str())
        .context("Failed to create Redis client")?;

    let manager = ConnectionManager::new(client)
        .await
        .context("Failed to create Redis connection manager")?;

    Ok(manager)
}

/// Create a Meilisearch client
fn create_meilisearch_client(
    config: &crate::config::MeilisearchConfig,
) -> Result<MeilisearchClient> {
    let client = MeilisearchClient::new(&config.url, config.api_key.as_deref());

    Ok(client)
}
