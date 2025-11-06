//! Two-Factor Authentication (2FA) using TOTP
//!
//! This module implements Time-based One-Time Password (TOTP) authentication,
//! allowing users to secure their accounts with authenticator apps like
//! Google Authenticator, Authy, or 1Password.

use rand::Rng;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use totp_rs::{Algorithm, Secret, TOTP};
use uuid::Uuid;

/// Errors that can occur during 2FA operations
#[derive(Debug, Error)]
pub enum TwoFactorError {
    #[error("Failed to generate TOTP secret: {0}")]
    SecretGeneration(String),

    #[error("Invalid TOTP code")]
    InvalidCode,

    #[error("TOTP validation failed: {0}")]
    ValidationFailed(String),

    #[error("2FA is not enabled for this user")]
    NotEnabled,

    #[error("2FA is already enabled for this user")]
    AlreadyEnabled,

    #[error("Invalid recovery code")]
    InvalidRecoveryCode,

    #[error("QR code generation failed: {0}")]
    QrCodeGeneration(String),
}

/// Two-Factor Authentication configuration for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwoFactorConfig {
    /// Whether 2FA is enabled
    pub enabled: bool,
    /// TOTP secret key (base32 encoded)
    pub secret: String,
    /// Recovery codes (hashed)
    pub recovery_codes: Vec<String>,
    /// Backup email for recovery
    pub backup_email: Option<String>,
    /// When 2FA was enabled
    pub enabled_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl TwoFactorConfig {
    /// Create a new disabled 2FA config
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            secret: String::new(),
            recovery_codes: Vec::new(),
            backup_email: None,
            enabled_at: None,
        }
    }

    /// Check if 2FA is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Two-Factor Authentication manager
pub struct TwoFactorManager {
    issuer: String,
}

impl TwoFactorManager {
    /// Create a new 2FA manager
    pub fn new(issuer: impl Into<String>) -> Self {
        Self {
            issuer: issuer.into(),
        }
    }

    /// Generate a new TOTP secret
    pub fn generate_secret(&self) -> Result<String, TwoFactorError> {
        let secret = Secret::generate_secret();
        Ok(secret.to_encoded().to_string())
    }

    /// Create TOTP instance from secret
    fn create_totp(&self, secret: &str, account_name: &str) -> Result<TOTP, TwoFactorError> {
        TOTP::new(
            Algorithm::SHA1,
            6,  // 6 digits
            1,  // 1 step (30 seconds)
            30, // 30 second time step
            Secret::Encoded(secret.to_string())
                .to_bytes()
                .map_err(|e| TwoFactorError::SecretGeneration(e.to_string()))?,
            Some(self.issuer.clone()),
            account_name.to_string(),
        )
        .map_err(|e| TwoFactorError::SecretGeneration(e.to_string()))
    }

    /// Generate QR code URL for authenticator app setup
    ///
    /// # Arguments
    /// * `secret` - The TOTP secret (base32 encoded)
    /// * `account_name` - Usually the user's email or username
    ///
    /// # Returns
    /// A URL that can be used to generate a QR code or opened in an authenticator app
    pub fn generate_qr_code_url(
        &self,
        secret: &str,
        account_name: &str,
    ) -> Result<String, TwoFactorError> {
        let totp = self.create_totp(secret, account_name)?;
        Ok(totp.get_url())
    }

    /// Generate QR code as base64-encoded PNG image
    pub fn generate_qr_code_image(
        &self,
        secret: &str,
        account_name: &str,
    ) -> Result<String, TwoFactorError> {
        let totp = self.create_totp(secret, account_name)?;
        let qr_code = totp
            .get_qr_base64()
            .map_err(|e| TwoFactorError::QrCodeGeneration(e.to_string()))?;
        Ok(qr_code)
    }

    /// Verify a TOTP code
    ///
    /// # Arguments
    /// * `secret` - The TOTP secret
    /// * `code` - The 6-digit code from the authenticator app
    /// * `account_name` - The account name (for TOTP generation)
    ///
    /// # Returns
    /// Ok(()) if the code is valid, Err otherwise
    pub fn verify_code(
        &self,
        secret: &str,
        code: &str,
        account_name: &str,
    ) -> Result<(), TwoFactorError> {
        let totp = self.create_totp(secret, account_name)?;

        // Allow 1 time step before and after (to account for clock skew)
        if totp.check_current(code).map_err(|e| {
            TwoFactorError::ValidationFailed(format!("TOTP verification failed: {}", e))
        })? {
            Ok(())
        } else {
            Err(TwoFactorError::InvalidCode)
        }
    }

    /// Generate recovery codes
    ///
    /// Creates 10 random recovery codes that can be used as backup authentication.
    /// These should be stored hashed (like passwords) and shown to the user only once.
    pub fn generate_recovery_codes(&self, count: usize) -> Vec<String> {
        let mut rng = rand::thread_rng();
        let mut codes = Vec::with_capacity(count);

        for _ in 0..count {
            // Generate 8-character alphanumeric code
            let code: String = (0..8)
                .map(|_| {
                    let idx = rng.gen_range(0..36);
                    if idx < 10 {
                        (b'0' + idx) as char
                    } else {
                        (b'a' + idx - 10) as char
                    }
                })
                .collect();

            // Format as XXXX-XXXX for readability
            let formatted = format!("{}-{}", &code[0..4], &code[4..8]);
            codes.push(formatted);
        }

        codes
    }

