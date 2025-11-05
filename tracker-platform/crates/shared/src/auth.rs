//! Authentication utilities for the unified tracker platform.
//!
//! This module provides JWT token management, password hashing/verification,
//! and passkey generation for tracker authentication.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppResult, AuthError};
use crate::types::{Passkey, UserId};

/// JWT claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Username
    pub username: String,
    /// User role
    pub role: String,
    /// Issued at timestamp
    pub iat: i64,
    /// Expiration timestamp
    pub exp: i64,
}

impl Claims {
    /// Create new claims for a user
    ///
    /// # Arguments
    ///
    /// * `user_id` - User identifier
    /// * `username` - Username
    /// * `role` - User role
    /// * `expires_in` - Duration until token expires
    pub fn new(user_id: UserId, username: String, role: String, expires_in: Duration) -> Self {
        let now = Utc::now();
        let exp = now + expires_in;

        Self {
            sub: user_id.to_string(),
            username,
            role,
            iat: now.timestamp(),
            exp: exp.timestamp(),
        }
    }

    /// Get user ID from claims
    pub fn user_id(&self) -> Result<UserId, uuid::Error> {
        Uuid::parse_str(&self.sub).map(UserId)
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }

    /// Check if user is admin
    pub fn is_admin(&self) -> bool {
        self.role.eq_ignore_ascii_case("admin")
    }

    /// Check if user is moderator or admin
    pub fn is_moderator(&self) -> bool {
        self.is_admin() || self.role.eq_ignore_ascii_case("moderator")
    }
}

/// JWT token manager
pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl JwtManager {
    /// Create a new JWT manager
    ///
    /// # Arguments
    ///
    /// * `secret` - Secret key for signing tokens
    pub fn new(secret: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            validation: Validation::default(),
        }
    }

    /// Generate a JWT token
    ///
    /// # Arguments
    ///
    /// * `claims` - JWT claims
    ///
    /// # Errors
    ///
    /// Returns an error if token generation fails
    pub fn generate_token(&self, claims: &Claims) -> AppResult<String> {
        let token = encode(&Header::default(), claims, &self.encoding_key)?;
        Ok(token)
    }

    /// Validate and decode a JWT token
    ///
    /// # Arguments
    ///
    /// * `token` - JWT token string
    ///
    /// # Errors
    ///
    /// Returns an error if token is invalid or expired
    pub fn validate_token(&self, token: &str) -> AppResult<Claims> {
        let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)?;
        Ok(token_data.claims)
    }
}

/// Password hasher using Argon2
pub struct PasswordHasher;

impl PasswordHasher {
    /// Hash a password using Argon2
    ///
    /// # Arguments
    ///
    /// * `password` - Plain text password
    ///
    /// # Errors
    ///
    /// Returns an error if hashing fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use shared::auth::PasswordHasher;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let hash = PasswordHasher::hash("my_password")?;
    /// println!("Password hash: {}", hash);
    /// # Ok(())
    /// # }
    /// ```
    pub fn hash(password: &str) -> AppResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AuthError::InvalidCredentials)?;

        Ok(password_hash.to_string())
    }

    /// Verify a password against a hash
    ///
    /// # Arguments
    ///
    /// * `password` - Plain text password
    /// * `hash` - Password hash to verify against
    ///
    /// # Errors
    ///
    /// Returns an error if verification fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use shared::auth::PasswordHasher;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let hash = PasswordHasher::hash("my_password")?;
    /// let is_valid = PasswordHasher::verify("my_password", &hash)?;
    /// assert!(is_valid);
    /// # Ok(())
    /// # }
    /// ```
    pub fn verify(password: &str, hash: &str) -> AppResult<bool> {
        let parsed_hash =
            PasswordHash::new(hash).map_err(|_| AuthError::InvalidCredentials)?;

        let argon2 = Argon2::default();

        match argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

/// Passkey generator for tracker authentication
pub struct PasskeyGenerator;

impl PasskeyGenerator {
    /// Generate a new random passkey
    ///
    /// # Example
    ///
    /// ```
    /// use shared::auth::PasskeyGenerator;
    ///
    /// let passkey = PasskeyGenerator::generate();
    /// assert_eq!(passkey.as_str().len(), 32);
    /// ```
    pub fn generate() -> Passkey {
        Passkey::generate()
    }

    /// Validate a passkey format
    ///
    /// # Arguments
    ///
    /// * `passkey` - Passkey string to validate
    ///
    /// # Errors
    ///
    /// Returns an error if the passkey format is invalid
    pub fn validate(passkey: &str) -> AppResult<()> {
        if passkey.len() != 32 {
            return Err(AuthError::InvalidPasskey.into());
        }

        if !passkey.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(AuthError::InvalidPasskey.into());
        }

        Ok(())
    }
}

