//! Configuration management for the unified tracker platform.
//!
//! This module provides a centralized configuration system that loads settings
//! from environment variables using dotenvy.

use std::env;

/// Application configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    /// PostgreSQL database URL
    pub database_url: String,
    /// Redis URL for caching and session management
    pub redis_url: String,
    /// JWT secret key for token signing
    pub jwt_secret: String,
    /// JWT refresh secret key for refresh token signing
    pub jwt_refresh_secret: String,
    /// JWT token expiration time in seconds (default: 3600 = 1 hour)
    pub jwt_expiration: i64,
    /// JWT refresh token expiration time in seconds (default: 604800 = 7 days)
    pub jwt_refresh_expiration: i64,
    /// Server host address
    pub server_host: String,
    /// Server port
    pub server_port: u16,
    /// Application environment (development, staging, production)
    pub environment: String,
    /// CORS allowed origins (comma-separated)
    pub cors_origins: Vec<String>,
    /// Maximum connections in database pool
    pub db_max_connections: u32,
    /// Minimum connections in database pool
    pub db_min_connections: u32,
    /// Redis pool size
    pub redis_pool_size: u32,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// # Errors
    ///
    /// Returns an error if required environment variables are missing or invalid.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use shared::config::Config;
    ///
    /// let config = Config::from_env().expect("Failed to load configuration");
    /// println!("Server will run on {}:{}", config.server_host, config.server_port);
    /// ```
    pub fn from_env() -> Result<Self, ConfigError> {
        // Load .env file if present
        dotenvy::dotenv().ok();

        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .map_err(|_| ConfigError::MissingVar("DATABASE_URL"))?,
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            jwt_secret: env::var("JWT_SECRET")
                .map_err(|_| ConfigError::MissingVar("JWT_SECRET"))?,
            jwt_refresh_secret: env::var("JWT_REFRESH_SECRET")
                .map_err(|_| ConfigError::MissingVar("JWT_REFRESH_SECRET"))?,
            jwt_expiration: env::var("JWT_EXPIRATION")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("JWT_EXPIRATION"))?,
            jwt_refresh_expiration: env::var("JWT_REFRESH_EXPIRATION")
                .unwrap_or_else(|_| "604800".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("JWT_REFRESH_EXPIRATION"))?,
            server_host: env::var("SERVER_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("SERVER_PORT"))?,
            environment: env::var("ENVIRONMENT")
                .unwrap_or_else(|_| "development".to_string()),
            cors_origins: env::var("CORS_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:3000".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            db_max_connections: env::var("DB_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("DB_MAX_CONNECTIONS"))?,
            db_min_connections: env::var("DB_MIN_CONNECTIONS")
                .unwrap_or_else(|_| "2".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("DB_MIN_CONNECTIONS"))?,
            redis_pool_size: env::var("REDIS_POOL_SIZE")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("REDIS_POOL_SIZE"))?,
        })
    }

    /// Check if running in production environment
    pub fn is_production(&self) -> bool {
        self.environment.eq_ignore_ascii_case("production")
    }

    /// Check if running in development environment
    pub fn is_development(&self) -> bool {
        self.environment.eq_ignore_ascii_case("development")
    }

    /// Get the full server address (host:port)
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}

/// Configuration error types.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Missing required environment variable
    #[error("Missing required environment variable: {0}")]
    MissingVar(&'static str),

    /// Invalid value for environment variable
    #[error("Invalid value for environment variable: {0}")]
    InvalidValue(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_production() {
        let config = Config {
            database_url: String::new(),
            redis_url: String::new(),
            jwt_secret: String::new(),
            jwt_refresh_secret: String::new(),
            jwt_expiration: 3600,
            jwt_refresh_expiration: 604800,
            server_host: String::new(),
            server_port: 8080,
            environment: "production".to_string(),
            cors_origins: vec![],
            db_max_connections: 10,
            db_min_connections: 2,
            redis_pool_size: 10,
        };

        assert!(config.is_production());
        assert!(!config.is_development());
    }

    #[test]
    fn test_server_address() {
        let config = Config {
            database_url: String::new(),
            redis_url: String::new(),
            jwt_secret: String::new(),
            jwt_refresh_secret: String::new(),
            jwt_expiration: 3600,
            jwt_refresh_expiration: 604800,
            server_host: "127.0.0.1".to_string(),
            server_port: 3000,
            environment: String::new(),
            cors_origins: vec![],
            db_max_connections: 10,
            db_min_connections: 2,
            redis_pool_size: 10,
        };

        assert_eq!(config.server_address(), "127.0.0.1:3000");
    }
}
