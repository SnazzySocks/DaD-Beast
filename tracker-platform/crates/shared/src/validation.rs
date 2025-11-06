//! Common validation rules for the unified tracker platform.
//!
//! This module provides validation functions for user input, including
//! email, password strength, and BitTorrent-specific formats.

use regex::Regex;
use std::sync::LazyLock;

use crate::error::{AppResult, ValidationError};
use crate::types::InfoHash;

/// Email validation regex
static EMAIL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
});

/// Username validation regex (alphanumeric, underscore, hyphen, 3-20 chars)
static USERNAME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-zA-Z0-9_-]{3,20}$").unwrap()
});

/// Validate email address format
///
/// # Arguments
///
/// * `email` - Email address to validate
///
/// # Errors
///
/// Returns an error if the email format is invalid
///
/// # Example
///
/// ```
/// use shared::validation::validate_email;
///
/// assert!(validate_email("user@example.com").is_ok());
/// assert!(validate_email("invalid-email").is_err());
/// ```
pub fn validate_email(email: &str) -> AppResult<()> {
    if email.is_empty() {
        return Err(ValidationError::MissingField("email".to_string()).into());
    }

    if !EMAIL_REGEX.is_match(email) {
        return Err(ValidationError::InvalidEmail.into());
    }

    if email.len() > 255 {
        return Err(
            ValidationError::InvalidField {
                field: "email".to_string(),
                message: "Email too long (max 255 characters)".to_string(),
            }
            .into(),
        );
    }

    Ok(())
}

/// Validate username format
///
/// # Arguments
///
/// * `username` - Username to validate
///
/// # Errors
///
/// Returns an error if the username format is invalid
///
/// # Example
///
/// ```
/// use shared::validation::validate_username;
///
/// assert!(validate_username("john_doe").is_ok());
/// assert!(validate_username("ab").is_err()); // Too short
/// ```
pub fn validate_username(username: &str) -> AppResult<()> {
    if username.is_empty() {
        return Err(ValidationError::MissingField("username".to_string()).into());
    }

    if !USERNAME_REGEX.is_match(username) {
        return Err(
            ValidationError::InvalidField {
                field: "username".to_string(),
                message: "Username must be 3-20 characters and contain only letters, numbers, underscore, and hyphen".to_string(),
            }
            .into(),
        );
    }

    Ok(())
}

/// Password strength requirements
#[derive(Debug, Clone)]
pub struct PasswordRequirements {
    /// Minimum length
    pub min_length: usize,
    /// Require uppercase letter
    pub require_uppercase: bool,
    /// Require lowercase letter
    pub require_lowercase: bool,
    /// Require digit
    pub require_digit: bool,
    /// Require special character
    pub require_special: bool,
}

impl Default for PasswordRequirements {
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

impl PasswordRequirements {
    /// Create lenient password requirements (for testing or legacy systems)
    pub fn lenient() -> Self {
        Self {
            min_length: 6,
            require_uppercase: false,
            require_lowercase: true,
            require_digit: false,
            require_special: false,
        }
    }

    /// Create strict password requirements
    pub fn strict() -> Self {
        Self {
            min_length: 12,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
            require_special: true,
        }
    }
}

/// Validate password strength
///
/// # Arguments
///
/// * `password` - Password to validate
/// * `requirements` - Password requirements (optional, uses default if None)
///
/// # Errors
///
/// Returns an error if the password doesn't meet requirements
///
/// # Example
///
/// ```
/// use shared::validation::{validate_password, PasswordRequirements};
///
/// assert!(validate_password("SecurePass123!", None).is_ok());
/// assert!(validate_password("weak", None).is_err());
/// ```
pub fn validate_password(
    password: &str,
    requirements: Option<PasswordRequirements>,
) -> AppResult<()> {
    if password.is_empty() {
        return Err(ValidationError::MissingField("password".to_string()).into());
    }

    let requirements = requirements.unwrap_or_default();
    let mut errors = Vec::new();

    if password.len() < requirements.min_length {
        errors.push(format!(
            "at least {} characters",
            requirements.min_length
        ));
    }

    if requirements.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
        errors.push("at least one uppercase letter".to_string());
    }

    if requirements.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
        errors.push("at least one lowercase letter".to_string());
    }

    if requirements.require_digit && !password.chars().any(|c| c.is_ascii_digit()) {
        errors.push("at least one digit".to_string());
    }

    if requirements.require_special
        && !password
            .chars()
            .any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c))
    {
        errors.push("at least one special character".to_string());
    }

    if !errors.is_empty() {
        return Err(
            ValidationError::WeakPassword(format!("Password must contain: {}", errors.join(", ")))
                .into(),
        );
    }

    Ok(())
}

/// Validate torrent info hash
///
/// # Arguments
///
/// * `info_hash` - Info hash string (40-character hex)
///
/// # Errors
///
/// Returns an error if the info hash format is invalid
///
/// # Example
///
/// ```
/// use shared::validation::validate_info_hash;
///
/// assert!(validate_info_hash("0123456789abcdef0123456789abcdef01234567").is_ok());
/// assert!(validate_info_hash("invalid").is_err());
/// ```
pub fn validate_info_hash(info_hash: &str) -> AppResult<()> {
    InfoHash::from_hex(info_hash)
        .map(|_| ())
        .map_err(|_| ValidationError::InvalidInfoHash.into())
}

/// Validate port number
///
/// # Arguments
///
/// * `port` - Port number to validate
///
/// # Errors
///
/// Returns an error if the port is invalid
pub fn validate_port(port: u16) -> AppResult<()> {
    if port == 0 {
        return Err(
            ValidationError::InvalidField {
                field: "port".to_string(),
                message: "Port cannot be 0".to_string(),
            }
            .into(),
        );
    }

    if port < 1024 {
        return Err(
            ValidationError::InvalidField {
                field: "port".to_string(),
                message: "Port must be >= 1024 (privileged ports not allowed)".to_string(),
            }
            .into(),
        );
    }

    Ok(())
}

