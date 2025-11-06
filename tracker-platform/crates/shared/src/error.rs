//! Common error types for the unified tracker platform.
//!
//! This module provides a unified error handling system with automatic conversion
//! to HTTP responses for use with Axum web framework.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use std::fmt;

/// Result type alias using AppError
pub type AppResult<T> = Result<T, AppError>;

/// Application-wide error type.
///
/// This enum represents all possible errors that can occur in the application.
/// It implements `IntoResponse` for automatic conversion to HTTP responses.
#[derive(Debug)]
pub enum AppError {
    /// Database-related errors
    Database(DatabaseError),
    /// Authentication/authorization errors
    Auth(AuthError),
    /// Validation errors
    Validation(ValidationError),
    /// Resource not found
    NotFound(String),
    /// Resource already exists
    AlreadyExists(String),
    /// Internal server error
    Internal(String),
    /// Bad request
    BadRequest(String),
    /// Rate limit exceeded
    RateLimitExceeded,
    /// Service unavailable
    ServiceUnavailable(String),
    /// External service error
    ExternalService(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(e) => write!(f, "Database error: {}", e),
            Self::Auth(e) => write!(f, "Authentication error: {}", e),
            Self::Validation(e) => write!(f, "Validation error: {}", e),
            Self::NotFound(msg) => write!(f, "Not found: {}", msg),
            Self::AlreadyExists(msg) => write!(f, "Already exists: {}", msg),
            Self::Internal(msg) => write!(f, "Internal error: {}", msg),
            Self::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            Self::RateLimitExceeded => write!(f, "Rate limit exceeded"),
            Self::ServiceUnavailable(msg) => write!(f, "Service unavailable: {}", msg),
            Self::ExternalService(msg) => write!(f, "External service error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Self::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred")
            }
            Self::Auth(e) => {
                let status = match e {
                    AuthError::InvalidCredentials => StatusCode::UNAUTHORIZED,
                    AuthError::TokenExpired => StatusCode::UNAUTHORIZED,
                    AuthError::InvalidToken => StatusCode::UNAUTHORIZED,
                    AuthError::MissingToken => StatusCode::UNAUTHORIZED,
                    AuthError::PermissionDenied => StatusCode::FORBIDDEN,
                    _ => StatusCode::UNAUTHORIZED,
                };
                (status, e.to_string().as_str())
            }
            Self::Validation(e) => (StatusCode::BAD_REQUEST, e.to_string().as_str()),
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg.as_str()),
            Self::AlreadyExists(msg) => (StatusCode::CONFLICT, msg.as_str()),
            Self::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            Self::RateLimitExceeded => (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded"),
            Self::ServiceUnavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg.as_str()),
            Self::ExternalService(msg) => {
                tracing::error!("External service error: {}", msg);
                (StatusCode::BAD_GATEWAY, "External service error")
            }
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

/// Database error types
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Query failed: {0}")]
    QueryFailed(String),

    #[error("Migration failed: {0}")]
    MigrationFailed(String),

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Record not found")]
    NotFound,

    #[error("Duplicate entry: {0}")]
    DuplicateEntry(String),

    #[error("Pool error: {0}")]
    PoolError(String),
}

/// Authentication error types
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Missing token")]
    MissingToken,

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Invalid passkey")]
    InvalidPasskey,

    #[error("Account disabled")]
    AccountDisabled,

    #[error("Email not verified")]
    EmailNotVerified,

    #[error("Two-factor authentication required")]
    TwoFactorRequired,

    #[error("Invalid two-factor code")]
    InvalidTwoFactorCode,
}

/// Validation error types
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid email format")]
    InvalidEmail,

    #[error("Password too weak: {0}")]
    WeakPassword(String),

    #[error("Invalid field: {field} - {message}")]
    InvalidField { field: String, message: String },

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid info hash format")]
    InvalidInfoHash,

    #[error("Invalid UUID format")]
    InvalidUuid,

    #[error("Value out of range: {0}")]
    OutOfRange(String),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

// Conversions from common error types

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Self::NotFound("Record not found".to_string()),
            sqlx::Error::Database(db_err) => {
                // Check for unique constraint violation
                if let Some(code) = db_err.code() {
                    if code == "23505" {
                        // PostgreSQL unique violation
                        return Self::AlreadyExists("Record already exists".to_string());
                    }
                }
                Self::Database(DatabaseError::QueryFailed(db_err.to_string()))
            }
            sqlx::Error::PoolTimedOut => {
                Self::Database(DatabaseError::PoolError("Pool timeout".to_string()))
            }
            _ => Self::Database(DatabaseError::QueryFailed(err.to_string())),
        }
    }
}

impl From<redis::RedisError> for AppError {
    fn from(err: redis::RedisError) -> Self {
        tracing::error!("Redis error: {:?}", err);
        Self::ServiceUnavailable(format!("Cache service error: {}", err))
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        use jsonwebtoken::errors::ErrorKind;

        match err.kind() {
            ErrorKind::ExpiredSignature => Self::Auth(AuthError::TokenExpired),
            ErrorKind::InvalidToken => Self::Auth(AuthError::InvalidToken),
            _ => Self::Auth(AuthError::InvalidToken),
        }
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(errors: validator::ValidationErrors) -> Self {
        let messages: Vec<String> = errors
            .field_errors()
            .iter()
            .map(|(field, errs)| {
                let msgs: Vec<String> = errs
                    .iter()
                    .filter_map(|e| e.message.as_ref().map(|m| m.to_string()))
                    .collect();
                format!("{}: {}", field, msgs.join(", "))
            })
            .collect();

        Self::Validation(ValidationError::InvalidFormat(messages.join("; ")))
    }
}

impl From<DatabaseError> for AppError {
    fn from(err: DatabaseError) -> Self {
        Self::Database(err)
    }
}

impl From<AuthError> for AppError {
    fn from(err: AuthError) -> Self {
        Self::Auth(err)
    }
}

impl From<ValidationError> for AppError {
    fn from(err: ValidationError) -> Self {
        Self::Validation(err)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        Self::Internal(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AppError::NotFound("User".to_string());
        assert_eq!(err.to_string(), "Not found: User");

        let err = AppError::Auth(AuthError::InvalidCredentials);
        assert_eq!(err.to_string(), "Authentication error: Invalid credentials");
    }

    #[test]
    fn test_validation_error() {
        let err = ValidationError::InvalidEmail;
        assert_eq!(err.to_string(), "Invalid email format");
    }
}
