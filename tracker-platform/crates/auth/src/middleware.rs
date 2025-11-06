//! Axum middleware for authentication and authorization
//!
//! This module provides middleware to protect routes, extract authenticated
//! users from JWT tokens, and enforce permission-based access control.

use crate::jwt::{Claims, JwtManager, TokenRevocationList};
use crate::permissions::Permission;
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json, RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

/// Authentication state shared across the application
#[derive(Clone)]
pub struct AuthState {
    pub jwt_manager: Arc<JwtManager>,
    pub revocation_list: Arc<TokenRevocationList>,
}

impl AuthState {
    pub fn new(jwt_manager: JwtManager, revocation_list: TokenRevocationList) -> Self {
        Self {
            jwt_manager: Arc::new(jwt_manager),
            revocation_list: Arc::new(revocation_list),
        }
    }
}

/// Authenticated user extracted from JWT token
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub permissions: Vec<Permission>,
    pub token_id: Uuid,
}

impl AuthUser {
    /// Check if user has a specific permission
    pub fn has_permission(&self, permission: Permission) -> bool {
        self.permissions.contains(&permission)
    }

    /// Check if user has any of the specified permissions
    pub fn has_any_permission(&self, permissions: &[Permission]) -> bool {
        permissions.iter().any(|p| self.permissions.contains(p))
    }

    /// Check if user has all of the specified permissions
    pub fn has_all_permissions(&self, permissions: &[Permission]) -> bool {
        permissions.iter().all(|p| self.permissions.contains(p))
    }
}

/// Error response for authentication failures
#[derive(Debug, Serialize)]
pub struct AuthError {
    pub error: String,
    pub message: String,
}

impl AuthError {
    pub fn new(error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
        }
    }

    pub fn unauthorized() -> Self {
        Self::new("unauthorized", "Authentication required")
    }

    pub fn invalid_token() -> Self {
        Self::new("invalid_token", "Invalid or expired token")
    }

    pub fn forbidden() -> Self {
        Self::new("forbidden", "Insufficient permissions")
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let status = match self.error.as_str() {
            "unauthorized" | "invalid_token" => StatusCode::UNAUTHORIZED,
            "forbidden" => StatusCode::FORBIDDEN,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, Json(self)).into_response()
    }
}

/// Extract authenticated user from request
///
/// This can be used as a parameter in Axum handlers to automatically
/// authenticate requests and extract user information.
///
/// # Example
/// ```no_run
/// use axum::{Json, response::IntoResponse};
/// use auth::middleware::AuthUser;
///
/// async fn protected_route(user: AuthUser) -> impl IntoResponse {
///     Json(format!("Hello, user {}!", user.user_id))
/// }
/// ```
#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    AuthState: FromRef<S>,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::unauthorized())?;

        let auth_state = AuthState::from_ref(state);

        // Validate the JWT token
        let claims = auth_state
            .jwt_manager
            .validate_token(bearer.token())
            .map_err(|_| AuthError::invalid_token())?;

        // Check if token is revoked
        let is_revoked = auth_state
            .revocation_list
            .is_revoked(claims.token_id())
            .await
            .unwrap_or(false);

        if is_revoked {
            return Err(AuthError::invalid_token());
        }

        // Check if all user tokens are revoked
        let user_revoked = auth_state
            .revocation_list
            .is_user_revoked(claims.user_id(), claims.iat)
            .await
            .unwrap_or(false);

        if user_revoked {
            return Err(AuthError::invalid_token());
        }

        Ok(AuthUser {
            user_id: claims.user_id(),
            permissions: claims.permissions,
            token_id: claims.token_id(),
        })
    }
}

/// Require specific permission(s) to access a route
///
/// # Example
/// ```no_run
/// use axum::{Router, routing::get};
/// use auth::middleware::RequirePermission;
/// use auth::permissions::Permission;
///
/// async fn admin_route() -> &'static str {
///     "Admin only"
/// }
///
/// let app = Router::new()
///     .route("/admin", get(admin_route))
///     .route_layer(RequirePermission::new(vec![Permission::SiteAdmin]));
/// ```
pub struct RequirePermission {
    required_permissions: Vec<Permission>,
    require_all: bool,
}

