//! User registration
//!
//! This module handles new user registration, including validation,
//! password hashing, passkey generation for BitTorrent, and optional
//! email verification.

use crate::password::{hash_password, validate_password_strength, PasswordError};
use crate::permissions::{PermissionSet, Role};
use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;
use validator::{Validate, ValidationError};

/// Errors that can occur during registration
#[derive(Debug, Error)]
pub enum RegistrationError {
    #[error("Email already registered")]
    EmailExists,

    #[error("Username already taken")]
    UsernameExists,

    #[error("Invalid email format")]
    InvalidEmail,

    #[error("Password validation failed: {0}")]
    InvalidPassword(String),

    #[error("Validation error: {0}")]
    ValidationFailed(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Failed to send verification email: {0}")]
    EmailSendFailed(String),

    #[error("Invitation code is invalid or has been used")]
    InvalidInviteCode,
}

impl From<PasswordError> for RegistrationError {
    fn from(err: PasswordError) -> Self {
        RegistrationError::InvalidPassword(err.to_string())
    }
}

/// User registration request
#[derive(Debug, Clone, Validate, Deserialize)]
pub struct RegisterRequest {
    /// Email address (must be unique)
    #[validate(email(message = "Invalid email address"))]
    pub email: String,

    /// Username (optional, can be auto-generated)
    #[validate(length(min = 3, max = 30, message = "Username must be 3-30 characters"))]
    pub username: Option<String>,

    /// Password (must meet strength requirements)
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,

    /// Password confirmation
    pub password_confirmation: String,

    /// Invitation code (if registration is invite-only)
    pub invite_code: Option<String>,
}

impl RegisterRequest {
    /// Validate password confirmation matches
    pub fn validate_password_match(&self) -> Result<(), ValidationError> {
        if self.password != self.password_confirmation {
            let mut error = ValidationError::new("password_mismatch");
            error.message = Some("Passwords do not match".into());
            return Err(error);
        }
        Ok(())
    }

    /// Validate password strength
    pub fn validate_password_strength(&self) -> Result<(), RegistrationError> {
        validate_password_strength(&self.password).map_err(Into::into)
    }
}

/// New user data
#[derive(Debug, Clone)]
pub struct NewUser {
    /// User ID
    pub id: Uuid,
    /// Email address
    pub email: String,
    /// Username
    pub username: String,
    /// Hashed password
    pub password_hash: String,
    /// BitTorrent passkey (32-character hex string)
    pub passkey: String,
    /// User role
    pub role: Role,
    /// User permissions
    pub permissions: PermissionSet,
    /// Email verified
    pub email_verified: bool,
    /// Account created timestamp
    pub created_at: DateTime<Utc>,
}

impl NewUser {
    /// Create a new user from registration request
    pub fn from_request(request: &RegisterRequest) -> Result<Self, RegistrationError> {
        // Hash password
        let password_hash = hash_password(&request.password)?;

        // Generate unique passkey for BitTorrent
        let passkey = generate_passkey();

        // Generate username if not provided
        let username = request
            .username
            .clone()
            .unwrap_or_else(|| generate_username_from_email(&request.email));

        Ok(Self {
            id: Uuid::new_v4(),
            email: request.email.clone(),
            username,
            password_hash,
            passkey,
            role: Role::NewUser, // Start with restricted permissions
            permissions: Role::NewUser.permissions(),
            email_verified: false,
            created_at: Utc::now(),
        })
    }
}

/// Email verification token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailVerificationToken {
    /// User ID
    pub user_id: Uuid,
    /// Email address
    pub email: String,
    /// Token expiration
    pub expires_at: DateTime<Utc>,
}

