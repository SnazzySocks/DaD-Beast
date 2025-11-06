//! Error types for the search crate

use thiserror::Error;

/// Result type alias for search operations
pub type SearchResult<T> = Result<T, SearchError>;

/// Error types that can occur during search operations
#[derive(Error, Debug)]
pub enum SearchError {
    #[error("Meilisearch error: {0}")]
    Meilisearch(#[from] meilisearch_sdk::errors::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Index not found: {0}")]
    IndexNotFound(String),

    #[error("Invalid query: {0}")]
    InvalidQuery(String),

    #[error("Client not initialized")]
    ClientNotInitialized,

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Indexing error: {0}")]
    Indexing(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}