impl RequirePermission {
    /// Create new permission requirement (requires ANY of the permissions)
    pub fn new(permissions: Vec<Permission>) -> Self {
        Self {
            required_permissions: permissions,
            require_all: false,
        }
    }

    /// Create new permission requirement (requires ALL permissions)
    pub fn all(permissions: Vec<Permission>) -> Self {
        Self {
            required_permissions: permissions,
            require_all: true,
        }
    }
}

/// User with permission requirements checked
#[derive(Debug, Clone)]
pub struct PermissionUser {
    pub user: AuthUser,
}

impl PermissionUser {
    pub fn user_id(&self) -> Uuid {
        self.user.user_id
    }

    pub fn has_permission(&self, permission: Permission) -> bool {
        self.user.has_permission(permission)
    }
}

impl std::ops::Deref for PermissionUser {
    type Target = AuthUser;

    fn deref(&self) -> &Self::Target {
        &self.user
    }
}

/// Optional authenticated user
///
/// Unlike `AuthUser`, this extractor doesn't fail if no authentication
/// is provided, allowing routes to support both authenticated and
/// unauthenticated access.
///
/// # Example
/// ```no_run
/// use auth::middleware::OptionalAuthUser;
///
/// async fn maybe_protected_route(user: OptionalAuthUser) {
///     match user.0 {
///         Some(auth_user) => println!("Authenticated: {}", auth_user.user_id),
///         None => println!("Anonymous user"),
///     }
/// }
/// ```
pub struct OptionalAuthUser(pub Option<AuthUser>);

#[async_trait]
impl<S> FromRequestParts<S> for OptionalAuthUser
where
    S: Send + Sync,
    AuthState: FromRef<S>,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let user = AuthUser::from_request_parts(parts, state).await.ok();
        Ok(OptionalAuthUser(user))
    }
}

/// Macro to create a permission checker
///
/// # Example
/// ```ignore
/// use auth::require_permission;
/// use auth::permissions::Permission;
///
/// async fn handler(user: AuthUser) -> Result<(), AuthError> {
///     require_permission!(user, Permission::Download)?;
///     // Route logic here
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! require_permission {
    ($user:expr, $permission:expr) => {
        if !$user.has_permission($permission) {
            return Err($crate::middleware::AuthError::forbidden());
        }
    };
}

/// Macro to require any of multiple permissions
#[macro_export]
macro_rules! require_any_permission {
    ($user:expr, $($permission:expr),+) => {
        if !$user.has_any_permission(&[$($permission),+]) {
            return Err($crate::middleware::AuthError::forbidden());
        }
    };
}

/// Macro to require all of multiple permissions
#[macro_export]
macro_rules! require_all_permissions {
    ($user:expr, $($permission:expr),+) => {
        if !$user.has_all_permissions(&[$($permission),+]) {
            return Err($crate::middleware::AuthError::forbidden());
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_user_permissions() {
        let user = AuthUser {
            user_id: Uuid::new_v4(),
            permissions: vec![Permission::Download, Permission::UploadTorrent],
            token_id: Uuid::new_v4(),
        };

        assert!(user.has_permission(Permission::Download));
        assert!(!user.has_permission(Permission::BanUsers));

        assert!(user.has_any_permission(&[Permission::Download, Permission::BanUsers]));
        assert!(!user.has_any_permission(&[Permission::BanUsers, Permission::SiteAdmin]));

        assert!(user.has_all_permissions(&[Permission::Download, Permission::UploadTorrent]));
        assert!(!user.has_all_permissions(&[Permission::Download, Permission::BanUsers]));
    }

    #[test]
    fn test_auth_error_creation() {
        let err = AuthError::unauthorized();
        assert_eq!(err.error, "unauthorized");

        let err = AuthError::invalid_token();
        assert_eq!(err.error, "invalid_token");

        let err = AuthError::forbidden();
        assert_eq!(err.error, "forbidden");
    }

    #[test]
    fn test_permission_user_deref() {
        let auth_user = AuthUser {
            user_id: Uuid::new_v4(),
            permissions: vec![Permission::Download],
            token_id: Uuid::new_v4(),
        };

        let perm_user = PermissionUser {
            user: auth_user.clone(),
        };

        assert_eq!(perm_user.user_id, auth_user.user_id);
        assert_eq!(perm_user.permissions, auth_user.permissions);
    }
}