impl EmailVerificationToken {
    /// Create a new email verification token (valid for 24 hours)
    pub fn new(user_id: Uuid, email: String) -> Self {
        Self {
            user_id,
            email,
            expires_at: Utc::now() + chrono::Duration::hours(24),
        }
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Encode token as URL-safe string
    pub fn encode(&self) -> Result<String, RegistrationError> {
        let json = serde_json::to_string(self)
            .map_err(|e| RegistrationError::ValidationFailed(e.to_string()))?;
        Ok(base64::encode(json))
    }

    /// Decode token from URL-safe string
    pub fn decode(encoded: &str) -> Result<Self, RegistrationError> {
        let json = base64::decode(encoded)
            .map_err(|_| RegistrationError::ValidationFailed("Invalid token format".to_string()))?;
        serde_json::from_slice(&json)
            .map_err(|_| RegistrationError::ValidationFailed("Invalid token data".to_string()))
    }
}

/// User registration service
pub struct RegistrationService {
    db_pool: PgPool,
    require_email_verification: bool,
    require_invite: bool,
}

impl RegistrationService {
    /// Create a new registration service
    pub fn new(
        db_pool: PgPool,
        require_email_verification: bool,
        require_invite: bool,
    ) -> Self {
        Self {
            db_pool,
            require_email_verification,
            require_invite,
        }
    }

    /// Check if email is already registered
    pub async fn email_exists(&self, email: &str) -> Result<bool, RegistrationError> {
        let result = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)"
        )
        .bind(email)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| RegistrationError::DatabaseError(e.to_string()))?;

        Ok(result)
    }

    /// Check if username is already taken
    pub async fn username_exists(&self, username: &str) -> Result<bool, RegistrationError> {
        let result = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)"
        )
        .bind(username)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| RegistrationError::DatabaseError(e.to_string()))?;

        Ok(result)
    }

    /// Validate and consume an invitation code
    async fn validate_invite_code(&self, code: &str) -> Result<(), RegistrationError> {
        // Check if invite exists and is not used
        let result = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(
                SELECT 1 FROM invitations
                WHERE code = $1 AND used_at IS NULL AND expires_at > NOW()
            )"
        )
        .bind(code)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| RegistrationError::DatabaseError(e.to_string()))?;

        if !result {
            return Err(RegistrationError::InvalidInviteCode);
        }

        // Mark invite as used
        sqlx::query(
            "UPDATE invitations SET used_at = NOW() WHERE code = $1"
        )
        .bind(code)
        .execute(&self.db_pool)
        .await
        .map_err(|e| RegistrationError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Register a new user
    pub async fn register(&self, request: RegisterRequest) -> Result<NewUser, RegistrationError> {
        // Validate request
        request
            .validate()
            .map_err(|e| RegistrationError::ValidationFailed(e.to_string()))?;

        request.validate_password_match()
            .map_err(|e| RegistrationError::ValidationFailed(e.to_string()))?;

        request.validate_password_strength()?;

        // Check invite code if required
        if self.require_invite {
            let invite_code = request
                .invite_code
                .as_ref()
                .ok_or(RegistrationError::InvalidInviteCode)?;
            self.validate_invite_code(invite_code).await?;
        }

        // Check if email already exists
        if self.email_exists(&request.email).await? {
            return Err(RegistrationError::EmailExists);
        }

        // Check if username already exists (if provided)
        if let Some(ref username) = request.username {
            if self.username_exists(username).await? {
                return Err(RegistrationError::UsernameExists);
            }
        }

        // Create new user
        let new_user = NewUser::from_request(&request)?;

        // Insert into database
        sqlx::query(
            r#"
            INSERT INTO users (
                id, email, username, password_hash, passkey,
                role, email_verified, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#
        )
        .bind(new_user.id)
        .bind(&new_user.email)
        .bind(&new_user.username)
        .bind(&new_user.password_hash)
        .bind(&new_user.passkey)
        .bind(serde_json::to_value(&new_user.role).unwrap())
        .bind(new_user.email_verified)
        .bind(new_user.created_at)
        .execute(&self.db_pool)
        .await
        .map_err(|e| RegistrationError::DatabaseError(e.to_string()))?;

        Ok(new_user)
    }

    /// Generate email verification token
    pub fn generate_verification_token(&self, user_id: Uuid, email: String) -> EmailVerificationToken {
        EmailVerificationToken::new(user_id, email)
    }

    /// Verify email with token
    pub async fn verify_email(&self, token: &str) -> Result<Uuid, RegistrationError> {
        let verification = EmailVerificationToken::decode(token)?;

        if verification.is_expired() {
            return Err(RegistrationError::ValidationFailed("Token expired".to_string()));
        }

        // Update user email_verified status
        sqlx::query(
            "UPDATE users SET email_verified = true WHERE id = $1 AND email = $2"
        )
        .bind(verification.user_id)
        .bind(&verification.email)
        .execute(&self.db_pool)
        .await
        .map_err(|e| RegistrationError::DatabaseError(e.to_string()))?;

        Ok(verification.user_id)
    }
}

/// Generate a random 32-character hex passkey for BitTorrent
pub fn generate_passkey() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 16] = rng.gen();
    hex::encode(bytes)
}

