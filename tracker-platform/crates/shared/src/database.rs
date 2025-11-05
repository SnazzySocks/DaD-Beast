//! Database connection pool and utilities.
//!
//! This module provides PostgreSQL connection pooling, migration management,
//! and health check functionality.

use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::migrate::MigrateDatabase;
use std::time::Duration;
use tracing::{info, warn};

use crate::config::Config;
use crate::error::{AppResult, DatabaseError};

/// Database connection pool wrapper
#[derive(Debug, Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Create a new database connection pool
    ///
    /// # Arguments
    ///
    /// * `config` - Application configuration containing database URL and pool settings
    ///
    /// # Errors
    ///
    /// Returns an error if the connection pool cannot be created
    ///
    /// # Example
    ///
    /// ```no_run
    /// use shared::config::Config;
    /// use shared::database::Database;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = Config::from_env()?;
    /// let db = Database::new(&config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(config: &Config) -> AppResult<Self> {
        info!("Creating database connection pool");

        let pool = PgPoolOptions::new()
            .max_connections(config.db_max_connections)
            .min_connections(config.db_min_connections)
            .acquire_timeout(Duration::from_secs(30))
            .idle_timeout(Duration::from_secs(600))
            .max_lifetime(Duration::from_secs(1800))
            .connect(&config.database_url)
            .await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;

        info!(
            "Database connection pool created with {} max connections",
            config.db_max_connections
        );

        Ok(Self { pool })
    }

    /// Create a new database connection pool from a database URL
    ///
    /// # Arguments
    ///
    /// * `database_url` - PostgreSQL connection URL
    ///
    /// # Errors
    ///
    /// Returns an error if the connection pool cannot be created
    pub async fn from_url(database_url: &str) -> AppResult<Self> {
        info!("Creating database connection pool from URL");

        let pool = PgPoolOptions::new()
            .max_connections(10)
            .min_connections(2)
            .acquire_timeout(Duration::from_secs(30))
            .connect(database_url)
            .await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;

        Ok(Self { pool })
    }

    /// Get a reference to the underlying connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Run database migrations
    ///
    /// # Errors
    ///
    /// Returns an error if migrations fail
    pub async fn run_migrations(&self) -> AppResult<()> {
        info!("Running database migrations");

        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| DatabaseError::MigrationFailed(e.to_string()))?;

        info!("Database migrations completed successfully");
        Ok(())
    }

    /// Check database connection health
    ///
    /// # Errors
    ///
    /// Returns an error if the health check fails
    pub async fn health_check(&self) -> AppResult<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    /// Get the number of active connections in the pool
    pub fn active_connections(&self) -> usize {
        self.pool.size() as usize
    }

    /// Get the number of idle connections in the pool
    pub fn idle_connections(&self) -> usize {
        self.pool.num_idle()
    }

    /// Close the database connection pool
    pub async fn close(&self) {
        info!("Closing database connection pool");
        self.pool.close().await;
    }
}

/// Create the database if it doesn't exist
///
/// # Arguments
///
/// * `database_url` - PostgreSQL connection URL
///
/// # Errors
///
/// Returns an error if database creation fails
pub async fn create_database_if_not_exists(database_url: &str) -> AppResult<()> {
    if !sqlx::Postgres::database_exists(database_url)
        .await
        .unwrap_or(false)
    {
        info!("Database does not exist, creating...");
        sqlx::Postgres::create_database(database_url)
            .await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;
        info!("Database created successfully");
    } else {
        info!("Database already exists");
    }

    Ok(())
}

/// Drop the database (use with caution!)
///
/// # Arguments
///
/// * `database_url` - PostgreSQL connection URL
///
/// # Errors
///
/// Returns an error if database deletion fails
pub async fn drop_database(database_url: &str) -> AppResult<()> {
    warn!("Dropping database - this operation cannot be undone!");

    if sqlx::Postgres::database_exists(database_url)
        .await
        .unwrap_or(false)
    {
        sqlx::Postgres::drop_database(database_url)
            .await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;
        info!("Database dropped successfully");
    } else {
        info!("Database does not exist, nothing to drop");
    }

    Ok(())
}

/// Execute a transaction with automatic rollback on error
///
/// # Example
///
/// ```no_run
/// use shared::database::{Database, transaction};
///
/// # async fn example(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
/// transaction(db.pool(), |tx| async move {
///     sqlx::query("INSERT INTO users (username) VALUES ($1)")
///         .bind("alice")
///         .execute(&mut **tx)
///         .await?;
///
///     sqlx::query("UPDATE stats SET user_count = user_count + 1")
///         .execute(&mut **tx)
///         .await?;
///
///     Ok(())
/// }).await?;
/// # Ok(())
/// # }
/// ```
pub async fn transaction<F, Fut, T>(pool: &PgPool, f: F) -> AppResult<T>
where
    F: FnOnce(&mut sqlx::Transaction<'_, sqlx::Postgres>) -> Fut,
    Fut: std::future::Future<Output = Result<T, sqlx::Error>>,
{
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| DatabaseError::TransactionFailed(e.to_string()))?;

    let result = f(&mut tx)
        .await
        .map_err(|e| DatabaseError::TransactionFailed(e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| DatabaseError::TransactionFailed(e.to_string()))?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires a running PostgreSQL instance
    async fn test_database_connection() {
        let database_url = "postgresql://postgres:postgres@localhost/test_db";
        let db = Database::from_url(database_url).await;
        assert!(db.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn test_health_check() {
        let database_url = "postgresql://postgres:postgres@localhost/test_db";
        let db = Database::from_url(database_url).await.unwrap();
        let result = db.health_check().await;
        assert!(result.is_ok());
    }
}
