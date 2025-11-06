//! JWT token generation and validation
//!
//! This module provides JSON Web Token (JWT) functionality for authentication,
//! supporting both access tokens (short-lived) and refresh tokens (long-lived).

use crate::permissions::Permission;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur during JWT operations
#[derive(Debug, Error)]
pub enum JwtError {
    #[error("Failed to generate token: {0}")]
    TokenGeneration(String),

    #[error("Invalid token: {0}")]
    InvalidToken(String),

    #[error("Token has expired")]
    TokenExpired,

    #[error("Token signature verification failed")]
    SignatureVerification,

    #[error("Missing required claim: {0}")]
    MissingClaim(String),
}

/// JWT token type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    /// Short-lived access token (15 minutes)
    Access,
    /// Long-lived refresh token (7 days)
    Refresh,
}

impl TokenType {
    /// Get the default expiration duration for this token type
    pub fn expiration_duration(&self) -> Duration {
        match self {
            TokenType::Access => Duration::minutes(15),
            TokenType::Refresh => Duration::days(7),
        }
    }
}

/// JWT claims structure
///
/// Contains all information needed to authenticate and authorize a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: Uuid,
    /// Token type (access or refresh)
    pub token_type: TokenType,
    /// User permissions
    pub permissions: Vec<Permission>,
    /// Token expiration time (Unix timestamp)
    pub exp: i64,
    /// Issued at time (Unix timestamp)
    pub iat: i64,
    /// JWT ID (for token revocation)
    pub jti: Uuid,
    /// Issuer
    pub iss: String,
}

impl Claims {
    /// Create new claims for an access token
    pub fn new_access_token(user_id: Uuid, permissions: Vec<Permission>) -> Self {
        let now = Utc::now();
        let exp = now + TokenType::Access.expiration_duration();

        Self {
            sub: user_id,
            token_type: TokenType::Access,
            permissions,
            exp: exp.timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4(),
            iss: "tracker-platform".to_string(),
        }
    }

    /// Create new claims for a refresh token
    pub fn new_refresh_token(user_id: Uuid) -> Self {
        let now = Utc::now();
        let exp = now + TokenType::Refresh.expiration_duration();

        Self {
            sub: user_id,
            token_type: TokenType::Refresh,
            permissions: Vec::new(), // Refresh tokens don't carry permissions
            exp: exp.timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4(),
            iss: "tracker-platform".to_string(),
        }
    }

    /// Create custom claims with specified expiration
    pub fn new_custom(
        user_id: Uuid,
        token_type: TokenType,
        permissions: Vec<Permission>,
        expiration: Duration,
    ) -> Self {
        let now = Utc::now();
        let exp = now + expiration;

        Self {
            sub: user_id,
            token_type,
            permissions,
            exp: exp.timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4(),
            iss: "tracker-platform".to_string(),
        }
    }

    /// Check if the token has expired
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }

    /// Get the user ID from claims
    pub fn user_id(&self) -> Uuid {
        self.sub
    }

    /// Get the token ID from claims
    pub fn token_id(&self) -> Uuid {
        self.jti
    }
}

