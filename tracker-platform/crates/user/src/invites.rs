//! Invitation system with tree tracking
//!
//! This module provides an invitation system for restricted registration:
//! - Generate invitation codes
//! - Invite tree tracking (who invited whom)
//! - Invitation quota by user class
//! - Track invite success rate
//! - Disable invites for bad inviters

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

/// Invite-related errors
#[derive(Debug, Error)]
pub enum InviteError {
    #[error("Invitation not found: {0}")]
    NotFound(String),

    #[error("Invitation already used")]
    AlreadyUsed,

    #[error("Invitation expired")]
    Expired,

    #[error("User has no invites remaining")]
    NoInvitesRemaining,

    #[error("User invite privileges disabled")]
    InvitesDisabled,

    #[error("User not found: {0}")]
    UserNotFound(Uuid),

    #[error("Invalid invite code format")]
    InvalidCodeFormat,

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Invitation code status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InviteStatus {
    /// Available to use
    Available,
    /// Used successfully
    Used,
    /// Expired without being used
    Expired,
    /// Revoked by inviter or admin
    Revoked,
}

/// Invitation code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invitation {
    /// Invitation code (unique string)
    pub code: String,

    /// Inviter user ID
    pub inviter_id: Uuid,

    /// Invitee user ID (when used)
    pub invitee_id: Option<Uuid>,

    /// Status
    pub status: InviteStatus,

    /// Expires at timestamp
    pub expires_at: DateTime<Utc>,

    /// Used at timestamp
    pub used_at: Option<DateTime<Utc>>,

    /// Custom message from inviter
    pub message: Option<String>,

    /// Created at timestamp
    pub created_at: DateTime<Utc>,
}

impl Invitation {
    /// Check if invitation is valid (available and not expired)
    pub fn is_valid(&self) -> bool {
        self.status == InviteStatus::Available && Utc::now() < self.expires_at
    }
}

/// Invite tree node (who invited whom)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteTree {
    /// User ID
    pub user_id: Uuid,

    /// Username
    pub username: String,

    /// Inviter user ID (None if original user)
    pub inviter_id: Option<Uuid>,

    /// Inviter username
    pub inviter_username: Option<String>,

    /// Number of users this user has invited
    pub invitee_count: i32,

    /// Number of successful invites (active users)
    pub successful_invites: i32,

    /// Number of failed invites (disabled/banned users)
    pub failed_invites: i32,

    /// Joined at timestamp
    pub joined_at: DateTime<Utc>,
}

/// User invite statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteStats {
    /// User ID
    pub user_id: Uuid,

    /// Total invites available
    pub total_invites: i32,

    /// Invites used
    pub invites_used: i32,

    /// Invites remaining
    pub invites_remaining: i32,

    /// Successful invites (active users)
    pub successful_invites: i32,

    /// Failed invites (disabled/banned users)
    pub failed_invites: i32,

    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,

    /// Whether invite privileges are enabled
    pub invites_enabled: bool,
}

/// Invite service for managing invitations
pub struct InviteService {
    db: PgPool,
}

