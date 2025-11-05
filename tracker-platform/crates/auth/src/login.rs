//! User login and authentication
//!
//! This module handles the login flow including password verification,
//! 2FA validation, JWT token generation, and session management.

use crate::jwt::{JwtManager, TokenPair};
use crate::password::verify_password;
use crate::permissions::PermissionSet;
use crate::session::{Session, SessionManager};
use crate::two_factor::{TwoFactorConfig, TwoFactorManager};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;
use validator::Validate;

/// Errors that can occur during login
#[derive(Debug, Error)]
pub enum LoginError {
    #[error("Invalid email or password")]
    InvalidCredentials,

    #[error("Two-factor authentication required")]
    TwoFactorRequired,

    #[error("Invalid 2FA code")]
    InvalidTwoFactorCode,

    #[error("Account is disabled")]
    AccountDisabled,

    #[error("Email not verified")]
    EmailNotVerified,

    #[error("Account is locked due to too many failed login attempts")]
    AccountLocked,

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Token generation failed: {0}")]
    TokenGenerationFailed(String),

    #[error("Session creation failed: {0}")]
    SessionCreationFailed(String),
}

/// Login request with email and password
#[derive(Debug, Clone, Validate, Deserialize)]
pub struct LoginRequest {
    /// Email address
    #[validate(email(message = "Invalid email address"))]
    pub email: String,

    /// Password
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,

    /// 2FA code (if enabled)
    pub totp_code: Option<String>,

    /// Recovery code (alternative to TOTP)
    pub recovery_code: Option<String>,

    /// Remember this device (longer session)
    #[serde(default)]
    pub remember_me: bool,
}

/// Login response with tokens and user info
#[derive(Debug, Clone, Serialize)]
pub struct LoginResponse {
    /// Access token (15 minutes)
    pub access_token: String,

    /// Refresh token (7 days or 30 days if remember_me)
    pub refresh_token: String,

    /// Token type (always "Bearer")
    pub token_type: String,

    /// Access token expiration in seconds
    pub expires_in: i64,

    /// User information
    pub user: UserInfo,

    /// Whether 2FA is enabled for this user
    pub two_factor_enabled: bool,
}

/// User information returned in login response
#[derive(Debug, Clone, Serialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub role: String,
    pub permissions: Vec<String>,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
}

/// User record from database
#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub role: String,
    pub permissions: PermissionSet,
    pub email_verified: bool,
    pub is_enabled: bool,
    pub two_factor_enabled: bool,
    pub two_factor_secret: Option<String>,
    pub recovery_codes: Vec<String>,
    pub failed_login_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl User {
    /// Check if account is locked
    pub fn is_locked(&self) -> bool {
        if let Some(locked_until) = self.locked_until {
            return Utc::now() < locked_until;
        }
        false
    }

    /// Convert to UserInfo
    pub fn to_info(&self) -> UserInfo {
        UserInfo {
            id: self.id,
            email: self.email.clone(),
            username: self.username.clone(),
            role: self.role.clone(),
            permissions: self
                .permissions
                .all()
                .iter()
                .map(|p| format!("{:?}", p))
                .collect(),
            email_verified: self.email_verified,
            created_at: self.created_at,
        }
    }
}

/// Login service
pub struct LoginService {
    db_pool: PgPool,
    jwt_manager: JwtManager,
    session_manager: SessionManager,
    two_factor_manager: TwoFactorManager,
    max_failed_attempts: i32,
    lockout_duration_minutes: i64,
    require_email_verification: bool,
}

impl LoginService {
    /// Create a new login service
    pub fn new(
        db_pool: PgPool,
        jwt_manager: JwtManager,
        session_manager: SessionManager,
        two_factor_manager: TwoFactorManager,
    ) -> Self {
        Self {
            db_pool,
            jwt_manager,
            session_manager,
            two_factor_manager,
            max_failed_attempts: 5,
            lockout_duration_minutes: 15,
            require_email_verification: true,
        }
    }

    /// Configure login settings
    pub fn with_settings(
        mut self,
        max_failed_attempts: i32,
        lockout_duration_minutes: i64,
        require_email_verification: bool,
    ) -> Self {
        self.max_failed_attempts = max_failed_attempts;
        self.lockout_duration_minutes = lockout_duration_minutes;
        self.require_email_verification = require_email_verification;
        self
    }