/// Token pair for access and refresh tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    /// Access token (short-lived)
    pub access_token: String,
    /// Refresh token (long-lived)
    pub refresh_token: String,
    /// Token type (always "Bearer")
    pub token_type: String,
    /// Expiration time in seconds
    pub expires_in: i64,
}

impl TokenPair {
    /// Create a new token pair
    pub fn new(
        access_token: String,
        refresh_token: String,
        expires_in: i64,
    ) -> Self {
        Self {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
        }
    }
}

/// Generate access and refresh tokens for a user
///
/// # Arguments
///
/// * `user_id` - User identifier
/// * `username` - Username
/// * `role` - User role
/// * `jwt_secret` - Secret for access token
/// * `jwt_refresh_secret` - Secret for refresh token
/// * `access_expiration` - Access token expiration in seconds
/// * `refresh_expiration` - Refresh token expiration in seconds
///
/// # Errors
///
/// Returns an error if token generation fails
pub fn generate_token_pair(
    user_id: UserId,
    username: String,
    role: String,
    jwt_secret: &str,
    jwt_refresh_secret: &str,
    access_expiration: i64,
    refresh_expiration: i64,
) -> AppResult<TokenPair> {
    let jwt_manager = JwtManager::new(jwt_secret);
    let refresh_manager = JwtManager::new(jwt_refresh_secret);

    let access_claims = Claims::new(
        user_id,
        username.clone(),
        role.clone(),
        Duration::seconds(access_expiration),
    );

    let refresh_claims = Claims::new(
        user_id,
        username,
        role,
        Duration::seconds(refresh_expiration),
    );

    let access_token = jwt_manager.generate_token(&access_claims)?;
    let refresh_token = refresh_manager.generate_token(&refresh_claims)?;

    Ok(TokenPair::new(access_token, refresh_token, access_expiration))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "secure_password_123";
        let hash = PasswordHasher::hash(password).unwrap();

        assert!(PasswordHasher::verify(password, &hash).unwrap());
        assert!(!PasswordHasher::verify("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_jwt_token() {
        let secret = "test_secret_key";
        let jwt_manager = JwtManager::new(secret);

        let user_id = UserId::new();
        let claims = Claims::new(
            user_id,
            "testuser".to_string(),
            "user".to_string(),
            Duration::hours(1),
        );

        let token = jwt_manager.generate_token(&claims).unwrap();
        let decoded_claims = jwt_manager.validate_token(&token).unwrap();

        assert_eq!(decoded_claims.sub, claims.sub);
        assert_eq!(decoded_claims.username, claims.username);
        assert!(!decoded_claims.is_expired());
    }

    #[test]
    fn test_passkey_generation() {
        let passkey = PasskeyGenerator::generate();
        assert_eq!(passkey.as_str().len(), 32);
        assert!(PasskeyGenerator::validate(passkey.as_str()).is_ok());
    }

    #[test]
    fn test_passkey_validation() {
        assert!(PasskeyGenerator::validate("0123456789abcdef0123456789abcdef").is_ok());
        assert!(PasskeyGenerator::validate("invalid").is_err());
        assert!(PasskeyGenerator::validate("gggggggggggggggggggggggggggggggg").is_err()); // Invalid hex
    }

    #[test]
    fn test_claims_roles() {
        let user_id = UserId::new();

        let admin_claims = Claims::new(
            user_id,
            "admin".to_string(),
            "admin".to_string(),
            Duration::hours(1),
        );
        assert!(admin_claims.is_admin());
        assert!(admin_claims.is_moderator());

        let mod_claims = Claims::new(
            user_id,
            "mod".to_string(),
            "moderator".to_string(),
            Duration::hours(1),
        );
        assert!(!mod_claims.is_admin());
        assert!(mod_claims.is_moderator());

        let user_claims = Claims::new(
            user_id,
            "user".to_string(),
            "user".to_string(),
            Duration::hours(1),
        );
        assert!(!user_claims.is_admin());
        assert!(!user_claims.is_moderator());
    }
}