/// Generate a username from email
fn generate_username_from_email(email: &str) -> String {
    // Take the part before @ and clean it up
    let local_part = email.split('@').next().unwrap_or("user");

    // Remove non-alphanumeric characters
    let cleaned: String = local_part
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect();

    // If empty, use default
    if cleaned.is_empty() {
        return format!("user_{}", Uuid::new_v4().to_string().split('-').next().unwrap());
    }

    // Truncate to max 30 chars and add random suffix
    let base = if cleaned.len() > 20 {
        &cleaned[..20]
    } else {
        &cleaned
    };

    format!("{}_{}", base, &Uuid::new_v4().to_string()[..6])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_passkey() {
        let passkey = generate_passkey();
        assert_eq!(passkey.len(), 32);

        // Should be valid hex
        assert!(hex::decode(&passkey).is_ok());
    }

    #[test]
    fn test_generate_username_from_email() {
        let username = generate_username_from_email("test@example.com");
        assert!(username.starts_with("test_"));
        assert!(username.len() > 5);
        assert!(username.len() <= 30);

        let username2 = generate_username_from_email("very.long.email.address@example.com");
        assert!(username2.len() <= 30);
    }

    #[test]
    fn test_register_request_validation() {
        let valid_request = RegisterRequest {
            email: "test@example.com".to_string(),
            username: Some("testuser".to_string()),
            password: "SecurePass123!".to_string(),
            password_confirmation: "SecurePass123!".to_string(),
            invite_code: None,
        };

        assert!(valid_request.validate().is_ok());
        assert!(valid_request.validate_password_match().is_ok());
        assert!(valid_request.validate_password_strength().is_ok());
    }

    #[test]
    fn test_register_request_invalid_email() {
        let request = RegisterRequest {
            email: "not-an-email".to_string(),
            username: Some("testuser".to_string()),
            password: "SecurePass123!".to_string(),
            password_confirmation: "SecurePass123!".to_string(),
            invite_code: None,
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_register_request_password_mismatch() {
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            username: Some("testuser".to_string()),
            password: "SecurePass123!".to_string(),
            password_confirmation: "DifferentPass123!".to_string(),
            invite_code: None,
        };

        assert!(request.validate_password_match().is_err());
    }

    #[test]
    fn test_register_request_weak_password() {
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            username: Some("testuser".to_string()),
            password: "weak".to_string(),
            password_confirmation: "weak".to_string(),
            invite_code: None,
        };

        assert!(request.validate_password_strength().is_err());
    }

    #[test]
    fn test_email_verification_token() {
        let user_id = Uuid::new_v4();
        let email = "test@example.com".to_string();
        let token = EmailVerificationToken::new(user_id, email.clone());

        assert_eq!(token.user_id, user_id);
        assert_eq!(token.email, email);
        assert!(!token.is_expired());

        // Test encoding/decoding
        let encoded = token.encode().unwrap();
        let decoded = EmailVerificationToken::decode(&encoded).unwrap();

        assert_eq!(decoded.user_id, user_id);
        assert_eq!(decoded.email, email);
    }

    #[test]
    fn test_new_user_from_request() {
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            username: Some("testuser".to_string()),
            password: "SecurePass123!".to_string(),
            password_confirmation: "SecurePass123!".to_string(),
            invite_code: None,
        };

        let user = NewUser::from_request(&request).unwrap();

        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.username, "testuser");
        assert_ne!(user.password_hash, request.password);
        assert_eq!(user.passkey.len(), 32);
        assert_eq!(user.role, Role::NewUser);
        assert!(!user.email_verified);
    }
}
