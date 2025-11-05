//! Freeleech system
//!
//! This module implements a three-tier freeleech system:
//! 1. **Global Freeleech**: Applied to specific torrents (set by staff)
//! 2. **Personal Freeleech Tokens**: One-time use tokens purchased with bonus points
//! 3. **Temporary Freeleech Windows**: Time-limited freeleech periods
//!
//! Freeleech allows users to download torrents without it counting against their
//! download statistics, helping maintain or improve their ratio.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

/// Freeleech-related errors
#[derive(Debug, Error)]
pub enum FreeleechError {
    #[error("Token not found: {0}")]
    TokenNotFound(Uuid),

    #[error("Token already used or expired")]
    TokenInvalid,

    #[error("Token does not belong to user")]
    TokenNotOwned,

    #[error("Insufficient bonus points: required {required}, available {available}")]
    InsufficientBonus { required: f64, available: f64 },

    #[error("User not found: {0}")]
    UserNotFound(Uuid),

    #[error("Torrent not found: {0}")]
    TorrentNotFound(Uuid),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Freeleech type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FreeleechType {
    /// Global freeleech applied to torrent
    Global,
    /// Personal freeleech token
    Personal,
    /// Temporary freeleech window
    Temporary,
}

/// Token status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TokenStatus {
    /// Token is available to use
    Available,
    /// Token is active on a torrent
    Active,
    /// Token has been used and expired
    Used,
    /// Token expired without being used
    Expired,
}

/// Freeleech token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeleechToken {
    /// Token ID
    pub id: Uuid,

    /// Owner user ID
    pub user_id: Uuid,

    /// Token status
    pub status: TokenStatus,

    /// Duration in hours
    pub duration_hours: i32,

    /// Cost in bonus points (for record keeping)
    pub cost: f64,

    /// Torrent ID (when activated)
    pub torrent_id: Option<Uuid>,

    /// Activated at timestamp
    pub activated_at: Option<DateTime<Utc>>,

    /// Expires at timestamp (set when activated)
    pub expires_at: Option<DateTime<Utc>>,

    /// Created at timestamp
    pub created_at: DateTime<Utc>,
}

impl FreeleechToken {
    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Check if token is usable
    pub fn is_usable(&self) -> bool {
        self.status == TokenStatus::Available && !self.is_expired()
    }
}

/// Global freeleech settings for a torrent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalFreeleech {
    /// Torrent ID
    pub torrent_id: Uuid,

    /// Percentage of download that counts (0-100, 0 = full freeleech)
    pub download_factor: i32,

    /// Percentage of upload that counts (0-200, 100 = normal, 200 = double)
    pub upload_factor: i32,

    /// Reason for freeleech
    pub reason: String,

    /// Set by user ID (staff member)
    pub set_by: Uuid,

    /// Expires at timestamp (None = permanent)
    pub expires_at: Option<DateTime<Utc>>,

    /// Created at timestamp
    pub created_at: DateTime<Utc>,
}

impl GlobalFreeleech {
    /// Check if this freeleech is still active
    pub fn is_active(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() <= expires_at
        } else {
            true // Permanent freeleech
        }
    }
}

/// Temporary freeleech window (site-wide)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporaryFreeleech {
    /// Window ID
    pub id: Uuid,

    /// Window name
    pub name: String,

    /// Description
    pub description: String,

    /// Download factor (0-100)
    pub download_factor: i32,

    /// Upload factor (0-200)
    pub upload_factor: i32,

    /// Start time
    pub start_time: DateTime<Utc>,

    /// End time
    pub end_time: DateTime<Utc>,

    /// Created at timestamp
    pub created_at: DateTime<Utc>,
}

impl TemporaryFreeleech {
    /// Check if this window is currently active
    pub fn is_active(&self) -> bool {
        let now = Utc::now();
        now >= self.start_time && now <= self.end_time
    }
}

/// Freeleech service for managing freeleech tokens and settings
pub struct FreeleechService {
    db: PgPool,
}

