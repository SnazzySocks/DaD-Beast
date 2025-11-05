//! Password management and validation
//!
//! This module provides secure password hashing, verification, and strength validation
//! using Argon2id, the recommended algorithm for password hashing.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use thiserror::Error;
use validator::ValidationError;

/// Errors that can occur during password operations
#[derive(Debug, Error)]
pub enum PasswordError {
    #[error("Password hashing failed: {0}")]
    HashingFailed(String),

    #[error("Password verification failed: {0}")]
    VerificationFailed(String),

    #[error("Invalid password hash format")]
    InvalidHashFormat,

    #[error("Password is too weak: {0}")]
    WeakPassword(String),
}

/// Password strength requirements
pub struct PasswordStrength {
    /// Minimum password length
    pub min_length: usize,
    /// Require at least one uppercase letter
    pub require_uppercase: bool,
    /// Require at least one lowercase letter
    pub require_lowercase: bool,
    /// Require at least one digit
    pub require_digit: bool,
    /// Require at least one special character
    pub require_special: bool,
}

impl Default for PasswordStrength {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
            require_special: true,
        }
    }
}

impl PasswordStrength {
    /// Create a basic password policy (8+ chars, no special requirements)
    pub fn basic() -> Self {
        Self {
            min_length: 8,
            require_uppercase: false,
            require_lowercase: false,
            require_digit: false,
            require_special: false,
        }
    }

    /// Create a moderate password policy (8+ chars, mixed case + numbers)
    pub fn moderate() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
            require_special: false,
        }
    }

    /// Create a strong password policy (default)
    pub fn strong() -> Self {
        Self::default()
    }

    /// Validate a password against this policy
    pub fn validate(&self, password: &str) -> Result<(), PasswordError> {
        let mut errors = Vec::new();

        // Check minimum length
        if password.len() < self.min_length {
            errors.push(format!("at least {} characters", self.min_length));
        }

        // Check for uppercase
        if self.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            errors.push("at least one uppercase letter".to_string());
        }

        // Check for lowercase
        if self.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            errors.push("at least one lowercase letter".to_string());
        }

        // Check for digits
        if self.require_digit && !password.chars().any(|c| c.is_ascii_digit()) {
            errors.push("at least one number".to_string());
        }

        // Check for special characters
        if self.require_special && !password.chars().any(|c| !c.is_alphanumeric()) {
            errors.push("at least one special character".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(PasswordError::WeakPassword(format!(
                "Password must contain: {}",
                errors.join(", ")
            )))
        }
    }
}

/// Hash a password using Argon2id
///
/// This uses secure defaults recommended by OWASP:
/// - Argon2id variant (resistant to both side-channel and GPU attacks)
/// - Automatically generated salt
/// - Standard parameters for security
///
/// # Example
/// ```no_run
/// use auth::password::hash_password;
///
/// let password = "SecurePassword123!";
/// let hash = hash_password(password).expect("Failed to hash password");
/// println!("Password hash: {}", hash);
/// ```
pub fn hash_password(password: &str) -> Result<String, PasswordError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| PasswordError::HashingFailed(e.to_string()))?;

    Ok(password_hash.to_string())
}

/// Verify a password against a hash
///
/// # Example
/// ```no_run
/// use auth::password::{hash_password, verify_password};
///
/// let password = "SecurePassword123!";
/// let hash = hash_password(password).unwrap();
///
/// assert!(verify_password(password, &hash).is_ok());
/// assert!(verify_password("WrongPassword", &hash).is_err());
/// ```
pub fn verify_password(password: &str, password_hash: &str) -> Result<(), PasswordError> {
    let parsed_hash = PasswordHash::new(password_hash)
        .map_err(|_| PasswordError::InvalidHashFormat)?;

    let argon2 = Argon2::default();

    argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|e| PasswordError::VerificationFailed(e.to_string()))
}

/// Validate password strength with default policy
///
/// # Example
/// ```no_run
/// use auth::password::validate_password_strength;
///
/// assert!(validate_password_strength("SecurePassword123!").is_ok());
/// assert!(validate_password_strength("weak").is_err());
/// ```
pub fn validate_password_strength(password: &str) -> Result<(), PasswordError> {
    PasswordStrength::default().validate(password)
}

/// Custom validator for use with the `validator` crate
///
/// # Example
/// ```no_run
/// use validator::Validate;
/// use serde::Deserialize;
///
/// #[derive(Deserialize, Validate)]
/// struct RegisterRequest {
///     #[validate(email)]
///     email: String,
///     #[validate(custom = "auth::password::validator_password_strength")]
///     password: String,
/// }
/// ```
pub fn validator_password_strength(password: &str) -> Result<(), ValidationError> {
    validate_password_strength(password).map_err(|e| {
        let mut error = ValidationError::new("password_strength");
        error.message = Some(std::borrow::Cow::Owned(e.to_string()));
        error
    })
}

