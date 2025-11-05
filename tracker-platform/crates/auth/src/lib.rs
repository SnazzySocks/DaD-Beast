//! Authentication and Authorization Service
//!
//! This crate provides a comprehensive authentication and authorization system
//! for the unified tracker platform, including:
//!
//! - **User Registration**: Email/password registration with validation, email verification
//! - **Login**: Secure login with password hashing (Argon2), JWT tokens, and session management
//! - **Two-Factor Authentication (2FA)**: TOTP-based 2FA with recovery codes
//! - **JWT Token Management**: Access and refresh tokens with automatic expiration
//! - **Session Management**: Redis-backed sessions with device tracking
//! - **Permission System**: Role-based access control (RBAC) with 20+ permissions
//! - **Middleware**: Axum extractors for authentication and authorization
//! - **Password Management**: Password reset, change, and strength validation
//!
//! # Architecture
//!
//! The auth crate follows a layered architecture:
//!
//! 1. **Core Layer** (`password`, `jwt`, `permissions`)
//!    - Password hashing and validation
//!    - JWT token generation and verification
//!    - Permission definitions and checks
//!
//! 2. **Service Layer** (`register`, `login`, `two_factor`, `session`)
//!    - Business logic for authentication flows
//!    - Database interactions
//!    - External service integration (email, Redis)
//!
//! 3. **Presentation Layer** (`middleware`)
//!    - Axum extractors and middleware
//!    - HTTP request/response handling
//!    - Route protection
//!
//! # Quick Start
//!
//! ## Setting up JWT and Session Management
//!
//! ```rust,no_run
//! use auth::{
//!     jwt::{JwtManager, TokenRevocationList},
//!     session::SessionManager,
//!     middleware::AuthState,
//! };
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize JWT manager with a secret key (store securely!)
//! let jwt_secret = std::env::var("JWT_SECRET")?;
//! let jwt_manager = JwtManager::new(&jwt_secret);
//!
//! // Initialize Redis for sessions and token revocation
//! let redis_url = std::env::var("REDIS_URL")?;
//! let redis_client = redis::Client::open(redis_url)?;
//!
//! let session_manager = SessionManager::new(redis_client.clone());
//! let revocation_list = TokenRevocationList::new(redis_client);
//!
//! // Create auth state for Axum
//! let auth_state = AuthState::new(jwt_manager, revocation_list);
//! # Ok(())
//! # }
//! ```
//!
//! ## User Registration
//!
//! ```rust,no_run
//! use auth::register::{RegisterRequest, RegistrationService};
//! use sqlx::PgPool;
//!
//! # async fn example(db_pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
//! let registration_service = RegistrationService::new(
//!     db_pool,
//!     true,  // require email verification
//!     false, // don't require invite
//! );
//!
//! let request = RegisterRequest {
//!     email: "user@example.com".to_string(),
//!     username: Some("username".to_string()),
//!     password: "SecurePassword123!".to_string(),
//!     password_confirmation: "SecurePassword123!".to_string(),
//!     invite_code: None,
//! };
//!
//! let new_user = registration_service.register(request).await?;
//! println!("Created user: {} ({})", new_user.username, new_user.id);
//! # Ok(())
//! # }
//! ```
//!
//! ## User Login
//!
//! ```rust,no_run
//! use auth::login::{LoginRequest, LoginService};
//! use auth::jwt::JwtManager;
//! use auth::session::SessionManager;
//! use auth::two_factor::TwoFactorManager;
//!
//! # async fn example(
//! #     db_pool: sqlx::PgPool,
//! #     jwt_manager: JwtManager,
//! #     session_manager: SessionManager,
//! # ) -> Result<(), Box<dyn std::error::Error>> {
//! let two_factor_manager = TwoFactorManager::new("TrackerPlatform");
//! let login_service = LoginService::new(
//!     db_pool,
//!     jwt_manager,
//!     session_manager,
//!     two_factor_manager,
//! );
//!
//! let request = LoginRequest {
//!     email: "user@example.com".to_string(),
//!     password: "SecurePassword123!".to_string(),
//!     totp_code: None,
//!     recovery_code: None,
//!     remember_me: false,
//! };
//!
//! let response = login_service.login(
//!     request,
//!     "127.0.0.1".to_string(),
//!     Some("Mozilla/5.0...".to_string()),
//! ).await?;
//!
//! println!("Access token: {}", response.access_token);
//! # Ok(())
//! # }
//! ```
//!
//! ## Protecting Routes with Middleware
//!
//! ```rust,no_run
//! use axum::{Router, routing::get, Json};
//! use auth::middleware::{AuthUser, AuthState};
//! use auth::permissions::Permission;
//!
//! async fn protected_route(user: AuthUser) -> Json<String> {
//!     Json(format!("Hello, user {}!", user.user_id))
//! }
//!
//! async fn admin_route(user: AuthUser) -> Result<Json<String>, auth::middleware::AuthError> {
//!     if !user.has_permission(Permission::SiteAdmin) {
//!         return Err(auth::middleware::AuthError::forbidden());
//!     }
//!     Ok(Json("Admin panel".to_string()))
//! }
//!
//! # fn example(auth_state: AuthState) {
//! let app = Router::new()
//!     .route("/protected", get(protected_route))
//!     .route("/admin", get(admin_route))
//!     .with_state(auth_state);
//! # }
//! ```
//!
//! # Security Considerations
//!
//! - **Password Storage**: Uses Argon2id for password hashing (industry standard)
//! - **JWT Secrets**: Store JWT_SECRET in environment variables, use strong random strings (32+ bytes)
//! - **Session Storage**: Redis provides fast, distributed session management
//! - **Token Revocation**: Supports both individual token and user-wide revocation
//! - **Rate Limiting**: Implement rate limiting at the API layer (not included in this crate)
//! - **HTTPS**: Always use HTTPS in production to protect tokens in transit
//! - **CORS**: Configure CORS appropriately to prevent unauthorized access
//!
//! # Database Schema
//!
//! This crate expects the following database tables (use migrations to create):
//!
//! - `users`: User accounts with credentials and metadata
//! - `invitations` (optional): Invitation codes for restricted registration
//!
//! See the `migrations/` directory for SQL schema definitions.