/// JWT token manager
///
/// Handles token generation, validation, and revocation.
pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl JwtManager {
    /// Create a new JWT manager with a secret key
    ///
    /// # Security Note
    /// The secret should be a strong, randomly generated string (at least 32 bytes).
    /// Store it securely (e.g., in environment variables, not in code).
    pub fn new(secret: &str) -> Self {
        let encoding_key = EncodingKey::from_secret(secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(secret.as_bytes());
        let mut validation = Validation::default();
        validation.set_issuer(&["tracker-platform"]);

        Self {
            encoding_key,
            decoding_key,
            validation,
        }
    }

    /// Generate a JWT token from claims
    pub fn generate_token(&self, claims: &Claims) -> Result<String, JwtError> {
        encode(&Header::default(), claims, &self.encoding_key)
            .map_err(|e| JwtError::TokenGeneration(e.to_string()))
    }

    /// Validate and decode a JWT token
    pub fn validate_token(&self, token: &str) -> Result<Claims, JwtError> {
        let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => JwtError::TokenExpired,
                jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                    JwtError::SignatureVerification
                }
                _ => JwtError::InvalidToken(e.to_string()),
            })?;

        Ok(token_data.claims)
    }

    /// Generate an access token
    pub fn generate_access_token(
        &self,
        user_id: Uuid,
        permissions: Vec<Permission>,
    ) -> Result<String, JwtError> {
        let claims = Claims::new_access_token(user_id, permissions);
        self.generate_token(&claims)
    }

    /// Generate a refresh token
    pub fn generate_refresh_token(&self, user_id: Uuid) -> Result<String, JwtError> {
        let claims = Claims::new_refresh_token(user_id);
        self.generate_token(&claims)
    }

    /// Generate both access and refresh tokens
    pub fn generate_token_pair(
        &self,
        user_id: Uuid,
        permissions: Vec<Permission>,
    ) -> Result<TokenPair, JwtError> {
        let access_token = self.generate_access_token(user_id, permissions)?;
        let refresh_token = self.generate_refresh_token(user_id)?;

        Ok(TokenPair {
            access_token,
            refresh_token,
        })
    }

    /// Refresh an access token using a refresh token
    ///
    /// Validates the refresh token and generates a new access token.
    pub fn refresh_access_token(
        &self,
        refresh_token: &str,
        permissions: Vec<Permission>,
    ) -> Result<String, JwtError> {
        let claims = self.validate_token(refresh_token)?;

        // Verify this is actually a refresh token
        if claims.token_type != TokenType::Refresh {
            return Err(JwtError::InvalidToken(
                "Expected refresh token".to_string(),
            ));
        }

        // Generate new access token
        self.generate_access_token(claims.user_id(), permissions)
    }
}

/// A pair of access and refresh tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
}

/// Token revocation list manager
///
/// Tracks revoked tokens using Redis for distributed systems.
pub struct TokenRevocationList {
    redis_client: redis::Client,
    key_prefix: String,
}

impl TokenRevocationList {
    /// Create a new token revocation list
    pub fn new(redis_client: redis::Client) -> Self {
        Self {
            redis_client,
            key_prefix: "auth:revoked_token".to_string(),
        }
    }

    /// Revoke a token
    ///
    /// Stores the token ID in Redis with an expiration matching the token's expiry.
    pub async fn revoke_token(&self, token_id: Uuid, expires_at: i64) -> Result<(), JwtError> {
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| JwtError::InvalidToken(format!("Redis connection error: {}", e)))?;

        let key = format!("{}:{}", self.key_prefix, token_id);
        let ttl = (expires_at - Utc::now().timestamp()).max(0) as usize;

        redis::cmd("SETEX")
            .arg(&key)
            .arg(ttl)
            .arg("revoked")
            .query_async(&mut conn)
            .await
            .map_err(|e| JwtError::InvalidToken(format!("Failed to revoke token: {}", e)))?;