impl InviteService {
    /// Create a new invite service
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Generate a new invitation code
    ///
    /// # Arguments
    ///
    /// * `inviter_id` - The inviter's user ID
    /// * `expires_in_days` - Number of days until expiration (default: 7)
    /// * `message` - Optional custom message
    pub async fn generate_invite(
        &self,
        inviter_id: Uuid,
        expires_in_days: Option<i64>,
        message: Option<String>,
    ) -> Result<Invitation, InviteError> {
        // Check if user has invites available
        let stats = self.get_invite_stats(inviter_id).await?;

        if !stats.invites_enabled {
            return Err(InviteError::InvitesDisabled);
        }

        if stats.invites_remaining <= 0 {
            return Err(InviteError::NoInvitesRemaining);
        }

        // Generate unique code
        let code = self.generate_unique_code().await?;

        let expires_at = Utc::now() + Duration::days(expires_in_days.unwrap_or(7));

        let invitation = sqlx::query_as!(
            Invitation,
            r#"
            INSERT INTO invitations
                (code, inviter_id, status, expires_at, message, created_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            RETURNING
                code,
                inviter_id,
                invitee_id,
                status as "status: InviteStatus",
                expires_at,
                used_at,
                message,
                created_at
            "#,
            code,
            inviter_id,
            InviteStatus::Available as InviteStatus,
            expires_at,
            message
        )
        .fetch_one(&self.db)
        .await?;

        Ok(invitation)
    }

    /// Use an invitation code
    ///
    /// # Arguments
    ///
    /// * `code` - The invitation code
    /// * `invitee_id` - The new user's ID
    pub async fn use_invite(
        &self,
        code: &str,
        invitee_id: Uuid,
    ) -> Result<Invitation, InviteError> {
        // Get invitation
        let invitation = self.get_invite_by_code(code).await?;

        // Validate invitation
        if !invitation.is_valid() {
            if invitation.status == InviteStatus::Used {
                return Err(InviteError::AlreadyUsed);
            } else if Utc::now() >= invitation.expires_at {
                return Err(InviteError::Expired);
            } else {
                return Err(InviteError::NotFound(code.to_string()));
            }
        }

        // Mark invitation as used
        let used_invitation = sqlx::query_as!(
            Invitation,
            r#"
            UPDATE invitations
            SET
                status = $2,
                invitee_id = $3,
                used_at = NOW()
            WHERE code = $1
            RETURNING
                code,
                inviter_id,
                invitee_id,
                status as "status: InviteStatus",
                expires_at,
                used_at,
                message,
                created_at
            "#,
            code,
            InviteStatus::Used as InviteStatus,
            invitee_id
        )
        .fetch_one(&self.db)
        .await?;

        Ok(used_invitation)
    }

    /// Get invitation by code
    ///
    /// # Arguments
    ///
    /// * `code` - The invitation code
    pub async fn get_invite_by_code(&self, code: &str) -> Result<Invitation, InviteError> {
        let invitation = sqlx::query_as!(
            Invitation,
            r#"
            SELECT
                code,
                inviter_id,
                invitee_id,
                status as "status: InviteStatus",
                expires_at,
                used_at,
                message,
                created_at
            FROM invitations
            WHERE code = $1
            "#,
            code
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| InviteError::NotFound(code.to_string()))?;

        Ok(invitation)
    }

    /// Get user's invitations
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `include_used` - Whether to include used invitations
    pub async fn get_user_invites(
        &self,
        user_id: Uuid,
        include_used: bool,
    ) -> Result<Vec<Invitation>, InviteError> {
        let invitations = if include_used {
            sqlx::query_as!(
                Invitation,
                r#"
                SELECT
                    code,
                    inviter_id,
                    invitee_id,
                    status as "status: InviteStatus",
                    expires_at,
                    used_at,
                    message,
                    created_at
                FROM invitations
                WHERE inviter_id = $1
                ORDER BY created_at DESC
                "#,
                user_id
            )
            .fetch_all(&self.db)
            .await?
        } else {
            sqlx::query_as!(
                Invitation,
                r#"
                SELECT
                    code,
                    inviter_id,
                    invitee_id,
                    status as "status: InviteStatus",
                    expires_at,
                    used_at,
                    message,
                    created_at
                FROM invitations
                WHERE inviter_id = $1 AND status = 'available'
                ORDER BY created_at DESC
                "#,
                user_id
            )
            .fetch_all(&self.db)
            .await?
        };

        Ok(invitations)
    }

    /// Get invite statistics for a user
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    pub async fn get_invite_stats(&self, user_id: Uuid) -> Result<InviteStats, InviteError> {
        let stats = sqlx::query!(
            r#"
            SELECT
                u.invite_quota as total_invites,
                u.invites_enabled,
                COALESCE(COUNT(i.code), 0)::int as "invites_used!",
                COALESCE(SUM(CASE WHEN invitee.enabled = true THEN 1 ELSE 0 END), 0)::int as "successful!",
                COALESCE(SUM(CASE WHEN invitee.enabled = false THEN 1 ELSE 0 END), 0)::int as "failed!"
            FROM users u
            LEFT JOIN invitations i ON u.id = i.inviter_id AND i.status = 'used'
            LEFT JOIN users invitee ON i.invitee_id = invitee.id
            WHERE u.id = $1
            GROUP BY u.id, u.invite_quota, u.invites_enabled
            "#,
            user_id
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(InviteError::UserNotFound(user_id))?;

        let invites_used = stats.invites_used;
        let total_invites = stats.total_invites;
        let invites_remaining = (total_invites - invites_used).max(0);
        let successful = stats.successful;
        let failed = stats.failed;

        let success_rate = if invites_used > 0 {
            successful as f64 / invites_used as f64
        } else {
            0.0
        };

        Ok(InviteStats {
            user_id,
            total_invites,
            invites_used,
            invites_remaining,
            successful_invites: successful,
            failed_invites: failed,
            success_rate,
            invites_enabled: stats.invites_enabled,
        })
    }

    /// Get invite tree for a user (who invited this user and who they invited)
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    pub async fn get_invite_tree(&self, user_id: Uuid) -> Result<InviteTree, InviteError> {
        let tree = sqlx::query!(
            r#"
            SELECT
                u.id as user_id,
                u.username,
                i.inviter_id,
                inviter.username as inviter_username,
                u.created_at as joined_at,
                COALESCE(COUNT(DISTINCT invites.invitee_id), 0)::int as "invitee_count!",
                COALESCE(SUM(CASE WHEN invitee.enabled = true THEN 1 ELSE 0 END), 0)::int as "successful!",
                COALESCE(SUM(CASE WHEN invitee.enabled = false THEN 1 ELSE 0 END), 0)::int as "failed!"
            FROM users u
            LEFT JOIN invitations i ON u.id = i.invitee_id
            LEFT JOIN users inviter ON i.inviter_id = inviter.id
            LEFT JOIN invitations invites ON u.id = invites.inviter_id AND invites.status = 'used'
            LEFT JOIN users invitee ON invites.invitee_id = invitee.id
            WHERE u.id = $1
            GROUP BY u.id, u.username, i.inviter_id, inviter.username, u.created_at
            "#,
            user_id
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(InviteError::UserNotFound(user_id))?;

        Ok(InviteTree {
            user_id: tree.user_id,
            username: tree.username,
            inviter_id: tree.inviter_id,
            inviter_username: tree.inviter_username,
            invitee_count: tree.invitee_count,
            successful_invites: tree.successful,
            failed_invites: tree.failed,
            joined_at: tree.joined_at,
        })
    }

    /// Get full invite tree chain (from root to user)
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    pub async fn get_invite_chain(&self, user_id: Uuid) -> Result<Vec<InviteTree>, InviteError> {
        let chain = sqlx::query!(
            r#"
            WITH RECURSIVE invite_chain AS (
                -- Base case: start with the given user
                SELECT
                    u.id,
                    u.username,
                    i.inviter_id,
                    u.created_at,
                    1 as depth
                FROM users u
                LEFT JOIN invitations i ON u.id = i.invitee_id
                WHERE u.id = $1

                UNION ALL

                -- Recursive case: follow inviter chain
                SELECT
                    u.id,
                    u.username,
                    i.inviter_id,
                    u.created_at,
                    ic.depth + 1
                FROM invite_chain ic
                JOIN users u ON ic.inviter_id = u.id
                LEFT JOIN invitations i ON u.id = i.invitee_id
                WHERE ic.inviter_id IS NOT NULL
            )
            SELECT
                ic.id as user_id,
                ic.username,
                ic.inviter_id,
                inviter.username as inviter_username,
                ic.created_at as joined_at,
                ic.depth,
                COALESCE(COUNT(DISTINCT invites.invitee_id), 0)::int as "invitee_count!",
                COALESCE(SUM(CASE WHEN invitee.enabled = true THEN 1 ELSE 0 END), 0)::int as "successful!",
                COALESCE(SUM(CASE WHEN invitee.enabled = false THEN 1 ELSE 0 END), 0)::int as "failed!"
            FROM invite_chain ic
            LEFT JOIN users inviter ON ic.inviter_id = inviter.id
            LEFT JOIN invitations invites ON ic.id = invites.inviter_id AND invites.status = 'used'
            LEFT JOIN users invitee ON invites.invitee_id = invitee.id
            GROUP BY ic.id, ic.username, ic.inviter_id, inviter.username, ic.created_at, ic.depth
            ORDER BY ic.depth DESC
            "#,
            user_id
        )
        .fetch_all(&self.db)
        .await?;

        Ok(chain
            .into_iter()
            .map(|row| InviteTree {
                user_id: row.user_id,
                username: row.username,
                inviter_id: row.inviter_id,
                inviter_username: row.inviter_username,
                invitee_count: row.invitee_count,
                successful_invites: row.successful,
                failed_invites: row.failed,
                joined_at: row.joined_at,
            })
            .collect())
    }

    /// Revoke an invitation
    ///
    /// # Arguments
    ///
    /// * `code` - The invitation code
    /// * `user_id` - The user revoking (must be inviter or admin)
    pub async fn revoke_invite(
        &self,
        code: &str,
        user_id: Uuid,
    ) -> Result<Invitation, InviteError> {
        let invitation = sqlx::query_as!(
            Invitation,
            r#"
            UPDATE invitations
            SET status = $3
            WHERE code = $1 AND (inviter_id = $2 OR $2 IN (SELECT id FROM users WHERE role = 'admin'))
            RETURNING
                code,
                inviter_id,
                invitee_id,
                status as "status: InviteStatus",
                expires_at,
                used_at,
                message,
                created_at
            "#,
            code,
            user_id,
            InviteStatus::Revoked as InviteStatus
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| InviteError::NotFound(code.to_string()))?;

        Ok(invitation)
    }

    /// Enable or disable invite privileges for a user
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `enabled` - Whether to enable or disable
    pub async fn set_invite_privileges(
        &self,
        user_id: Uuid,
        enabled: bool,
    ) -> Result<(), InviteError> {
        sqlx::query!(
            r#"
            UPDATE users
            SET invites_enabled = $2
            WHERE id = $1
            "#,
            user_id,
            enabled
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Update invite quota for a user
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `quota` - New invite quota
    pub async fn update_invite_quota(
        &self,
        user_id: Uuid,
        quota: i32,
    ) -> Result<(), InviteError> {
        sqlx::query!(
            r#"
            UPDATE users
            SET invite_quota = $2
            WHERE id = $1
            "#,
            user_id,
            quota
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Expire old invitations (cleanup job)
    pub async fn expire_old_invites(&self) -> Result<i64, InviteError> {
        let result = sqlx::query!(
            r#"
            UPDATE invitations
            SET status = 'expired'
            WHERE status = 'available'
                AND expires_at <= NOW()
            "#
        )
        .execute(&self.db)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    /// Generate a unique invitation code
    async fn generate_unique_code(&self) -> Result<String, InviteError> {
        use rand::Rng;

        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        const CODE_LENGTH: usize = 16;

        let mut rng = rand::thread_rng();

        // Try up to 10 times to generate a unique code
        for _ in 0..10 {
            let code: String = (0..CODE_LENGTH)
                .map(|_| {
                    let idx = rng.gen_range(0..CHARSET.len());
                    CHARSET[idx] as char
                })
                .collect();

            // Check if code exists
            let exists = sqlx::query_scalar!(
                r#"
                SELECT EXISTS(SELECT 1 FROM invitations WHERE code = $1)
                "#,
                code
            )
            .fetch_one(&self.db)
            .await?
            .unwrap_or(false);

            if !exists {
                return Ok(code);
            }
        }

        Err(InviteError::InvalidCodeFormat)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invitation_is_valid() {
        let invitation = Invitation {
            code: "TEST123".to_string(),
            inviter_id: Uuid::new_v4(),
            invitee_id: None,
            status: InviteStatus::Available,
            expires_at: Utc::now() + Duration::days(7),
            used_at: None,
            message: None,
            created_at: Utc::now(),
        };

        assert!(invitation.is_valid());

        // Used invitation
        let mut used = invitation.clone();
        used.status = InviteStatus::Used;
        assert!(!used.is_valid());

        // Expired invitation
        let mut expired = invitation;
        expired.expires_at = Utc::now() - Duration::days(1);
        assert!(!expired.is_valid());
    }

    #[test]
    fn test_invite_status() {
        let statuses = vec![
            InviteStatus::Available,
            InviteStatus::Used,
            InviteStatus::Expired,
            InviteStatus::Revoked,
        ];

        // Ensure all statuses are distinct
        for (i, s1) in statuses.iter().enumerate() {
            for (j, s2) in statuses.iter().enumerate() {
                if i == j {
                    assert_eq!(s1, s2);
                } else {
                    assert_ne!(s1, s2);
                }
            }
        }
    }

    #[test]
    fn test_invite_stats_success_rate() {
        let stats = InviteStats {
            user_id: Uuid::new_v4(),
            total_invites: 10,
            invites_used: 5,
            invites_remaining: 5,
            successful_invites: 4,
            failed_invites: 1,
            success_rate: 0.8,
            invites_enabled: true,
        };

        assert_eq!(stats.success_rate, 0.8);
        assert_eq!(stats.invites_remaining, 5);
    }
}