    /// Get user by email
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, LoginError> {
        let record = sqlx::query!(
            r#"
            SELECT
                id, email, username, password_hash, role,
                email_verified, is_enabled,
                two_factor_enabled, two_factor_secret, recovery_codes,
                failed_login_attempts, locked_until, created_at
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| LoginError::DatabaseError(e.to_string()))?;

        Ok(record.map(|r| User {
            id: r.id,
            email: r.email,
            username: r.username,
            password_hash: r.password_hash,
            role: r.role.as_str().unwrap_or("user").to_string(),
            permissions: PermissionSet::new(), // Will be loaded separately if needed
            email_verified: r.email_verified,
            is_enabled: r.is_enabled,
            two_factor_enabled: r.two_factor_enabled,
            two_factor_secret: r.two_factor_secret,
            recovery_codes: r.recovery_codes.unwrap_or_default(),
            failed_login_attempts: r.failed_login_attempts,
            locked_until: r.locked_until,
            created_at: r.created_at,
        }))
    }

    /// Record failed login attempt
    async fn record_failed_attempt(&self, user_id: Uuid) -> Result<(), LoginError> {
        let failed_attempts = sqlx::query_scalar::<_, i32>(
            "UPDATE users
             SET failed_login_attempts = failed_login_attempts + 1,
                 last_failed_login = NOW()
             WHERE id = $1
             RETURNING failed_login_attempts"
        )
        .bind(user_id)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| LoginError::DatabaseError(e.to_string()))?;

        // Lock account if too many failed attempts
        if failed_attempts >= self.max_failed_attempts {
            let locked_until = Utc::now() + chrono::Duration::minutes(self.lockout_duration_minutes);
            sqlx::query(
                "UPDATE users SET locked_until = $1 WHERE id = $2"
            )
            .bind(locked_until)
            .bind(user_id)
            .execute(&self.db_pool)
            .await
            .map_err(|e| LoginError::DatabaseError(e.to_string()))?;
        }