        Ok(())
    }

    /// Check if a token is revoked
    pub async fn is_revoked(&self, token_id: Uuid) -> Result<bool, JwtError> {
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| JwtError::InvalidToken(format!("Redis connection error: {}", e)))?;

        let key = format!("{}:{}", self.key_prefix, token_id);

        let exists: bool = redis::cmd("EXISTS")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .map_err(|e| JwtError::InvalidToken(format!("Failed to check revocation: {}", e)))?;

        Ok(exists)
    }

    /// Revoke all tokens for a user
    ///
    /// This is typically used when a user changes their password or logs out from all devices.
    pub async fn revoke_user_tokens(&self, user_id: Uuid, ttl: i64) -> Result<(), JwtError> {
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| JwtError::InvalidToken(format!("Redis connection error: {}", e)))?;

        let key = format!("{}:user:{}", self.key_prefix, user_id);
        let ttl_secs = ttl.max(0) as usize;

        redis::cmd("SETEX")
            .arg(&key)
            .arg(ttl_secs)
            .arg(Utc::now().timestamp())
            .query_async(&mut conn)
            .await
            .map_err(|e| JwtError::InvalidToken(format!("Failed to revoke user tokens: {}", e)))?;

        Ok(())
    }

    /// Check if all tokens for a user are revoked
    pub async fn is_user_revoked(&self, user_id: Uuid, issued_at: i64) -> Result<bool, JwtError> {
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| JwtError::InvalidToken(format!("Redis connection error: {}", e)))?;

        let key = format!("{}:user:{}", self.key_prefix, user_id);

        let revoke_time: Option<i64> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .map_err(|e| JwtError::InvalidToken(format!("Failed to check user revocation: {}", e)))?;

        Ok(revoke_time.map_or(false, |rt| issued_at < rt))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SECRET: &str = "test-secret-key-at-least-32-characters-long-for-security";

    #[test]
    fn test_token_generation() {
        let manager = JwtManager::new(TEST_SECRET);
        let user_id = Uuid::new_v4();
        let permissions = vec![Permission::Download, Permission::UploadTorrent];

        let token = manager.generate_access_token(user_id, permissions).unwrap();
        assert!(!token.is_empty());
    }

    #[test]
    fn test_token_validation() {
        let manager = JwtManager::new(TEST_SECRET);
        let user_id = Uuid::new_v4();
        let permissions = vec![Permission::Download];

        let token = manager.generate_access_token(user_id, permissions.clone()).unwrap();
        let claims = manager.validate_token(&token).unwrap();

        assert_eq!(claims.user_id(), user_id);
        assert_eq!(claims.token_type, TokenType::Access);
        assert_eq!(claims.permissions, permissions);
    }

    #[test]
    fn test_token_pair_generation() {
        let manager = JwtManager::new(TEST_SECRET);
        let user_id = Uuid::new_v4();
        let permissions = vec![Permission::Download];

        let pair = manager.generate_token_pair(user_id, permissions).unwrap();
        assert!(!pair.access_token.is_empty());
        assert!(!pair.refresh_token.is_empty());

        // Validate both tokens
        let access_claims = manager.validate_token(&pair.access_token).unwrap();
        let refresh_claims = manager.validate_token(&pair.refresh_token).unwrap();

        assert_eq!(access_claims.token_type, TokenType::Access);
        assert_eq!(refresh_claims.token_type, TokenType::Refresh);
    }

    #[test]
    fn test_refresh_token_flow() {
        let manager = JwtManager::new(TEST_SECRET);
        let user_id = Uuid::new_v4();
        let permissions = vec![Permission::Download];

        let refresh_token = manager.generate_refresh_token(user_id).unwrap();
        let new_access_token = manager
            .refresh_access_token(&refresh_token, permissions.clone())
            .unwrap();

        let claims = manager.validate_token(&new_access_token).unwrap();
        assert_eq!(claims.user_id(), user_id);
        assert_eq!(claims.permissions, permissions);
    }

    #[test]
    fn test_invalid_token() {
        let manager = JwtManager::new(TEST_SECRET);
        let result = manager.validate_token("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn test_expired_token() {
        let manager = JwtManager::new(TEST_SECRET);
        let user_id = Uuid::new_v4();

        // Create an expired token
        let claims = Claims::new_custom(
            user_id,
            TokenType::Access,
            vec![],
            Duration::seconds(-1), // Already expired
        );

        let token = manager.generate_token(&claims).unwrap();
        let result = manager.validate_token(&token);

        assert!(matches!(result, Err(JwtError::TokenExpired)));
    }

    #[test]
    fn test_claims_expiry_check() {
        let user_id = Uuid::new_v4();

        let valid_claims = Claims::new_access_token(user_id, vec![]);
        assert!(!valid_claims.is_expired());

        let expired_claims = Claims::new_custom(
            user_id,
            TokenType::Access,
            vec![],
            Duration::seconds(-1),
        );
        assert!(expired_claims.is_expired());
    }
}
