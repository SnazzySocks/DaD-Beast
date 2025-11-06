use anyhow::{Context, Result};
use config::{Config as ConfigBuilder, Environment, File};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub meilisearch: MeilisearchConfig,
    pub kafka: Option<KafkaConfig>,
    pub auth: AuthConfig,
    pub storage: StorageConfig,
    pub telemetry: TelemetryConfig,
    pub cors: CorsConfig,
    pub rate_limit: RateLimitConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
    pub max_connections: Option<usize>,
    pub request_timeout_secs: Option<u64>,
    pub graceful_shutdown_timeout_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_secs: u64,
    pub idle_timeout_secs: u64,
    pub max_lifetime_secs: u64,
    pub run_migrations: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: Option<u32>,
    pub connection_timeout_secs: u64,
    pub response_timeout_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MeilisearchConfig {
    pub url: String,
    pub api_key: Option<String>,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KafkaConfig {
    pub brokers: String,
    pub group_id: String,
    pub topics: KafkaTopics,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KafkaTopics {
    pub torrent_events: String,
    pub user_events: String,
    pub peer_events: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiration_hours: u64,
    pub refresh_token_expiration_days: u64,
    pub password_reset_expiration_hours: u64,
    pub email_verification_expiration_hours: u64,
    pub max_login_attempts: u32,
    pub lockout_duration_minutes: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    pub upload_dir: PathBuf,
    pub max_upload_size_mb: u64,
    pub allowed_mime_types: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TelemetryConfig {
    pub service_name: String,
    pub environment: String,
    pub log_level: String,
    pub log_format: String, // "json" or "pretty"
    pub otlp_endpoint: Option<String>,
    pub metrics_enabled: bool,
    pub metrics_port: Option<u16>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub max_age_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub requests_per_second: u32,
    pub burst_size: u32,
}

impl Config {
    pub fn load() -> Result<Self> {
        // Load .env file if it exists
        dotenvy::dotenv().ok();

        let environment = std::env::var("APP_ENVIRONMENT").unwrap_or_else(|_| "development".into());

        let config = ConfigBuilder::builder()
            // Start with default values
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 8080)?
            .set_default("server.graceful_shutdown_timeout_secs", 30)?
            .set_default("database.max_connections", 20)?
            .set_default("database.min_connections", 2)?
            .set_default("database.connect_timeout_secs", 30)?
            .set_default("database.idle_timeout_secs", 600)?
            .set_default("database.max_lifetime_secs", 1800)?
            .set_default("database.run_migrations", false)?
            .set_default("redis.connection_timeout_secs", 10)?
            .set_default("redis.response_timeout_secs", 5)?
            .set_default("meilisearch.timeout_secs", 30)?
            .set_default("auth.jwt_expiration_hours", 24)?
            .set_default("auth.refresh_token_expiration_days", 30)?
            .set_default("auth.password_reset_expiration_hours", 1)?
            .set_default("auth.email_verification_expiration_hours", 24)?
            .set_default("auth.max_login_attempts", 5)?
            .set_default("auth.lockout_duration_minutes", 15)?
            .set_default("storage.upload_dir", "/tmp/uploads")?
            .set_default("storage.max_upload_size_mb", 100)?
            .set_default(
                "storage.allowed_mime_types",
                vec!["image/jpeg", "image/png", "image/gif", "image/webp"],
            )?
            .set_default("telemetry.service_name", "tracker-platform")?
            .set_default("telemetry.environment", &environment)?
            .set_default("telemetry.log_level", "info")?
            .set_default("telemetry.log_format", "pretty")?
            .set_default("telemetry.metrics_enabled", true)?
            .set_default("telemetry.metrics_port", 9090)?
            .set_default(
                "cors.allowed_origins",
                vec!["http://localhost:3000", "http://localhost:8080"],
            )?
            .set_default(
                "cors.allowed_methods",
                vec!["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS"],
            )?
            .set_default(
                "cors.allowed_headers",
                vec!["Content-Type", "Authorization", "X-Request-ID"],
            )?
            .set_default("cors.max_age_secs", 3600)?
            .set_default("rate_limit.enabled", true)?
            .set_default("rate_limit.requests_per_second", 100)?
            .set_default("rate_limit.burst_size", 200)?
            // Load config file if it exists
            .add_source(File::with_name("config/default").required(false))
            .add_source(File::with_name(&format!("config/{}", environment)).required(false))
            // Override with environment variables (prefix: APP_)
            .add_source(
                Environment::with_prefix("APP")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()
            .context("Failed to build configuration")?;

        let config: Config = config
            .try_deserialize()
            .context("Failed to deserialize configuration")?;

        config.validate()?;

        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        // Validate server config
        if self.server.port == 0 {
            anyhow::bail!("Server port must be greater than 0");
        }

        // Validate database config
        if self.database.url.is_empty() {
            anyhow::bail!("Database URL is required");
        }
        if self.database.max_connections == 0 {
            anyhow::bail!("Database max_connections must be greater than 0");
        }

        // Validate Redis config
        if self.redis.url.is_empty() {
            anyhow::bail!("Redis URL is required");
        }

        // Validate Meilisearch config
        if self.meilisearch.url.is_empty() {
            anyhow::bail!("Meilisearch URL is required");
        }

        // Validate auth config
        if self.auth.jwt_secret.is_empty() {
            anyhow::bail!("JWT secret is required");
        }
        if self.auth.jwt_secret.len() < 32 {
            anyhow::bail!("JWT secret must be at least 32 characters");
        }

        // Validate storage config
        if self.storage.max_upload_size_mb == 0 {
            anyhow::bail!("Max upload size must be greater than 0");
        }

        // Validate telemetry config
        if !["trace", "debug", "info", "warn", "error"].contains(&self.telemetry.log_level.as_str())
        {
            anyhow::bail!(
                "Invalid log level: {}. Must be one of: trace, debug, info, warn, error",
                self.telemetry.log_level
            );
        }

        if !["json", "pretty"].contains(&self.telemetry.log_format.as_str()) {
            anyhow::bail!(
                "Invalid log format: {}. Must be one of: json, pretty",
                self.telemetry.log_format
            );
        }

        Ok(())
    }

    pub fn is_production(&self) -> bool {
        self.telemetry.environment == "production"
    }

    pub fn is_development(&self) -> bool {
        self.telemetry.environment == "development"
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                workers: None,
                max_connections: None,
                request_timeout_secs: Some(30),
                graceful_shutdown_timeout_secs: 30,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost/tracker".to_string(),
                max_connections: 20,
                min_connections: 2,
                connect_timeout_secs: 30,
                idle_timeout_secs: 600,
                max_lifetime_secs: 1800,
                run_migrations: false,
            },
            redis: RedisConfig {
                url: "redis://localhost:6379".to_string(),
                pool_size: None,
                connection_timeout_secs: 10,
                response_timeout_secs: 5,
            },
            meilisearch: MeilisearchConfig {
                url: "http://localhost:7700".to_string(),
                api_key: None,
                timeout_secs: 30,
            },
            kafka: None,
            auth: AuthConfig {
                jwt_secret: "change-me-in-production-min-32-chars".to_string(),
                jwt_expiration_hours: 24,
                refresh_token_expiration_days: 30,
                password_reset_expiration_hours: 1,
                email_verification_expiration_hours: 24,
                max_login_attempts: 5,
                lockout_duration_minutes: 15,
            },
            storage: StorageConfig {
                upload_dir: PathBuf::from("/tmp/uploads"),
                max_upload_size_mb: 100,
                allowed_mime_types: vec![
                    "image/jpeg".to_string(),
                    "image/png".to_string(),
                    "image/gif".to_string(),
                    "image/webp".to_string(),
                ],
            },
            telemetry: TelemetryConfig {
                service_name: "tracker-platform".to_string(),
                environment: "development".to_string(),
                log_level: "info".to_string(),
                log_format: "pretty".to_string(),
                otlp_endpoint: None,
                metrics_enabled: true,
                metrics_port: Some(9090),
            },
            cors: CorsConfig {
                allowed_origins: vec![
                    "http://localhost:3000".to_string(),
                    "http://localhost:8080".to_string(),
                ],
                allowed_methods: vec![
                    "GET".to_string(),
                    "POST".to_string(),
                    "PUT".to_string(),
                    "DELETE".to_string(),
                    "PATCH".to_string(),
                    "OPTIONS".to_string(),
                ],
                allowed_headers: vec![
                    "Content-Type".to_string(),
                    "Authorization".to_string(),
                    "X-Request-ID".to_string(),
                ],
                max_age_secs: 3600,
            },
            rate_limit: RateLimitConfig {
                enabled: true,
                requests_per_second: 100,
                burst_size: 200,
            },
        }
    }
}