        Ok(())
    }

    /// Reset failed login attempts
    async fn reset_failed_attempts(&self, user_id: Uuid) -> Result<(), LoginError> {
        sqlx::query(
            "UPDATE users
             SET failed_login_attempts = 0,
                 locked_until = NULL,
                 last_login = NOW()
             WHERE id = $1"
        )
        .bind(user_id)
        .execute(&self.db_pool)
        .await
        .map_err(|e| LoginError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Verify 2FA code
    fn verify_two_factor(&self, user: &User, code: &str) -> Result<(), LoginError> {
        if !user.two_factor_enabled {
            return Ok(());
        }

        let secret = user
            .two_factor_secret
            .as_ref()
            .ok_or(LoginError::InvalidTwoFactorCode)?;

        self.two_factor_manager
            .verify_code(secret, code, &user.email)
            .map_err(|_| LoginError::InvalidTwoFactorCode)
    }

    /// Verify recovery code and consume it
    async fn verify_recovery_code(&self, user: &User, code: &str) -> Result<(), LoginError> {
        let index = self
            .two_factor_manager
            .verify_recovery_code(code, &user.recovery_codes)
            .map_err(|_| LoginError::InvalidTwoFactorCode)?;

        // Remove used recovery code
        sqlx::query(
            "UPDATE users
             SET recovery_codes = array_remove(recovery_codes, $1)
             WHERE id = $2"
        )
        .bind(&user.recovery_codes[index])
        .bind(user.id)
        .execute(&self.db_pool)
        .await
        .map_err(|e| LoginError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Perform login
    pub async fn login(
        &self,
        request: LoginRequest,
        ip_address: String,
        user_agent: Option<String>,
    ) -> Result<LoginResponse, LoginError> {
        // Validate request
        request
            .validate()
            .map_err(|_| LoginError::InvalidCredentials)?;

        // Get user
        let user = self
            .get_user_by_email(&request.email)
            .await?
            .ok_or(LoginError::InvalidCredentials)?;

        // Check if account is locked
        if user.is_locked() {
            return Err(LoginError::AccountLocked);
        }

        // Check if account is enabled
        if !user.is_enabled {
            return Err(LoginError::AccountDisabled);
        }

        // Check email verification
        if self.require_email_verification && !user.email_verified {
            return Err(LoginError::EmailNotVerified);
        }

        // Verify password
        if verify_password(&request.password, &user.password_hash).is_err() {
            self.record_failed_attempt(user.id).await?;
            return Err(LoginError::InvalidCredentials);
        }

        // Check 2FA if enabled
        if user.two_factor_enabled {
            if let Some(totp_code) = &request.totp_code {
                self.verify_two_factor(&user, totp_code)?;
            } else if let Some(recovery_code) = &request.recovery_code {
                self.verify_recovery_code(&user, recovery_code).await?;
            } else {
                return Err(LoginError::TwoFactorRequired);
            }
        }

        // Reset failed attempts
        self.reset_failed_attempts(user.id).await?;

        // Generate tokens
        let token_pair = self
            .jwt_manager
            .generate_token_pair(user.id, user.permissions.all())
            .map_err(|e| LoginError::TokenGenerationFailed(e.to_string()))?;

        // Create session
        let ttl = if request.remember_me {
            30 * 24 * 60 * 60 // 30 days
        } else {
            7 * 24 * 60 * 60 // 7 days
        };

        // Decode token to get token_id
        let claims = self
            .jwt_manager
            .validate_token(&token_pair.access_token)
            .map_err(|e| LoginError::TokenGenerationFailed(e.to_string()))?;

        let session = Session::new(user.id, claims.token_id(), ip_address, user_agent, ttl);

        self.session_manager
            .create_session(&session)
            .await
            .map_err(|e| LoginError::SessionCreationFailed(e.to_string()))?;

        // Build response
        Ok(LoginResponse {
            access_token: token_pair.access_token,
            refresh_token: token_pair.refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: 15 * 60, // 15 minutes
            user: user.to_info(),
            two_factor_enabled: user.two_factor_enabled,
        })
    }

    /// Refresh access token
    pub async fn refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<TokenPair, LoginError> {
        // Validate refresh token
        let claims = self
            .jwt_manager
            .validate_token(refresh_token)
            .map_err(|_| LoginError::InvalidCredentials)?;

        // Get user to fetch current permissions
        let user = sqlx::query!(
            "SELECT id, role FROM users WHERE id = $1 AND is_enabled = true",
            claims.user_id()
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| LoginError::DatabaseError(e.to_string()))?
        .ok_or(LoginError::InvalidCredentials)?;

        // Get permissions (simplified - in production, load from database)
        let permissions = PermissionSet::new();

        // Generate new token pair
        let new_tokens = self
            .jwt_manager
            .generate_token_pair(user.id, permissions.all())
            .map_err(|e| LoginError::TokenGenerationFailed(e.to_string()))?;

        Ok(new_tokens)
    }

    /// Logout (revoke session)
    pub async fn logout(&self, session_id: Uuid) -> Result<(), LoginError> {
        self.session_manager
            .delete_session(session_id)
            .await
            .map_err(|e| LoginError::SessionCreationFailed(e.to_string()))?;

        Ok(())
    }

    /// Logout from all devices (revoke all sessions)
    pub async fn logout_all(&self, user_id: Uuid) -> Result<usize, LoginError> {
        let count = self
            .session_manager
            .delete_all_user_sessions(user_id)
            .await
            .map_err(|e| LoginError::SessionCreationFailed(e.to_string()))?;

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_request_validation() {
        let valid_request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            totp_code: None,
            recovery_code: None,
            remember_me: false,
        };

        assert!(valid_request.validate().is_ok());
    }

    #[test]
    fn test_login_request_invalid_email() {
        let request = LoginRequest {
            email: "not-an-email".to_string(),
            password: "password123".to_string(),
            totp_code: None,
            recovery_code: None,
            remember_me: false,
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_user_is_locked() {
        let mut user = User {
            id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            username: "testuser".to_string(),
            password_hash: "hash".to_string(),
            role: "user".to_string(),
            permissions: PermissionSet::new(),
            email_verified: true,
            is_enabled: true,
            two_factor_enabled: false,
            two_factor_secret: None,
            recovery_codes: Vec::new(),
            failed_login_attempts: 0,
            locked_until: None,
            created_at: Utc::now(),
        };

        assert!(!user.is_locked());

        user.locked_until = Some(Utc::now() + chrono::Duration::hours(1));
        assert!(user.is_locked());

        user.locked_until = Some(Utc::now() - chrono::Duration::hours(1));
        assert!(!user.is_locked());
    }

    #[test]
    fn test_user_to_info() {
        let user = User {
            id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            username: "testuser".to_string(),
            password_hash: "hash".to_string(),
            role: "user".to_string(),
            permissions: PermissionSet::new(),
            email_verified: true,
            is_enabled: true,
            two_factor_enabled: false,
            two_factor_secret: None,
            recovery_codes: Vec::new(),
            failed_login_attempts: 0,
            locked_until: None,
            created_at: Utc::now(),
        };

        let info = user.to_info();
        assert_eq!(info.email, "test@example.com");
        assert_eq!(info.username, "testuser");
        assert!(info.email_verified);
    }
}