    /// Hash recovery codes for storage
    ///
    /// Recovery codes should be stored hashed, similar to passwords.
    pub fn hash_recovery_codes(&self, codes: &[String]) -> Result<Vec<String>, TwoFactorError> {
        use crate::password::hash_password;

        codes
            .iter()
            .map(|code| {
                hash_password(code)
                    .map_err(|e| TwoFactorError::SecretGeneration(format!("Failed to hash recovery code: {}", e)))
            })
            .collect()
    }

    /// Verify a recovery code
    pub fn verify_recovery_code(
        &self,
        code: &str,
        hashed_codes: &[String],
    ) -> Result<usize, TwoFactorError> {
        use crate::password::verify_password;

        for (index, hashed_code) in hashed_codes.iter().enumerate() {
            if verify_password(code, hashed_code).is_ok() {
                return Ok(index);
            }
        }

        Err(TwoFactorError::InvalidRecoveryCode)
    }

    /// Enable 2FA for a user
    ///
    /// Returns the secret and recovery codes (unhashed) to show to the user.
    pub fn enable_2fa(
        &self,
        account_name: &str,
    ) -> Result<(String, Vec<String>, String), TwoFactorError> {
        let secret = self.generate_secret()?;
        let recovery_codes = self.generate_recovery_codes(10);
        let qr_url = self.generate_qr_code_url(&secret, account_name)?;

        Ok((secret, recovery_codes, qr_url))
    }
}

/// 2FA setup response
#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorSetup {
    /// The TOTP secret (to be stored securely)
    pub secret: String,
    /// QR code URL for authenticator apps
    pub qr_code_url: String,
    /// QR code as base64-encoded PNG
    pub qr_code_image: String,
    /// Recovery codes (show once to user)
    pub recovery_codes: Vec<String>,
    /// Manual entry key (alternative to QR code)
    pub manual_entry_key: String,
}

impl TwoFactorSetup {
    /// Create a new 2FA setup configuration
    pub fn new(
        secret: String,
        qr_code_url: String,
        qr_code_image: String,
        recovery_codes: Vec<String>,
    ) -> Self {
        let manual_entry_key = secret.clone();
        Self {
            secret,
            qr_code_url,
            qr_code_image,
            recovery_codes,
            manual_entry_key,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_secret() {
        let manager = TwoFactorManager::new("TestTracker");
        let secret = manager.generate_secret().unwrap();

        assert!(!secret.is_empty());
        assert!(secret.len() > 10);
    }

    #[test]
    fn test_generate_qr_code_url() {
        let manager = TwoFactorManager::new("TestTracker");
        let secret = manager.generate_secret().unwrap();
        let url = manager
            .generate_qr_code_url(&secret, "test@example.com")
            .unwrap();

        assert!(url.starts_with("otpauth://totp/"));
        assert!(url.contains("TestTracker"));
        assert!(url.contains("test@example.com"));
    }

    #[test]
    fn test_verify_code() {
        let manager = TwoFactorManager::new("TestTracker");
        let secret = manager.generate_secret().unwrap();

        // Generate current code
        let totp = manager
            .create_totp(&secret, "test@example.com")
            .unwrap();
        let code = totp.generate_current().unwrap();

        // Verify the code
        assert!(manager
            .verify_code(&secret, &code, "test@example.com")
            .is_ok());

        // Invalid code should fail
        assert!(manager
            .verify_code(&secret, "000000", "test@example.com")
            .is_err());
    }

    #[test]
    fn test_generate_recovery_codes() {
        let manager = TwoFactorManager::new("TestTracker");
        let codes = manager.generate_recovery_codes(10);

        assert_eq!(codes.len(), 10);

        for code in codes {
            // Should be in format XXXX-XXXX
            assert_eq!(code.len(), 9);
            assert_eq!(code.chars().nth(4).unwrap(), '-');
        }
    }

    #[test]
    fn test_hash_and_verify_recovery_codes() {
        let manager = TwoFactorManager::new("TestTracker");
        let codes = manager.generate_recovery_codes(5);
        let hashed_codes = manager.hash_recovery_codes(&codes).unwrap();

        // Verify first code
        let index = manager.verify_recovery_code(&codes[0], &hashed_codes).unwrap();
        assert_eq!(index, 0);

        // Verify last code
        let index = manager
            .verify_recovery_code(&codes[4], &hashed_codes)
            .unwrap();
        assert_eq!(index, 4);

        // Invalid code should fail
        assert!(manager
            .verify_recovery_code("invalid-code", &hashed_codes)
            .is_err());
    }

    #[test]
    fn test_enable_2fa() {
        let manager = TwoFactorManager::new("TestTracker");
        let (secret, recovery_codes, qr_url) =
            manager.enable_2fa("test@example.com").unwrap();

        assert!(!secret.is_empty());
        assert_eq!(recovery_codes.len(), 10);
        assert!(qr_url.starts_with("otpauth://totp/"));
    }

    #[test]
    fn test_two_factor_config() {
        let config = TwoFactorConfig::disabled();
        assert!(!config.is_enabled());
        assert!(config.secret.is_empty());
        assert!(config.recovery_codes.is_empty());
    }

    #[test]
    fn test_qr_code_image_generation() {
        let manager = TwoFactorManager::new("TestTracker");
        let secret = manager.generate_secret().unwrap();
        let image = manager
            .generate_qr_code_image(&secret, "test@example.com")
            .unwrap();

        // Should be base64 encoded data
        assert!(!image.is_empty());
    }
}