impl FreeleechService {
    /// Create a new freeleech service
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Purchase a freeleech token with bonus points
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `cost` - The cost in bonus points
    /// * `duration_hours` - Duration of the token in hours (default: 24)
    ///
    /// # Returns
    ///
    /// Returns the created token
    pub async fn purchase_token(
        &self,
        user_id: Uuid,
        cost: f64,
        duration_hours: Option<i32>,
    ) -> Result<FreeleechToken, FreeleechError> {
        let duration = duration_hours.unwrap_or(24);

        // Check user's bonus balance
        let current_balance = sqlx::query_scalar!(
            r#"
            SELECT seedbonus
            FROM user_statistics
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(FreeleechError::UserNotFound(user_id))?;

        if current_balance < cost {
            return Err(FreeleechError::InsufficientBonus {
                required: cost,
                available: current_balance,
            });
        }

        let mut tx = self.db.begin().await?;

        // Deduct bonus points
        sqlx::query!(
            r#"
            UPDATE user_statistics
            SET seedbonus = seedbonus - $2, updated_at = NOW()
            WHERE user_id = $1
            "#,
            user_id,
            cost
        )
        .execute(&mut *tx)
        .await?;

        // Create token
        let token_id = Uuid::new_v4();
        let token = sqlx::query_as!(
            FreeleechToken,
            r#"
            INSERT INTO freeleech_tokens
                (id, user_id, status, duration_hours, cost, created_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            RETURNING
                id,
                user_id,
                status as "status: TokenStatus",
                duration_hours,
                cost,
                torrent_id,
                activated_at,
                expires_at,
                created_at
            "#,
            token_id,
            user_id,
            TokenStatus::Available as TokenStatus,
            duration,
            cost
        )
        .fetch_one(&mut *tx)
        .await?;

        // Record bonus transaction
        sqlx::query!(
            r#"
            INSERT INTO bonus_transactions
                (id, user_id, transaction_type, amount, balance_after, description, created_at)
            VALUES ($1, $2, 'purchased_freeleech', $3, $4, $5, NOW())
            "#,
            Uuid::new_v4(),
            user_id,
            -cost,
            current_balance - cost,
            format!("Purchased freeleech token ({} hours)", duration)
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(token)
    }

    /// Activate a freeleech token on a torrent
    ///
    /// # Arguments
    ///
    /// * `token_id` - The token ID
    /// * `user_id` - The user ID (for verification)
    /// * `torrent_id` - The torrent ID to apply token to
    pub async fn activate_token(
        &self,
        token_id: Uuid,
        user_id: Uuid,
        torrent_id: Uuid,
    ) -> Result<FreeleechToken, FreeleechError> {
        // Verify torrent exists
        let torrent_exists = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(SELECT 1 FROM torrents WHERE id = $1)
            "#,
            torrent_id
        )
        .fetch_one(&self.db)
        .await?
        .unwrap_or(false);

        if !torrent_exists {
            return Err(FreeleechError::TorrentNotFound(torrent_id));
        }

        // Get token and verify ownership
        let token = self.get_token(token_id).await?;

        if token.user_id != user_id {
            return Err(FreeleechError::TokenNotOwned);
        }

        if !token.is_usable() {
            return Err(FreeleechError::TokenInvalid);
        }

        // Activate token
        let expires_at = Utc::now() + Duration::hours(token.duration_hours as i64);

        let updated_token = sqlx::query_as!(
            FreeleechToken,
            r#"
            UPDATE freeleech_tokens
            SET
                status = $2,
                torrent_id = $3,
                activated_at = NOW(),
                expires_at = $4
            WHERE id = $1
            RETURNING
                id,
                user_id,
                status as "status: TokenStatus",
                duration_hours,
                cost,
                torrent_id,
                activated_at,
                expires_at,
                created_at
            "#,
            token_id,
            TokenStatus::Active as TokenStatus,
            torrent_id,
            expires_at
        )
        .fetch_one(&self.db)
        .await?;

        Ok(updated_token)
    }

    /// Get a freeleech token by ID
    ///
    /// # Arguments
    ///
    /// * `token_id` - The token ID
    pub async fn get_token(&self, token_id: Uuid) -> Result<FreeleechToken, FreeleechError> {
        let token = sqlx::query_as!(
            FreeleechToken,
            r#"
            SELECT
                id,
                user_id,
                status as "status: TokenStatus",
                duration_hours,
                cost,
                torrent_id,
                activated_at,
                expires_at,
                created_at
            FROM freeleech_tokens
            WHERE id = $1
            "#,
            token_id
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(FreeleechError::TokenNotFound(token_id))?;

        Ok(token)
    }

    /// Get user's freeleech tokens
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `include_used` - Whether to include used/expired tokens
    pub async fn get_user_tokens(
        &self,
        user_id: Uuid,
        include_used: bool,
    ) -> Result<Vec<FreeleechToken>, FreeleechError> {
        let tokens = if include_used {
            sqlx::query_as!(
                FreeleechToken,
                r#"
                SELECT
                    id,
                    user_id,
                    status as "status: TokenStatus",
                    duration_hours,
                    cost,
                    torrent_id,
                    activated_at,
                    expires_at,
                    created_at
                FROM freeleech_tokens
                WHERE user_id = $1
                ORDER BY created_at DESC
                "#,
                user_id
            )
            .fetch_all(&self.db)
            .await?
        } else {
            sqlx::query_as!(
                FreeleechToken,
                r#"
                SELECT
                    id,
                    user_id,
                    status as "status: TokenStatus",
                    duration_hours,
                    cost,
                    torrent_id,
                    activated_at,
                    expires_at,
                    created_at
                FROM freeleech_tokens
                WHERE user_id = $1 AND status IN ('available', 'active')
                ORDER BY created_at DESC
                "#,
                user_id
            )
            .fetch_all(&self.db)
            .await?
        };

        Ok(tokens)
    }

    /// Check if a torrent has freeleech for a user
    ///
    /// # Arguments
    ///
    /// * `torrent_id` - The torrent ID
    /// * `user_id` - The user ID (for personal tokens)
    ///
    /// # Returns
    ///
    /// Returns (has_freeleech, download_factor, upload_factor)
    pub async fn check_freeleech(
        &self,
        torrent_id: Uuid,
        user_id: Uuid,
    ) -> Result<(bool, i32, i32), FreeleechError> {
        // Check global freeleech on torrent
        if let Some(global) = self.get_global_freeleech(torrent_id).await? {
            if global.is_active() {
                return Ok((true, global.download_factor, global.upload_factor));
            }
        }

        // Check personal token
        let has_personal_token = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM freeleech_tokens
                WHERE user_id = $1
                    AND torrent_id = $2
                    AND status = 'active'
                    AND expires_at > NOW()
            )
            "#,
            user_id,
            torrent_id
        )
        .fetch_one(&self.db)
        .await?
        .unwrap_or(false);

        if has_personal_token {
            return Ok((true, 0, 100)); // Full freeleech with personal token
        }

        // Check temporary freeleech window
        if let Some(temp) = self.get_active_temporary_freeleech().await? {
            return Ok((true, temp.download_factor, temp.upload_factor));
        }

        Ok((false, 100, 100)) // No freeleech
    }

    /// Set global freeleech on a torrent
    ///
    /// # Arguments
    ///
    /// * `torrent_id` - The torrent ID
    /// * `download_factor` - Download factor (0-100)
    /// * `upload_factor` - Upload factor (0-200)
    /// * `reason` - Reason for freeleech
    /// * `set_by` - Staff user ID
    /// * `expires_at` - Optional expiry time
    pub async fn set_global_freeleech(
        &self,
        torrent_id: Uuid,
        download_factor: i32,
        upload_factor: i32,
        reason: String,
        set_by: Uuid,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<GlobalFreeleech, FreeleechError> {
        let freeleech = sqlx::query_as!(
            GlobalFreeleech,
            r#"
            INSERT INTO global_freeleech
                (torrent_id, download_factor, upload_factor, reason, set_by, expires_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            ON CONFLICT (torrent_id)
            DO UPDATE SET
                download_factor = $2,
                upload_factor = $3,
                reason = $4,
                set_by = $5,
                expires_at = $6
            RETURNING torrent_id, download_factor, upload_factor, reason, set_by, expires_at, created_at
            "#,
            torrent_id,
            download_factor,
            upload_factor,
            reason,
            set_by,
            expires_at
        )
        .fetch_one(&self.db)
        .await?;

        Ok(freeleech)
    }

    /// Get global freeleech for a torrent
    ///
    /// # Arguments
    ///
    /// * `torrent_id` - The torrent ID
    pub async fn get_global_freeleech(
        &self,
        torrent_id: Uuid,
    ) -> Result<Option<GlobalFreeleech>, FreeleechError> {
        let freeleech = sqlx::query_as!(
            GlobalFreeleech,
            r#"
            SELECT torrent_id, download_factor, upload_factor, reason, set_by, expires_at, created_at
            FROM global_freeleech
            WHERE torrent_id = $1
            "#,
            torrent_id
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(freeleech)
    }

    /// Remove global freeleech from a torrent
    ///
    /// # Arguments
    ///
    /// * `torrent_id` - The torrent ID
    pub async fn remove_global_freeleech(&self, torrent_id: Uuid) -> Result<(), FreeleechError> {
        sqlx::query!(
            r#"
            DELETE FROM global_freeleech
            WHERE torrent_id = $1
            "#,
            torrent_id
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Get currently active temporary freeleech window
    pub async fn get_active_temporary_freeleech(
        &self,
    ) -> Result<Option<TemporaryFreeleech>, FreeleechError> {
        let now = Utc::now();

        let freeleech = sqlx::query_as!(
            TemporaryFreeleech,
            r#"
            SELECT id, name, description, download_factor, upload_factor, start_time, end_time, created_at
            FROM temporary_freeleech
            WHERE start_time <= $1 AND end_time >= $1
            ORDER BY start_time DESC
            LIMIT 1
            "#,
            now
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(freeleech)
    }

    /// Expire old tokens (cleanup job)
    pub async fn expire_old_tokens(&self) -> Result<i64, FreeleechError> {
        let result = sqlx::query!(
            r#"
            UPDATE freeleech_tokens
            SET status = 'expired'
            WHERE status = 'active'
                AND expires_at <= NOW()
            "#
        )
        .execute(&self.db)
        .await?;

        Ok(result.rows_affected() as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_is_expired() {
        let mut token = FreeleechToken {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            status: TokenStatus::Active,
            duration_hours: 24,
            cost: 1000.0,
            torrent_id: Some(Uuid::new_v4()),
            activated_at: Some(Utc::now() - Duration::hours(25)),
            expires_at: Some(Utc::now() - Duration::hours(1)),
            created_at: Utc::now() - Duration::hours(26),
        };

        assert!(token.is_expired());

        // Not expired
        token.expires_at = Some(Utc::now() + Duration::hours(1));
        assert!(!token.is_expired());

        // No expiry set
        token.expires_at = None;
        assert!(!token.is_expired());
    }

    #[test]
    fn test_token_is_usable() {
        let token = FreeleechToken {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            status: TokenStatus::Available,
            duration_hours: 24,
            cost: 1000.0,
            torrent_id: None,
            activated_at: None,
            expires_at: None,
            created_at: Utc::now(),
        };

        assert!(token.is_usable());

        // Used token
        let mut used_token = token.clone();
        used_token.status = TokenStatus::Used;
        assert!(!used_token.is_usable());
    }

    #[test]
    fn test_global_freeleech_is_active() {
        let freeleech = GlobalFreeleech {
            torrent_id: Uuid::new_v4(),
            download_factor: 0,
            upload_factor: 100,
            reason: "Test".to_string(),
            set_by: Uuid::new_v4(),
            expires_at: Some(Utc::now() + Duration::hours(1)),
            created_at: Utc::now(),
        };

        assert!(freeleech.is_active());

        // Expired
        let mut expired = freeleech.clone();
        expired.expires_at = Some(Utc::now() - Duration::hours(1));
        assert!(!expired.is_active());

        // Permanent
        let mut permanent = freeleech;
        permanent.expires_at = None;
        assert!(permanent.is_active());
    }

    #[test]
    fn test_temporary_freeleech_is_active() {
        let freeleech = TemporaryFreeleech {
            id: Uuid::new_v4(),
            name: "Test Window".to_string(),
            description: "Test".to_string(),
            download_factor: 0,
            upload_factor: 100,
            start_time: Utc::now() - Duration::hours(1),
            end_time: Utc::now() + Duration::hours(1),
            created_at: Utc::now() - Duration::hours(2),
        };

        assert!(freeleech.is_active());

        // Not started yet
        let mut future = freeleech.clone();
        future.start_time = Utc::now() + Duration::hours(1);
        future.end_time = Utc::now() + Duration::hours(2);
        assert!(!future.is_active());

        // Already ended
        let mut past = freeleech;
        past.start_time = Utc::now() - Duration::hours(2);
        past.end_time = Utc::now() - Duration::hours(1);
        assert!(!past.is_active());
    }
}