/// Validate torrent name
///
/// # Arguments
///
/// * `name` - Torrent name to validate
///
/// # Errors
///
/// Returns an error if the name is invalid
pub fn validate_torrent_name(name: &str) -> AppResult<()> {
    if name.is_empty() {
        return Err(ValidationError::MissingField("name".to_string()).into());
    }

    if name.len() > 255 {
        return Err(
            ValidationError::InvalidField {
                field: "name".to_string(),
                message: "Name too long (max 255 characters)".to_string(),
            }
            .into(),
        );
    }

    // Check for invalid characters
    let invalid_chars = ['/', '\\', '\0', '<', '>', ':', '"', '|', '?', '*'];
    if name.chars().any(|c| invalid_chars.contains(&c)) {
        return Err(
            ValidationError::InvalidField {
                field: "name".to_string(),
                message: "Name contains invalid characters".to_string(),
            }
            .into(),
        );
    }

    Ok(())
}

/// Validate file size
///
/// # Arguments
///
/// * `size` - File size in bytes
/// * `max_size` - Maximum allowed size in bytes
///
/// # Errors
///
/// Returns an error if the size is invalid
pub fn validate_file_size(size: i64, max_size: i64) -> AppResult<()> {
    if size <= 0 {
        return Err(
            ValidationError::InvalidField {
                field: "size".to_string(),
                message: "Size must be positive".to_string(),
            }
            .into(),
        );
    }

    if size > max_size {
        return Err(
            ValidationError::InvalidField {
                field: "size".to_string(),
                message: format!("Size exceeds maximum of {} bytes", max_size),
            }
            .into(),
        );
    }

    Ok(())
}

/// Validate ratio (uploaded/downloaded)
///
/// # Arguments
///
/// * `uploaded` - Bytes uploaded
/// * `downloaded` - Bytes downloaded
///
/// # Errors
///
/// Returns an error if the values are invalid
pub fn validate_ratio(uploaded: i64, downloaded: i64) -> AppResult<()> {
    if uploaded < 0 {
        return Err(
            ValidationError::InvalidField {
                field: "uploaded".to_string(),
                message: "Uploaded bytes cannot be negative".to_string(),
            }
            .into(),
        );
    }

    if downloaded < 0 {
        return Err(
            ValidationError::InvalidField {
                field: "downloaded".to_string(),
                message: "Downloaded bytes cannot be negative".to_string(),
            }
            .into(),
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("test.user+tag@example.co.uk").is_ok());
        assert!(validate_email("invalid-email").is_err());
        assert!(validate_email("@example.com").is_err());
        assert!(validate_email("user@").is_err());
        assert!(validate_email("").is_err());
    }

    #[test]
    fn test_username_validation() {
        assert!(validate_username("john_doe").is_ok());
        assert!(validate_username("user123").is_ok());
        assert!(validate_username("test-user").is_ok());
        assert!(validate_username("ab").is_err()); // Too short
        assert!(validate_username("a".repeat(21).as_str()).is_err()); // Too long
        assert!(validate_username("user@name").is_err()); // Invalid character
        assert!(validate_username("").is_err());
    }

    #[test]
    fn test_password_validation() {
        assert!(validate_password("SecurePass123!", None).is_ok());
        assert!(validate_password("weak", None).is_err());
        assert!(validate_password("NoDigits!", None).is_err());
        assert!(validate_password("nouppercse123!", None).is_err());
        assert!(validate_password("NOLOWERCASE123!", None).is_err());
        assert!(validate_password("NoSpecial123", None).is_err());

        // Test lenient requirements
        let lenient = Some(PasswordRequirements::lenient());
        assert!(validate_password("simple", lenient).is_ok());
    }

    #[test]
    fn test_info_hash_validation() {
        assert!(validate_info_hash("0123456789abcdef0123456789abcdef01234567").is_ok());
        assert!(validate_info_hash("ABCDEF0123456789ABCDEF0123456789ABCDEF01").is_ok());
        assert!(validate_info_hash("invalid").is_err());
        assert!(validate_info_hash("").is_err());
        assert!(validate_info_hash("0123456789abcdef").is_err()); // Too short
    }

    #[test]
    fn test_port_validation() {
        assert!(validate_port(8080).is_ok());
        assert!(validate_port(65535).is_ok());
        assert!(validate_port(0).is_err());
        assert!(validate_port(80).is_err()); // Privileged port
    }

    #[test]
    fn test_torrent_name_validation() {
        assert!(validate_torrent_name("My Torrent").is_ok());
        assert!(validate_torrent_name("movie-2024").is_ok());
        assert!(validate_torrent_name("").is_err());
        assert!(validate_torrent_name("a".repeat(256).as_str()).is_err());
        assert!(validate_torrent_name("invalid/name").is_err());
        assert!(validate_torrent_name("invalid\\name").is_err());
    }

    #[test]
    fn test_file_size_validation() {
        assert!(validate_file_size(1024, 1000000).is_ok());
        assert!(validate_file_size(0, 1000000).is_err());
        assert!(validate_file_size(-1, 1000000).is_err());
        assert!(validate_file_size(2000000, 1000000).is_err());
    }

    #[test]
    fn test_ratio_validation() {
        assert!(validate_ratio(1000, 500).is_ok());
        assert!(validate_ratio(0, 0).is_ok());
        assert!(validate_ratio(-1, 500).is_err());
        assert!(validate_ratio(1000, -500).is_err());
    }
}