/// Generate a secure random password
///
/// Useful for generating temporary passwords or API keys.
pub fn generate_random_password(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789\
                            !@#$%^&*";
    let mut rng = rand::thread_rng();

    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Password reset token data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PasswordResetToken {
    /// User ID this token is for
    pub user_id: uuid::Uuid,
    /// Email address
    pub email: String,
    /// Token expiration time
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

impl PasswordResetToken {
    /// Create a new password reset token (valid for 1 hour)
    pub fn new(user_id: uuid::Uuid, email: String) -> Self {
        Self {
            user_id,
            email,
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
        }
    }

    /// Create a new password reset token with custom expiry
    pub fn new_with_expiry(
        user_id: uuid::Uuid,
        email: String,
        duration: chrono::Duration,
    ) -> Self {
        Self {
            user_id,
            email,
            expires_at: chrono::Utc::now() + duration,
        }
    }

    /// Check if the token has expired
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() > self.expires_at
    }

    /// Encode the token as a URL-safe string
    pub fn encode(&self) -> Result<String, PasswordError> {
        let json = serde_json::to_string(self)
            .map_err(|e| PasswordError::HashingFailed(format!("Failed to encode token: {}", e)))?;
        Ok(base64::encode(json))
    }

    /// Decode a token from a URL-safe string
    pub fn decode(encoded: &str) -> Result<Self, PasswordError> {
        let json = base64::decode(encoded)
            .map_err(|e| PasswordError::InvalidHashFormat)?;
        let token: PasswordResetToken = serde_json::from_slice(&json)
            .map_err(|e| PasswordError::InvalidHashFormat)?;
        Ok(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "TestPassword123!";
        let hash = hash_password(password).expect("Failed to hash password");

        assert!(!hash.is_empty());
        assert!(hash.starts_with("$argon2"));
    }

    #[test]
    fn test_password_verification_success() {
        let password = "TestPassword123!";
        let hash = hash_password(password).unwrap();

        assert!(verify_password(password, &hash).is_ok());
    }

    #[test]
    fn test_password_verification_failure() {
        let password = "TestPassword123!";
        let hash = hash_password(password).unwrap();

        assert!(verify_password("WrongPassword", &hash).is_err());
    }

    #[test]
    fn test_password_strength_validation() {
        let policy = PasswordStrength::default();

        // Valid passwords
        assert!(policy.validate("SecurePass123!").is_ok());
        assert!(policy.validate("MyP@ssw0rd").is_ok());

        // Too short
        assert!(policy.validate("Short1!").is_err());

        // Missing uppercase
        assert!(policy.validate("password123!").is_err());

        // Missing lowercase
        assert!(policy.validate("PASSWORD123!").is_err());

        // Missing digit
        assert!(policy.validate("Password!").is_err());

        // Missing special character
        assert!(policy.validate("Password123").is_err());
    }

    #[test]
    fn test_password_strength_policies() {
        let basic = PasswordStrength::basic();
        assert!(basic.validate("password").is_ok());

        let moderate = PasswordStrength::moderate();
        assert!(moderate.validate("Password123").is_ok());
        assert!(moderate.validate("password").is_err());

        let strong = PasswordStrength::strong();
        assert!(strong.validate("Password123!").is_ok());
        assert!(strong.validate("Password123").is_err());
    }

    #[test]
    fn test_generate_random_password() {
        let password = generate_random_password(16);
        assert_eq!(password.len(), 16);

        // Should pass strong validation with high probability
        let policy = PasswordStrength::default();
        // Generate multiple times to ensure randomness
        let mut valid_count = 0;
        for _ in 0..10 {
            let pwd = generate_random_password(16);
            if policy.validate(&pwd).is_ok() {
                valid_count += 1;
            }
        }
        assert!(valid_count > 0);
    }

    #[test]
    fn test_password_reset_token() {
        let user_id = uuid::Uuid::new_v4();
        let email = "test@example.com".to_string();
        let token = PasswordResetToken::new(user_id, email.clone());

        assert_eq!(token.user_id, user_id);
        assert_eq!(token.email, email);
        assert!(!token.is_expired());

        // Test encoding/decoding
        let encoded = token.encode().unwrap();
        let decoded = PasswordResetToken::decode(&encoded).unwrap();

        assert_eq!(decoded.user_id, user_id);
        assert_eq!(decoded.email, email);
    }

    #[test]
    fn test_password_reset_token_expiry() {
        let user_id = uuid::Uuid::new_v4();
        let email = "test@example.com".to_string();
        let token = PasswordResetToken::new_with_expiry(
            user_id,
            email,
            chrono::Duration::seconds(-1),
        );

        assert!(token.is_expired());
    }
}