// Re-export commonly used types
pub use uuid::Uuid;

// Module declarations
pub mod jwt;
pub mod login;
pub mod middleware;
pub mod password;
pub mod permissions;
pub mod register;
pub mod session;
pub mod two_factor;

// Re-export key types for convenience
pub use jwt::{Claims, JwtManager, TokenPair, TokenRevocationList};
pub use login::{LoginError, LoginRequest, LoginResponse, LoginService};
pub use middleware::{AuthError, AuthState, AuthUser, OptionalAuthUser};
pub use password::{
    hash_password, validate_password_strength, verify_password, PasswordError, PasswordStrength,
};
pub use permissions::{Permission, PermissionSet, Role};
pub use register::{
    generate_passkey, EmailVerificationToken, NewUser, RegisterRequest, RegistrationError,
    RegistrationService,
};
pub use session::{parse_user_agent, Session, SessionError, SessionManager};
pub use two_factor::{
    TwoFactorConfig, TwoFactorError, TwoFactorManager, TwoFactorSetup,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Authentication module for complete auth functionality
///
/// This is a convenience re-export that groups all authentication-related
/// functionality in one place.
pub mod auth {
    pub use crate::jwt::*;
    pub use crate::login::*;
    pub use crate::middleware::*;
    pub use crate::password::*;
    pub use crate::permissions::*;
    pub use crate::register::*;
    pub use crate::session::*;
    pub use crate::two_factor::*;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_exports() {
        // Verify key types are exported
        let _: Result<(), PasswordError> = Ok(());
        let _: Result<(), LoginError> = Ok(());
        let _: Result<(), RegistrationError> = Ok(());
        let _: Result<(), SessionError> = Ok(());
        let _: Result<(), TwoFactorError> = Ok(());
    }
}
